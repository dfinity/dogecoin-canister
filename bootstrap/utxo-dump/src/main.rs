use bitcoin::{Address as BtcAddress, dogecoin::Address as DogeAddress, Network as BtcNetwork, dogecoin::Network as DogeNetwork, PubkeyHash, PublicKey, ScriptBuf};
use secp256k1::{PublicKey as Secp256k1Pk};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Cursor, Error, ErrorKind, Read, Write};
use libc::{rlimit, setrlimit, RLIMIT_NOFILE};

use std::path::Path;
use std::process::{exit};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use bitcoin::hashes::Hash;
use bitcoin::Script;
use clap::{Parser, ValueEnum};
use rusty_leveldb::{LdbIterator, Options, DB};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

const VERSION: &str = "1.0.0";

const DB_KEYS_UTXOS: char = 'C'; // 0x43 = 67
const DB_KEYS_UTXOS_LEGACY: char = 'c'; // 0x63 = 99

const DB_KEYS_OBFUSCATE_KEY_PREFIX: [u8;2] = [0x0e, 0x00];
const DB_KEYS_OBFUSCATE_KEY: &str = "obfuscate_key";

#[derive(Clone, Debug, ValueEnum)]
enum Blockchain {
    Bitcoin(BtcNetwork),
    Dogecoin(DogeNetwork),
}

impl Blockchain {
    fn get_utxo_prefix(&self) -> u8 {
        match self {
            Blockchain::Bitcoin(_) => u8::try_from(DB_KEYS_UTXOS).unwrap(),
            Blockchain::Dogecoin(_) => u8::try_from(DB_KEYS_UTXOS_LEGACY).unwrap(),
        }
    }

    fn ticker(&self) -> &str {
        match self {
            Blockchain::Bitcoin(_) => "BTC",
            Blockchain::Dogecoin(_) => "DOGE",
        }
    }
    
    fn create_p2pkh_address(&self, pubkey_hash: PubkeyHash) -> Result<String> {
        match self {
            Blockchain::Bitcoin(network) => {
                Ok(BtcAddress::p2pkh(pubkey_hash, network).to_string())
            }
            Blockchain::Dogecoin(network) => {
                Ok(DogeAddress::p2pkh(pubkey_hash, network).to_string())
            }
        }
    }
    
    fn create_p2sh_address(&self, script: &Script) -> Result<String> {
        match self {
            Blockchain::Bitcoin(network) => {
                Ok(BtcAddress::p2sh(script, network)?.to_string())
            }
            Blockchain::Dogecoin(network) => {
                Ok(DogeAddress::p2sh(script, network)?.to_string())
            }
        }
    }

    fn deserialize_db_value(&self, value: Vec<u8>) -> Vec<DBOutput> {
        match self {
            Blockchain::Bitcoin(_) => {
                deserialize_db_value(self, value)
            }
            Blockchain::Dogecoin(_) => {
                deserialize_db_value_legacy(self, value)
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "utxo-dump")]
#[command(about = "Dumps UTXO set from chainstate LevelDB to CSV")]
#[command(version = VERSION)]
struct Args {
    /// Location of chainstate db
    #[arg(short = 'd', long = "db")]
    chainstate: String,

    /// Name of file to dump utxo list to
    #[arg(short = 'o', default_value = "utxodump.csv")]
    output_file: String,

    /// Fields to include in output
    #[arg(
        short = 'f',
        long = "fields",
        default_value = "height,txid,vout,amount,type,address,script,coinbase,nsize"
    )]
    fields: String,

    /// Blockchain (bitcoin, dogecoin)
    #[arg(short = 'b', long = "blockchain", default_value = "dogecoin")]
    blockchain: Blockchain,

    /// Is the chainstate leveldb for testnet?
    #[arg(long = "testnet")]
    testnet: bool,

    /// Print utxos as we process them (will be about ??? times slower)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Convert public keys in P2PK locking scripts to addresses also
    #[arg(long = "p2pkaddresses")]
    p2pk_addresses: bool,

    /// Do not display any progress or results
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check if OS is macOS and set rlimit
    #[cfg(target_os = "macos")]
    set_macos_rlimit(&args);

    let blockchain = args.blockchain;

    let testnet = args.testnet || args.chainstate.contains("testnet");

    if !Path::new(&args.chainstate).exists() {
        anyhow::bail!("Couldn't find {}", args.chainstate);
    }

    let fields_selected = validate_and_parse_fields(&args.fields)?;

    // Open LevelDB database
    let options = Options::default();
    let mut database = DB::open(&args.chainstate, options).context("Couldn't open LevelDB")?;

    // Create output file and CSV writer
    let output_file = File::create(&args.output_file).context("Failed to create output file")?;
    let mut writer = BufWriter::new(output_file);

    if !args.quiet {
        println!(
            "Processing {} and writing results to {}",
            args.chainstate, args.output_file
        );
    }

    // Write CSV header
    if !args.quiet {
        println!("{}", args.fields);
    }
    writeln!(writer, "{}", args.fields)?;

    // Initialize statistics
    let mut total_amount: i64 = 0;
    let mut script_type_count = HashMap::new();
    script_type_count.insert("p2pk", 0);
    script_type_count.insert("p2pkh", 0);
    script_type_count.insert("p2sh", 0);
    script_type_count.insert("p2ms", 0);
    script_type_count.insert("non-standard", 0);

    // Setup signal handling for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    std::thread::spawn(move || {
        let mut signals = Signals::new(TERM_SIGNALS).expect("Failed to register signal handler");
        for _sig in signals.forever() {
            r.store(false, Ordering::SeqCst);
            break;
        }
    });

    let mut iter = database.new_iter()?;
    let mut count = 0;

    // Read obfuscation key
    iter.reset();
    let (key, value) = iter.next().unwrap();
    let (key_prefix, key) = key.split_at(2);
    let key_str= String::from_utf8(key.to_vec()).expect("valid UTF-8");
    let mut obfuscate_key: Vec<u8> = Vec::new();
    if key_prefix == DB_KEYS_OBFUSCATE_KEY_PREFIX && key_str == DB_KEYS_OBFUSCATE_KEY {
        obfuscate_key = value.clone();
        if !args.quiet {
            println!(">>> Obfuscation key: {}", hex::encode(&obfuscate_key));
        }
    }
    if obfuscate_key.is_empty() { // TODO: replace with anyhow::bail!()?
        eprintln!("Error: No obfuscation key found in chainstate database.");
        eprintln!("This database may be corrupted or from an older Bitcoin version that doesn't use obfuscation.");
        eprintln!("Cannot process UTXO values without the obfuscation key.");
        exit(1);
    }

    // Iterate over LevelDB
    iter.seek_to_first();
    while iter.valid() {
        if !running.load(Ordering::SeqCst) {
            if !args.quiet {
                println!("Interrupt signal caught. Shutting down gracefully.");
            }
            break;
        }

        if let Some((key, value)) = iter.next() {
            // First byte in key indicates the type of key we've got for leveldb
            let prefix = key[0];
            if prefix == blockchain.get_utxo_prefix() {
                // ---------------
                // Key for Bitcoin
                // ---------------

                //      430000155b9869d56c66d9e86e3c01de38e3892a42b99949fe109ac034fff6583900
                //      <><--------------------------------------------------------------><>
                //      /                               |                                  \
                //  prefix                      txid (little-endian)                 vout (varint)

                // ----------------
                // Key for Dogecoin
                // ----------------

                // Ref: <https://en.bitcoin.it/wiki/Bitcoin_Core_0.11_(ch_2):_Data_Storage>

                //      6300007c264b7c8b886b0c7d086d7b42e1737d6c3fb94b85eaecda45be1eb3b6a0
                //      <><-------------------------------------------------------------->
                //      /                               |
                //  prefix                      txid (little-endian)

                let mut csv_output = HashMap::new();

                // Ignore first byte (size of the obfuscate key)
                let mut obfuscate_key_extended = obfuscate_key[1..].to_vec();

                // Extend obfuscate key to match value length
                // Example
                //   [8 175 184 95 99 240 37 253 115 1 161 4 33 81 167 111 145 131 0 233 37 232 118 180 123 120 78]    <- value (len = 27)
                //   [8 177 45 206 253 143 135 37 54]                                                                  <- obfuscate_key[1..] (len = 9)
                //   [8 177 45 206 253 143 135 37 54 8 177 45 206 253 143 135 37 54 8 177 45 206 253 143 135 37 54]    <- obfuscate_key_extended (len = 27)
                while obfuscate_key_extended.len() < value.len() {
                    let key_len = obfuscate_key[1..].len();
                    for i in 0..key_len {
                        if obfuscate_key_extended.len() >= value.len() {
                            break;
                        }
                        obfuscate_key_extended.push(obfuscate_key[1 + i]);
                    }
                }

                // XOR deobfuscate the value
                let mut xor_result = Vec::new();
                for i in 0..value.len() {
                    xor_result.push(value[i] ^ obfuscate_key_extended[i]);
                }

                let outputs = blockchain.deserialize_db_value(xor_result);

                for output in outputs {
                    // txid
                    if *fields_selected.get("txid").unwrap_or(&false) {
                        let txid_le = &key[1..33];

                        // Reverse byte order (little-endian to big-endian)
                        let mut txid = Vec::new();
                        for i in (0..txid_le.len()).rev() {
                            txid.push(txid_le[i]);
                        }
                        csv_output.insert("txid", hex::encode(txid));
                    }

                    // vout
                    if *fields_selected.get("vout").unwrap_or(&false) {
                        let index_bytes = &key[33..];
                        let vout = varint_decode(index_bytes).0;
                        csv_output.insert("vout", vout.to_string());
                    }

                    // coinbase
                    if *fields_selected.get("coinbase").unwrap_or(&false) {
                        csv_output.insert("coinbase", output.coinbase.to_string());
                    }

                    // amount
                    if *fields_selected.get("amount").unwrap_or(&false) {
                        let amount = output.tx_out.amount;
                        csv_output.insert("amount", amount.to_string());
                        total_amount += amount;
                    }

                    // nsize
                    if *fields_selected.get("nsize").unwrap_or(&false) {
                        csv_output.insert("nsize", output.tx_out.n_size.to_string());
                    }

                    // Address and script type processing
                    if *fields_selected.get("address").unwrap_or(&false)
                        || *fields_selected.get("type").unwrap_or(&false)
                    {
                        let script_type = output.tx_out.script_type;
                        csv_output.insert("address", output.tx_out.address);
                        csv_output.insert("type", script_type.clone());

                        // Update script type count
                        if let Some(count) = script_type_count.get_mut(&script_type) {
                            *count += 1;
                        }
                    }

                    if *fields_selected.get("script").unwrap_or(&false) {
                        csv_output.insert("script", hex::encode(output.tx_out.script_pubkey));
                    }

                    // Build CSV output
                    let mut csvline = Vec::new();
                    for field in args.fields.split(',') {
                        let field = field.trim();
                        csvline.push(csv_output.get(field).unwrap_or(&String::new()).clone());
                    }
                    let csvline = csvline.join(",");

                    count += 1;

                    // Print progress and results
                    if !args.quiet {
                        if args.verbose {
                            println!("{}", csvline);
                        } else if count > 0 && count % 100000 == 0 {
                            println!("{} utxos processed", count);
                        }
                    }

                    // Write to file
                    writeln!(writer, "{}", csvline)?;
                }
            }
        } else {
            break; // TODO: keys are sorted, so we are guaranteed to process all utxos. TO BE VERIFIED.
        }
    }

    // Flush the writer
    writer.flush()?;

    // Final progress report
    if !args.quiet {
        println!();
        println!("Total UTXOs: {}", count);

        if *fields_selected.get("amount").unwrap_or(&false) {
            println!("Total {}:   {:.8}", blockchain.ticker(), total_amount as f64 / 100_000_000.0);
        }

        if *fields_selected.get("type").unwrap_or(&false) {
            println!("Script Types:");
            for (script_type, count) in script_type_count {
                println!(" {:<12} {}", script_type, count);
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_macos_rlimit(args: &Args) -> Result<()> {
    if !args.quiet {
        println!("Setting rlimit to 4096");
    }

    let lim = rlimit {
        rlim_cur: 4_096, // soft limit
        rlim_max: 4_096, // hard limit
    };

    let ret = unsafe { setrlimit(RLIMIT_NOFILE, &lim) };
    if ret != 0 {
        eprintln!("Failed to set rlimit: {}", std::io::Error::last_os_error());
    } else {
        println!("Successfully updated RLIMIT_NOFILE to 4096");
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn set_macos_rlimit(_args: &Args) -> Result<()> {
    Ok(())
}

fn validate_and_parse_fields(fields_str: &str) -> Result<HashMap<String, bool>> {
    let fields_allowed = vec![
        "count", "txid", "vout", "height", "coinbase", "amount", "nsize", "script", "type",
        "address",
    ];

    let mut fields_selected = HashMap::new();
    for field in &fields_allowed {
        fields_selected.insert(field.to_string(), false);
    }

    for field in fields_str.split(',') {
        let field = field.trim();
        if !fields_allowed.contains(&field) {
            anyhow::bail!(
                // TODO: why anyhow bail?
                "'{}' is not a field you can use for the output.\nChoose from the following: {}",
                field,
                fields_allowed.join(",")
            );
        }
        fields_selected.insert(field.to_string(), true);
    }

    Ok(fields_selected)
}

/// Read raw varint bytes from data starting at offset, returning (varint_bytes, bytes_read)
fn read_varint<R: Read>(reader: &mut R) -> Result<u64, Error> {
    let mut result = 0u64; // TODO: can it be larger than u64?
    let mut shift = 0;

    loop {
        if shift >= 64 {
            return Err(Error::new(ErrorKind::InvalidData, "VARINT overflow"));
        }

        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        let b = byte[0];

        result |= ((b & 0x7F) as u64) << shift;

        if (b & 0x80) == 0 {
            break;
        }

        shift += 7;
    }

    Ok(result)

    // // Loop through bytes starting from offset
    // for (i, &byte) in data[offset..].iter().enumerate() {
    //     // Store each byte as we go
    //     result.push(byte);
    //
    //     // Bitwise AND with 128 (0b10000000) to check if the 8th bit is set
    //     let set = byte & 128; // 0b10000000 is same as 1 << 7
    //
    //     // When you get to one without the 8th bit set, return that byte slice
    //     if set == 0 {
    //         return (result.clone(), result.len());
    //     }
    // }

    // Return zero bytes read if we haven't managed to read bytes properly
    // (result, 0)
}

/// Decode a single varint from bytes using Bitcoin's encoding scheme, returning (value, bytes_read)
fn varint_decode(data: &[u8]) -> (i64, usize) {
    if data.is_empty() {
        return (0, 0);
    }

    let mut n = 0i64;
    let mut bytes_read = 0;

    for &byte in data {
        bytes_read += 1;

        // 1. Shift n left 7 bits (add some extra bits to work with)
        n = n << 7;

        // 2. Set the last 7 bits of each byte into the total value
        // AND extracts 7 bits only (ignore the 8th bit)
        n = n | (byte & 127) as i64;

        // 3. Add 1 each time (only for the ones where the 8th bit is set)
        if byte & 128 != 0 {
            // 0b10000000 <- AND to check if the 8th bit is set
            n += 1;
        }
    }

    (n, bytes_read)
}

/// Decompress amount value (Bitcoin Core compression)
fn decompress_amount(compressed: i64) -> i64 {
    let mut x = compressed;
    let mut n = 0; // decompressed value

    // Return value if it is zero (nothing to decompress)
    if x == 0 {
        return 0;
    }

    // Decompress...
    x = x - 1; // subtract 1 first
    let e = x % 10; // remainder mod 10
    x = x / 10; // quotient mod 10 (reduce x down by 10)

    // If the remainder is less than 9
    if e < 9 {
        let d = x % 9; // remainder mod 9
        x = x / 9; // (reduce x down by 9)
        n = x * 10 + d + 1; // work out n
    } else {
        n = x + 1;
    }

    // Multiply n by 10 to the power of the first remainder
    let result = (n as f64) * 10f64.powi(e as i32);

    result as i64
}

struct DBOutput {
    coinbase: bool,
    height: u32,
    tx_out: TxOut,
}

#[derive(Debug, Clone)]
struct TxOut {
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
    pub n_size: u64,
    script_type: String,
    address: String
}

fn deserialize_db_value(blockchain: &Blockchain, value: Vec<u8>) -> Vec<DBOutput> {
    // -----------------
    // Value for Bitcoin
    // -----------------

    //          c0842680ed5900a38f35518de4487c108e3810e6794fb68b189d8b <- deobfuscated value (XOR)
    //          <----><----><><-------------------------------------->
    //           /      |    \                   |
    //      varint   varint   varint          script <- P2PKH/P2SH hash160, P2PK public key, or complete script
    //         |        |     n_size
    //         |        |
    //         |     amount (compressed)
    //         |
    //         |
    //  100000100001010100110
    //  <------------------> \
    //         height         coinbase

    // First varint (height, coinbase)
    let (varint_bytes, bytes_read) = read_varint(&xor_result, offset);
    offset += bytes_read;
    let varint_decoded = varint_decode(&varint_bytes).0;

    if *fields_selected.get("height").unwrap_or(&false)
        || *fields_selected.get("coinbase").unwrap_or(&false)
    {
        let height = varint_decoded >> 1;
        let coinbase = varint_decoded & 1;

        output.insert("height", height.to_string());
        output.insert("coinbase", coinbase.to_string());
    }

    // Second varint (amount compressed)
    let (varint_bytes, bytes_read) = read_varint(&xor_result, offset);
    offset += bytes_read;
    let varint_decoded = varint_decode(&varint_bytes).0;

    if *fields_selected.get("amount").unwrap_or(&false) {
        let amount = decompress_amount(varint_decoded);
        output.insert("amount", amount.to_string());
        total_amount += amount;
    }

    // TODO today: run the script and check if there are with n_size 4 or 5.

    // Third varint (nsize)
    // n_size - byte to indicate the type or size of script

    //  0  = P2PKH <- hash160 PK
    //  1  = P2SH  <- hash160 script
    //  2  = P2PK 02publickey <- compressed PK, y=even - here and following: n_size makes up part of the PK in the actual script
    //  3  = P2PK 03publickey <- compressed PK, y=odd
    //  4  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=even
    //  5  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=odd
    //  6+ = [size of the upcoming script] (subtract 6 though to get the actual size in bytes, to account for the previous 5 script types already taken)
    let (varint_bytes, bytes_read) = read_varint(&xor_result, offset);
    offset += bytes_read;
    let nsize = varint_decode(&varint_bytes).0;

    if *fields_selected.get("nsize").unwrap_or(&false) {
        output.insert("nsize", nsize.to_string());
    }

    // Move offset back for P2PK types 2,3,4,5 since n_size makes up part of the public key
    if nsize > 1 && nsize < 6 {
        offset -= 1;
    }

    let script = &xor_result[offset..];
    let mut p2pk_script_bytes = Vec::new(); // For P2PK case

    // Address and script type processing
    if *fields_selected.get("address").unwrap_or(&false)
        || *fields_selected.get("type").unwrap_or(&false)
    {
        let mut address = String::new();
        let mut script_type = String::new();

        match nsize {
            0 => {
                address = blockchain.create_p2pkh_address(
                    PubkeyHash::from_byte_array(<[u8; 20]>::try_from(script)?),
                    testnet,
                )?;
                script_type = "p2pkh".to_string();
            }
            1 => {
                let parsed_script = Script::from_bytes(script);
                address = blockchain.create_p2sh_address(&parsed_script, testnet)?;
                script_type = "p2sh".to_string();
            }
            2 | 3 => {
                println!("size of script is {}", script.len());
                println!("2 | 3 script is {}", hex::encode(script));
                script_type = "p2pk".to_string();
                let pk = PublicKey::from_slice(&script)?;
                let script_buf = ScriptBuf::new_p2pk(&pk);
                p2pk_script_bytes = script_buf.into_bytes();
            }
            4 | 5 => {
                script_type = "p2pk".to_string();
                println!("4 | 5 script is {}", hex::encode(script));
                let prefix = script[0];
                let pk = if prefix == 0x04 {
                    let mut bytes = Vec::with_capacity(33);
                    bytes.push(0x02);
                    bytes.extend_from_slice(&script[1..]);
                    Secp256k1Pk::from_slice(&bytes).expect("valid public key")
                } else if prefix == 0x05 {
                    let mut bytes = Vec::with_capacity(33);
                    bytes.push(0x03);
                    bytes.extend_from_slice(&script[1..]);
                    Secp256k1Pk::from_slice(&bytes).expect("valid public key")
                } else {
                    panic!("unexpected prefix: {prefix}");
                };

                let uncompressed = pk.serialize_uncompressed();
                let pk = PublicKey::from_slice(&uncompressed)?;
                let script_buf = ScriptBuf::new_p2pk(&pk);
                p2pk_script_bytes = script_buf.into_bytes();
            }
            _ => {
                (address, script_type) = process_non_standard_script_and_address(
                    script,
                    nsize,
                    testnet,
                    args.p2pk_addresses,
                    &fields_selected,
                );
            }
        };

        output.insert("address", address);
        output.insert("type", script_type.clone());

        // Update script type count
        if let Some(count) = script_type_count.get_mut(script_type.as_str()) {
            *count += 1;
        }
    }

    if *fields_selected.get("script").unwrap_or(&false) {
        let script_to_encode = if nsize == 2 || nsize == 3 {
            &p2pk_script_bytes
        } else {
            script
        };
        output.insert("script", hex::encode(script_to_encode));
    }
}

fn deserialize_db_value_legacy(blockchain: &Blockchain, value: Vec<u8>) -> Vec<DBOutput> {
    let mut cursor = Cursor::new(value);

    // ------------------
    // Value for Dogecoin
    // ------------------

    // Ref: <https://en.bitcoin.it/wiki/Bitcoin_Core_0.11_(ch_2):_Data_Storage>
    //      <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L74>
    //      <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L156>

    //   0109044086ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4eebbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa486af3b <- deobfuscated value (XOR)
    //   <><><--><--------------------------------------------------><----------------------------------------------><---->
    //   |  |  |                          |                                                  |                          |
    //   |  | mask                      vout[4]                                           vout[14]                   height
    //   | code
    // version

    // First varint (version)
    let _version = read_varint(&mut cursor);

    // Second varint (code)
    let code = read_varint(&mut cursor).unwrap();

    let coinbase = (code & 1) != 0;

    let vout0_unspent = (code & 2) != 0;
    let vout1_unspent = (code & 4) != 0;

    let mask_bytes = if vout0_unspent || vout1_unspent {
        (code / 8) as u32
    } else {
        ((code / 8) + 1) as u32
    };

    let mut unspent_outputs = vec![false; 2];
    unspent_outputs[0] = vout0_unspent;
    unspent_outputs[1] = vout1_unspent;

    if mask_bytes > 0 {
        let max_outputs_from_mask = (mask_bytes * 8) as usize;
        let mut additional_unspent_outputs = vec![false; max_outputs_from_mask];

        let mut mask_data = vec![0u8; mask_bytes as usize];
        cursor.read_exact(&mut mask_data)?;

        for byte_idx in 0..mask_bytes {
            let byte_val = mask_data[byte_idx as usize];

            for bit_idx in 0..8 {
                let output_idx = ((byte_idx * 8) + bit_idx) as usize;
                if output_idx < max_outputs_from_mask {
                    additional_unspent_outputs[output_idx] = (byte_val & (1 << bit_idx)) != 0;
                }
            }
        }

        let actual_size = additional_unspent_outputs.iter().rposition(|&x| x)
            .map(|pos| pos + 1)
            .unwrap_or(0);
        additional_unspent_outputs.truncate(actual_size); // TODO: check if this is necessary

        unspent_outputs.extend(additional_unspent_outputs);
    }

    let mut outputs = vec![None; unspent_outputs.len()];

    for (i, &is_unspent) in unspent_outputs.iter().enumerate() {
        if is_unspent {
            let txout = deserialize_txout(&mut cursor, &blockchain)?;
            outputs[i] = Some(txout);
        }
    }

    let height = read_varint(&mut cursor)? as u32;

    let mut db_outputs = vec![];

    for output in outputs {
        let db_output = DBOutput {
            coinbase,
            height
        };
        db_outputs.push(db_output);
    }

    db_outputs
}

fn deserialize_txout<R: Read>(reader: &mut R, blockchain: &Blockchain) -> Result<TxOut, Error> {
    let compressed_amount = read_varint(reader)?;
    let amount = decompress_amount(compressed_amount);

    // n_size - byte to indicate the type or size of script

    //  0  = P2PKH <- hash160 PK
    //  1  = P2SH  <- hash160 script
    //  2  = P2PK 02publickey <- compressed PK, y=even - here and following: n_size makes up part of the PK in the actual script
    //  3  = P2PK 03publickey <- compressed PK, y=odd
    //  4  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=even
    //  5  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=odd
    //  6+ = [size of the upcoming script] (subtract 6 though to get the actual size in bytes, to account for the previous 5 script types already taken)
    let nsize = read_varint(reader)?;

    let mut address = String::new();
    let mut script_type = String::new();

    if nsize < 6 {
        // Compressed script: nsize is the compression type (0-5)
        let data_size = match nsize {
            0 | 1 => 20,        // P2PKH, P2SH: 20 bytes
            2 | 3 | 4 | 5 => 32, // Pubkey variants: 32 bytes
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid compression type")),
        };

        // Read the compressed data
        let mut compressed_data = vec![0u8; data_size];
        reader.read_exact(&mut compressed_data)?;

        // Decompress based on type
        match nsize {
            0 => {
                address = blockchain.create_p2pkh_address(
                    PubkeyHash::from_byte_array(<[u8; 20]>::try_from(&compressed_data)?),
                )?;
                script_type = "p2pkh".to_string();
            },
            1 => {
                let parsed_script = Script::from_bytes(&compressed_data);
                address = blockchain.create_p2sh_address(&parsed_script)?;
                script_type = "p2sh".to_string();
            },
            prefix @ (2 | 3) => {
                let mut public_key = vec![prefix]; // TODO: this could be cleaner
                public_key.extend_from_slice(compressed_data.as_ref());

                let pk = PublicKey::from_slice(public_key.as_ref())?;
                let script_buf = ScriptBuf::new_p2pk(&pk);
                p2pk_script_bytes = script_buf.into_bytes();
                script_type = "p2pk".to_string();
            },
            prefix @ (4 | 5) => {
                let pk = if prefix == 0x04 {
                    let mut bytes = Vec::with_capacity(33);
                    bytes.push(0x02);
                    bytes.extend_from_slice(&compressed_data[1..]);
                    Secp256k1Pk::from_slice(&bytes).expect("valid public key")
                } else if prefix == 0x05 {
                    let mut bytes = Vec::with_capacity(33);
                    bytes.push(0x03);
                    bytes.extend_from_slice(&compressed_data[1..]);
                    Secp256k1Pk::from_slice(&bytes).expect("valid public key")
                } else {
                    panic!("unexpected prefix: {prefix}");
                };

                let uncompressed = pk.serialize_uncompressed();
                let pk = PublicKey::from_slice(&uncompressed)?;
                let script_buf = ScriptBuf::new_p2pk(&pk);
                p2pk_script_bytes = script_buf.into_bytes();
                script_type = "p2pk".to_string();
            },
            _ => unreachable!(),
        }
    } else {
        // Regular script: nsize = actual_size + 6
        let actual_size = nsize - 6;
        if script.len() >= 37 && script.last() == Some(&174) { // 174 = 0xae = OP_CHECKMULTISIG
            script_type = "p2ms".to_string();
        } else {
            script_type = "non-standard".to_string();
        }

        let mut script_data = vec![0u8; actual_size as usize];
        reader.read_exact(&mut script_data)?;
        script_data
    };

    TxOut {
        amount,
        n_size: nSize,
        script_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_parsing() {
        // Test small values
        let data = [0x01];
        assert_eq!(varint_decode(&data), (1, 1));

        // Test larger value
        let data = [0xFD, 0x02];
        assert_eq!(varint_decode(&data), (509, 2));
    }

    #[test]
    fn test_amount_decompression() {
        assert_eq!(decompress_amount(0x0), 0);
        assert_eq!(decompress_amount(0x1), 1);
        assert_eq!(decompress_amount(0x7), 1000000); // 1 cent
        assert_eq!(decompress_amount(0x9), 100000000); // 1 BTC
    }

    #[test]
    fn test_header_code_decoding() {
        let (coinbase, vout0, vout1, mask_bytes) = decode_header_code(4);
        assert_eq!(coinbase, false);
        assert_eq!(vout0, false);
        assert_eq!(vout1, true);
        assert_eq!(mask_bytes, 0);

        let (coinbase, vout0, vout1, mask_bytes) = decode_header_code(9);
        assert_eq!(coinbase, true);
        assert_eq!(vout0, false);
        assert_eq!(vout1, false);
        assert_eq!(mask_bytes, 2);
    }

    #[test]
    fn test_read_dogecoin() {
        /**
        * Pruned version of CTransaction: only retains metadata and unspent transaction outputs
        *
        * Serialized format:
        * - VARINT(nVersion)
        * - VARINT(nCode)
        * - unspentness bitvector, for vout[2] and further; least significant byte first
        * - the non-spent CTxOuts (via CTxOutCompressor)
        * - VARINT(nHeight)
        *
        * The nCode value consists of:
        * - bit 0: IsCoinBase()
        * - bit 1: vout[0] is not spent
        * - bit 2: vout[1] is not spent
        * - The higher bits encode N, the number of non-zero bytes in the following bitvector.
        *   - In case both bit 1 and bit 2 are unset, they encode N-1, as there must be at
        *     least one non-spent output).
        *
        * Example: 0104835800816115944e077fe7c803cfa57f29b36bf87c1d358bb85e
        *          <><><--------------------------------------------><---->
        *          |  \                  |                             /
        *    version   code             vout[1]                  height
        *
        *    - version = 1
        *    - code = 4 (vout[1] is not spent, and 0 non-zero bytes of bitvector follow)
        *    - unspentness bitvector: as 0 non-zero bytes follow, it has length 0
        *    - vout[1]: 835800816115944e077fe7c803cfa57f29b36bf87c1d35
        *               * 8358: compact amount representation for 60000000000 (600 BTC)
        *               * 00: special txout type pay-to-pubkey-hash
        *               * 816115944e077fe7c803cfa57f29b36bf87c1d35: address uint160
        *    - height = 203998
        *
        *
        * Example: 0109044086ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4eebbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa486af3b
        *          <><><--><--------------------------------------------------><----------------------------------------------><---->
        *         /  \   \                     |                                                           |                     /
        *  version  code  unspentness       vout[4]                                                     vout[16]           height
        *
        *  - version = 1
        *  - code = 9 (coinbase, neither vout[0] or vout[1] are unspent,
        *                2 (1, +1 because both bit 1 and bit 2 are unset) non-zero bitvector bytes follow)
        *  - unspentness bitvector: bits 2 (0x04) and 14 (0x4000) are set, so vout[2+2] and vout[14+2] are unspent
        *  - vout[4]: 86ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4ee
        *             * 86ef97d579: compact amount representation for 234925952 (2.35 BTC)
        *             * 00: special txout type pay-to-pubkey-hash
        *             * 61b01caab50f1b8e9c50a5057eb43c2d9563a4ee: address uint160
        *  - vout[16]: bbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa4
        *              * bbd123: compact amount representation for 110397 (0.001 BTC)
        *              * 00: special txout type pay-to-pubkey-hash
        *              * 8c988f1a4a4de2161e0f50aac7f17e7f9555caa4: address uint160
        *  - height = 120891
        */

        let hex_data = "0104835800816115944e077fe7c803cfa57f29b36bf87c1d358bb85e";
    }
}