mod serialization;
mod blockchain;
mod utxo;
mod utils;

use bitcoin::{Network as BtcNetwork, dogecoin::Network as DogeNetwork};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor, Write};


use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use blockchain::Blockchain;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use rusty_leveldb::{LdbIterator, Options, DB};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use crate::serialization::read_varint;
#[cfg(target_os = "macos")]
use crate::utils::set_macos_rlimit;

const VERSION: &str = "1.0.0";

const DB_KEYS_OBFUSCATE_KEY_PREFIX: [u8;2] = [0x0e, 0x00];
const DB_KEYS_OBFUSCATE_KEY: &str = "obfuscate_key";

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BlockchainKind {
    Bitcoin,
    Dogecoin,
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
    #[arg(short = 'o', long = "output", default_value = "utxodump.csv")]
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
    blockchain: BlockchainKind,

    /// Is the chainstate leveldb for testnet?
    #[arg(long = "testnet")]
    testnet: bool,

    /// Print utxos as we process them
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Convert public keys in P2PK locking scripts to addresses
    #[arg(long = "p2pkaddresses")]
    p2pk_addresses: bool,

    /// Do not display any progress or results
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
}


impl Args {
    fn to_blockchain(&self) -> Blockchain {
        match self.blockchain {
            BlockchainKind::Bitcoin => {
                if self.testnet || self.chainstate.contains("testnet") {
                    Blockchain::Bitcoin(BtcNetwork::Testnet)
                } else {
                    Blockchain::Bitcoin(BtcNetwork::Bitcoin)
                }
            }
            BlockchainKind::Dogecoin => {
                if self.testnet || self.chainstate.contains("testnet") {
                    Blockchain::Dogecoin(DogeNetwork::Testnet)
                } else {
                    Blockchain::Dogecoin(DogeNetwork::Dogecoin)
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Check if OS is macOS and set rlimit
    #[cfg(target_os = "macos")]
    set_macos_rlimit(&args)?;

    if !Path::new(&args.chainstate).exists() {
        anyhow::bail!("Couldn't find {}", args.chainstate);
    }

    let blockchain = args.to_blockchain();

    let fields_selected = validate_and_parse_fields(&args.fields)?;

    let options = Options::default();
    let mut database = DB::open(&args.chainstate, options).context("Couldn't open LevelDB")?;

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
    let mut total_amount: u64 = 0;
    let mut utxo_count = 0;
    let mut script_type_count: HashMap<&str, u32> = HashMap::new();
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

    let mut db_iter = database.new_iter()?;

    // Read first value: obfuscation key
    db_iter.reset();
    let (key, value) = db_iter.next().unwrap();
    let (prefix, key) = key.split_at(2);
    let key_str= String::from_utf8(key.to_vec()).expect("valid UTF-8");
    let mut obfuscate_key: Vec<u8> = Vec::new();
    if prefix == DB_KEYS_OBFUSCATE_KEY_PREFIX && key_str == DB_KEYS_OBFUSCATE_KEY {
        obfuscate_key = value;
        if !args.quiet {
            println!(">>> Obfuscation key: {}", hex::encode(&obfuscate_key));
        }
    }
    if obfuscate_key.is_empty() {
        anyhow::bail!(
            "No obfuscation key found in chainstate database.\n\
             This database may be corrupted or from an older Bitcoin version that doesn't use obfuscation.\n\
             Cannot process UTXO values without the obfuscation key."
        );
    }

    db_iter.seek_to_first();
    while db_iter.valid() {
        if !running.load(Ordering::SeqCst) {
            if !args.quiet {
                println!("Interrupt signal caught. Shutting down gracefully.");
            }
            break;
        }

        if let Some((key, value)) = db_iter.next() {
            let prefix = key[0];
            if prefix == blockchain.utxo_key_prefix() {
                // --------------------
                // Key for Bitcoin UTXO
                // --------------------

                //      430000155b9869d56c66d9e86e3c01de38e3892a42b99949fe109ac034fff6583900
                //      <><--------------------------------------------------------------><>
                //      /                               |                                  \
                //  prefix                      txid (little-endian)                 vout (varint)

                // ---------------------
                // Key for Dogecoin UTXO
                // ---------------------

                // Ref: <https://en.bitcoin.it/wiki/Bitcoin_Core_0.11_(ch_2):_Data_Storage>

                //      6300007c264b7c8b886b0c7d086d7b42e1737d6c3fb94b85eaecda45be1eb3b6a0
                //      <><-------------------------------------------------------------->
                //      /                               |
                //  prefix                      txid (little-endian)

                // Ignore first byte (size of the obfuscate key)
                let mut obfuscate_key_extended = obfuscate_key[1..].to_vec(); // TODO: 1) remove size when reading key, 2) put size value in variable, 3) try to simplify code for extending key.

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
                let mut deobfuscated_value = Vec::new();
                for i in 0..value.len() {
                    deobfuscated_value.push(value[i] ^ obfuscate_key_extended[i]);
                }

                let mut csv_output = HashMap::new();

                let outputs = blockchain.deserialize_db_value(deobfuscated_value)?;

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
                        if key.len() >= 34 { // Modern: vout is encoded in the key
                            let vout_bytes = &key[33..];
                            let mut cursor = Cursor::new(vout_bytes);
                            let vout = read_varint(&mut cursor)?;
                            csv_output.insert("vout", vout.to_string());
                        } else if key.len() == 33 { // Legacy: vout is encoded in the value
                            csv_output.insert("vout", output.vout.to_string());
                        } else {
                            anyhow::bail!("Invalid key length: {}", key.len());
                        }
                    }

                    // coinbase
                    if *fields_selected.get("coinbase").unwrap_or(&false) || *fields_selected.get("height").unwrap_or(&false) {
                        csv_output.insert("coinbase", output.coinbase.to_string());
                        csv_output.insert("height", output.height.to_string());
                    }

                    // amount
                    if *fields_selected.get("amount").unwrap_or(&false) {
                        let amount = output.txout.amount;
                        csv_output.insert("amount", amount.to_string());
                        total_amount += amount;
                    }

                    // nsize
                    if *fields_selected.get("nsize").unwrap_or(&false) {
                        csv_output.insert("nsize", output.txout.nsize.to_string());
                    }

                    // Address and script type processing
                    if *fields_selected.get("address").unwrap_or(&false)
                        || *fields_selected.get("type").unwrap_or(&false)
                    {
                        let script_type = output.txout.script_type;
                        // Update script type statistics
                        if let Some(count) = script_type_count.get_mut(script_type.as_str()) {
                            *count += 1;
                        }
                        csv_output.insert("address", output.txout.address);
                        csv_output.insert("type", script_type);
                    }

                    if *fields_selected.get("script").unwrap_or(&false) {
                        csv_output.insert("script", hex::encode(output.txout.script));
                    }

                    // Build CSV output
                    let mut csvline = Vec::new();
                    for field in args.fields.split(',') {
                        let field = field.trim();
                        csvline.push(csv_output.get(field).unwrap_or(&String::new()).clone());
                    }
                    let csvline = csvline.join(",");
                    writeln!(writer, "{}", csvline)?;

                    utxo_count += 1;
                    if !args.quiet {
                        if args.verbose {
                            println!("{}", csvline);
                        } else if utxo_count > 0 && utxo_count % 100000 == 0 {
                            println!("{} utxos processed", utxo_count);
                        }
                    }
                }
            }
        } else {
            break; // TODO: keys are sorted, so we are guaranteed to process all utxos. TO BE VERIFIED.
        }
    }
    writer.flush()?;

    if !args.quiet {
        println!();
        println!("Total UTXOs: {}", utxo_count);

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
                "'{}' is not a field you can use for the output.\nChoose from the following: {}",
                field,
                fields_allowed.join(",")
            );
        }
        fields_selected.insert(field.to_string(), true);
    }

    Ok(fields_selected)
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use bitcoin::PubkeyHash;
    use crate::utxo::deserialize_db_utxo_legacy;
    use super::*;

    #[test]
    fn test_deserialize_db_utxo_legacy() {
        /*
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
        * Ref: <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L40>
        */

        let blockchain = Blockchain::Bitcoin(BtcNetwork::Bitcoin);
        let hex_data = "0104835800816115944e077fe7c803cfa57f29b36bf87c1d358bb85e";
        let outputs = deserialize_db_utxo_legacy(&blockchain, hex::decode(hex_data).unwrap()).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].vout, 1);
        assert_eq!(outputs[0].coinbase, false);
        assert_eq!(outputs[0].txout.amount, 60000000000);
        assert_eq!(outputs[0].txout.script_type, "p2pkh");
        let pubkey_hash = PubkeyHash::from_str("816115944e077fe7c803cfa57f29b36bf87c1d35").unwrap();
        assert_eq!(outputs[0].txout.address, blockchain.p2pkh_address(pubkey_hash));
        assert_eq!(outputs[0].txout.nsize, 0);
        assert_eq!(outputs[0].height, 203998);

        /*
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
        *
        * Ref: <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L55>
        */

        let blockchain = Blockchain::Bitcoin(BtcNetwork::Bitcoin);
        let hex_data = "0109044086ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4eebbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa486af3b";
        let outputs = deserialize_db_utxo_legacy(&blockchain, hex::decode(hex_data).unwrap()).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].vout, 4);
        assert_eq!(outputs[0].coinbase, true);
        assert_eq!(outputs[0].txout.amount, 234925952);
        assert_eq!(outputs[0].txout.script_type, "p2pkh");
        let pubkey_hash_0 = PubkeyHash::from_str("61b01caab50f1b8e9c50a5057eb43c2d9563a4ee").unwrap();
        assert_eq!(outputs[0].txout.address, blockchain.p2pkh_address(pubkey_hash_0));
        assert_eq!(outputs[0].txout.nsize, 0);
        assert_eq!(outputs[0].height, 120891);
        assert_eq!(outputs[1].vout, 16);
        assert_eq!(outputs[1].coinbase, true);
        assert_eq!(outputs[1].txout.amount, 110397);
        assert_eq!(outputs[1].txout.script_type, "p2pkh");
        let pubkey_hash_1 = PubkeyHash::from_str("8c988f1a4a4de2161e0f50aac7f17e7f9555caa4").unwrap();
        assert_eq!(outputs[1].txout.address, blockchain.p2pkh_address(pubkey_hash_1));
        assert_eq!(outputs[1].txout.nsize, 0);
        assert_eq!(outputs[1].height, 120891);
    }
}