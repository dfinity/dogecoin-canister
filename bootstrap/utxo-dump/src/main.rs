use bitcoin::{Address, Network, PubkeyHash, PublicKey, ScriptBuf};
use secp256k1::{PublicKey as Secp256k1Pk, Secp256k1};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::{exit, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use bitcoin::hashes::Hash;
use bitcoin::Script;
use clap::Parser;
use rusty_leveldb::{LdbIterator, Options, DB};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

const VERSION: &str = "1.0.0";

#[derive(Parser)]
#[command(name = "utxo-dump")]
#[command(about = "Dumps Bitcoin UTXO set from chainstate LevelDB to CSV")]
#[command(version = VERSION)]
struct Args {
    /// Location of bitcoin chainstate db
    #[arg(short = 'd', long = "db", default_value_t = get_default_chainstate())]
    chainstate: String,

    /// Name of file to dump utxo list to
    #[arg(short = 'o', default_value = "utxodump.csv")]
    output_file: String,

    /// Fields to include in output
    #[arg(
        short = 'f',
        long = "fields",
        default_value = "count,txid,vout,amount,type,address"
    )]
    fields: String,

    /// Is the chainstate leveldb for testnet?
    #[arg(long = "testnet")]
    testnet: bool,

    /// Print utxos as we process them (will be about ??? times slower)
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Convert public keys in P2PK locking scripts to addresses also
    #[arg(long = "p2pkaddresses")]
    p2pk_addresses: bool,

    /// Ignore warnings if dogecoind is running in the background
    #[arg(long = "nowarnings")]
    no_warnings: bool,

    /// Do not display any progress or results
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}

fn get_default_chainstate() -> String {
    if let Ok(home) = env::var("HOME") {
        format!("{}/.dogecoin/chainstate/", home)
    } else {
        ".dogecoin/chainstate/".to_string()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check if Dogecoin is running first (unless nowarnings is set)
    // if !args.no_warnings {
    //     check_dogecoin_running(&args)?;
    // }

    // Check if OS is macOS and set ulimit
    // #[cfg(target_os = "macos")]
    // set_macos_ulimit(&args)?;

    // Determine if we're using testnet
    let testnet = args.testnet || args.chainstate.contains("testnet");

    // Check chainstate LevelDB folder exists
    if !Path::new(&args.chainstate).exists() {
        anyhow::bail!("Couldn't find {}", args.chainstate);
    }

    // Validate and parse output fields
    let fields_selected = validate_and_parse_fields(&args.fields)?;

    // Open LevelDB database
    let options = Options::default();
    // Disable compression to avoid corrupting the database

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

    // Iterate over LevelDB
    let mut iter = database.new_iter()?;
    let mut count = 0;

    iter.reset();
    let (key, value) = iter.next().unwrap();
    let prefix = key[0];

    let mut obfuscate_key: Vec<u8> = Vec::new();

    // Key of obfuscate key: "0e00obfuscate_key"
    if prefix == 14 {
        obfuscate_key = value.clone();
        println!(">>> Obfuscation key: {}", hex::encode(&obfuscate_key));
    }

    let mut count_iter = 0;

    // Main iteration loop
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

            count_iter += 1;

            // Stop after seeing first 5 entries to understand the database structure
            // if count_iter <= 500 {
            //     println!("Entry {}: prefix={} (0x{:02x})", count_iter, prefix, prefix);
            //     println!("  Key:   {}", hex::encode(&key));
            //     println!("  Value: {}", hex::encode(&value));
            // }

            // UTXO entry
            // Bitcoin:  67 = 0x43 = C = "utxo"
            // Dogecoin: 99 = 0x63 = c = "utxo"
            if prefix == 67 {
                // ---
                // Key
                // ---
                //
                //      63fff73e569fa1ee147fe0f55a7a7c002d2a40519605300e39783611a0216dfa5d
                //      <><--------------------------------------------------------------><>
                //      /                               |                                  \
                //  prefix                      txid (little-endian)                 vout (varint)

                let mut output = HashMap::new();

                // txid
                if *fields_selected.get("txid").unwrap_or(&false) {
                    let txid_le = &key[1..33];

                    // Reverse byte order (little-endian to big-endian)
                    let mut txid = Vec::new();
                    for i in (0..txid_le.len()).rev() {
                        txid.push(txid_le[i]);
                    }
                    output.insert("txid", hex::encode(txid));
                }

                // vout
                if *fields_selected.get("vout").unwrap_or(&false) {
                    let index_bytes = &key[33..];
                    let vout = varint_decode(index_bytes).0;
                    output.insert("vout", vout.to_string());
                }

                let needs_value_processing = *fields_selected.get("type").unwrap_or(&false)
                    || *fields_selected.get("height").unwrap_or(&false)
                    || *fields_selected.get("coinbase").unwrap_or(&false)
                    || *fields_selected.get("amount").unwrap_or(&false)
                    || *fields_selected.get("nsize").unwrap_or(&false)
                    || *fields_selected.get("script").unwrap_or(&false)
                    || *fields_selected.get("address").unwrap_or(&false);

                if needs_value_processing {
                    if obfuscate_key.is_empty() {
                        eprintln!("Error: No obfuscation key found in chainstate database.");
                        eprintln!("This database may be corrupted or from an older Bitcoin version that doesn't use obfuscation.");
                        eprintln!("Cannot process UTXO values without the obfuscation key.");
                        exit(1);
                    }

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

                    // if count_iter <= 500 {
                    //     println!("  Deobfuscated Value: {}", hex::encode(&xor_result));
                    // }

                    let mut offset = 0;

                    // -----
                    // Value
                    // -----

                    //          71a9e87d62de25953e189f706bcf59263f15de1bf6c893bda9b045 <- value
                    //          b12dcefd8f872536b12dcefd8f872536b12dcefd8f872536b12dce <- obfuscate_key_extended
                    //          c0842680ed5900a38f35518de4487c108e3810e6794fb68b189d8b <- deobfuscated value (XOR)
                    //          <----><----><><-------------------------------------->
                    //           /      |    \                   |
                    //      varint   varint   varint          script <- P2PKH/P2SH hash160, P2PK public key, or complete script
                    //         |        |     nSize
                    //         |        |
                    //         |     amount (compressed)
                    //         |
                    //         |
                    //  100000100001010100110
                    //  <------------------> \
                    //         height         coinbase

                    // First varint (height, coinbase)
                    let (varint_bytes, bytes_read) = varint_read(&xor_result, offset);
                    offset += bytes_read;
                    let varint_decoded = varint_decode(&varint_bytes).0;

                    // if count_iter <= 500 {
                    //     println!("  Varint: {}", varint_decoded);
                    // }

                    if *fields_selected.get("height").unwrap_or(&false)
                        || *fields_selected.get("coinbase").unwrap_or(&false)
                    {
                        let height = varint_decoded >> 1;
                        let coinbase = varint_decoded & 1;

                        output.insert("height", height.to_string());
                        output.insert("coinbase", coinbase.to_string());
                    }

                    // Second varint (amount compressed)
                    let (varint_bytes, bytes_read) = varint_read(&xor_result, offset);
                    offset += bytes_read;
                    let varint_decoded = varint_decode(&varint_bytes).0;

                    if *fields_selected.get("amount").unwrap_or(&false) {
                        let amount = decompress_value(varint_decoded);
                        output.insert("amount", amount.to_string());
                        total_amount += amount;
                    }

                    // if count_iter <= 500 {
                    //     println!("  Amount: {}", decompress_value(varint_decoded));
                    // }

                    // TODO today: run the script and check if there are with nSize 4 or 5.

                    // Third varint (nsize)
                    // nSize - byte to indicate the type or size of script

                    //  0  = P2PKH <- hash160 PK
                    //  1  = P2SH  <- hash160 script
                    //  2  = P2PK 02publickey <- compressed PK, y=even - here and following: nSize makes up part of the PK in the actual script
                    //  3  = P2PK 03publickey <- compressed PK, y=odd
                    //  4  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=even
                    //  5  = P2PK 04publickey <- uncompressed PK (but has been turned into compressed PK in level DB), y=odd
                    //  6+ = [size of the upcoming script] (subtract 6 though to get the actual size in bytes, to account for the previous 5 script types already taken)
                    let (varint_bytes, bytes_read) = varint_read(&xor_result, offset);
                    offset += bytes_read;
                    let nsize = varint_decode(&varint_bytes).0;

                    if *fields_selected.get("nsize").unwrap_or(&false) {
                        output.insert("nsize", nsize.to_string());
                    }

                    if nsize != 4 && nsize != 5 {
                        println!("Entry {}: prefix={} (0x{:02x})", count_iter, prefix, prefix);
                        println!("  Key:   {}", hex::encode(&key));
                        println!("  Value: {}", hex::encode(&value));
                        println!("  Deobfuscated Value: {}", hex::encode(&xor_result));
                        println!("  nSize: {}", nsize);
                    }

                    // Move offset back for P2PK types 2,3,4,5 since nSize makes up part of the public key
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

                        let network = if !testnet {
                            Network::Bitcoin
                        } else {
                            Network::Testnet
                        };

                        match nsize {
                            0 => {
                                address = Address::p2pkh(
                                    PubkeyHash::from_byte_array(<[u8; 20]>::try_from(script)?),
                                    network,
                                )
                                .to_string();
                                script_type = "p2pkh".to_string();
                            }
                            1 => {
                                let parsed_script = Script::from_bytes(script);
                                address = Address::p2sh(&parsed_script, network)?.to_string();
                                script_type = "p2sh".to_string();
                            }
                            2 | 3 => {
                                println!("size of script is {}", script.len());
                                script_type = "p2pk".to_string();
                                let pk = PublicKey::from_slice(&script)?;
                                let script_buf = ScriptBuf::new_p2pk(&pk);
                                p2pk_script_bytes = script_buf.into_bytes();
                            }
                            4 | 5 => {
                                script_type = "p2pk".to_string();
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

                count += 1;
                output.insert("count", count.to_string());

                // Build CSV output
                let mut csvline = Vec::new();
                for field in args.fields.split(',') {
                    let field = field.trim();
                    csvline.push(output.get(field).unwrap_or(&String::new()).clone());
                }
                let csvline = csvline.join(",");

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
        } else {
            break;
        }
    }

    // Flush the writer
    writer.flush()?;

    // Final progress report
    if !args.quiet {
        println!();
        println!("Total UTXOs: {}", count);

        if *fields_selected.get("amount").unwrap_or(&false) {
            println!("Total DOGE:   {:.8}", total_amount as f64 / 100_000_000.0);
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

// fn check_dogecoin_running(_args: &Args) -> Result<()> {
//     let output = Command::new("dogecoin-cli").arg("getnetworkinfo").output();
//
//     if output.is_ok() {
//         println!("Dogecoin is running. You should shut it down with `dogecoin-cli stop` first.");
//         println!("We don't want to access the chainstate LevelDB while Dogecoin is running.");
//         println!("Note: If you do stop dogecoind, make sure that it won't auto-restart (e.g. if it's running as a systemd service).");
//
//         print!("Do you wish to continue anyway? [y/N] (default N): ");
//         io::stdout().flush()?;
//
//         let mut response = String::new();
//         io::stdin().read_line(&mut response)?;
//         let response = response.trim().to_lowercase();
//
//         if response != "y" && response != "yes" {
//             exit(0);
//         }
//     }
//
//     Ok(())
// }

// #[cfg(target_os = "macos")]
// fn set_macos_ulimit(args: &Args) -> Result<()> {
//     if !args.quiet {
//         println!("Setting ulimit to 4096");
//     }
//
//     let _output = Command::new("sh").arg("-c").arg("ulimit -n 4096").output();
//
//     // Note: This doesn't actually change the ulimit for the current process
//     // In a real implementation, you'd need to use libc::setrlimit
//
//     Ok(())
// }

#[cfg(not(target_os = "macos"))]
fn set_macos_ulimit(_args: &Args) -> Result<()> {
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
fn varint_read(data: &[u8], offset: usize) -> (Vec<u8>, usize) {
    let mut result = Vec::new();

    // Loop through bytes starting from offset
    for (i, &byte) in data[offset..].iter().enumerate() {
        // Store each byte as we go
        result.push(byte);

        // Bitwise AND with 128 (0b10000000) to check if the 8th bit is set
        let set = byte & 128; // 0b10000000 is same as 1 << 7

        // When you get to one without the 8th bit set, return that byte slice
        if set == 0 {
            return (result.clone(), result.len());
        }
    }

    // Return zero bytes read if we haven't managed to read bytes properly
    (result, 0)
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
fn decompress_value(compressed: i64) -> i64 {
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

/// Process script data to determine address and script type
fn process_non_standard_script_and_address(
    script: &[u8],
    nsize: i64,
    testnet: bool,
    p2pk_addresses: bool,
    fields_selected: &HashMap<String, bool>,
) -> (String, String) {
    let mut address = String::new();
    // let script_type = match nsize {
    // 2..=5 => {
    //     // P2PK
    //     if *fields_selected.get("address").unwrap_or(&false)
    //         && p2pk_addresses
    //         && !script.is_empty()
    //     {
    //         let prefix = if testnet { 0x6f } else { 0x00 };
    //         address = public_key_to_address(script, prefix);
    //     }
    //     "p2pk".to_string()
    // }
    // };

    // Check for P2MS (multisig)
    let script_type = if script.len() >= 37 && script.last() == Some(&174) {
        // 174 = 0xae = OP_CHECKMULTISIG
        "p2ms".to_string()
    } else {
        "non-standard".to_string()
    };

    (address, script_type)
}
