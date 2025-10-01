mod blockchain;
mod chainstate;
mod serialization;
#[cfg(target_os = "macos")]
mod utils;

use bitcoin::{dogecoin::Network as DogeNetwork, Network as BtcNetwork};
use std::collections::{BTreeMap, HashMap};
use std::io::{BufWriter, Cursor, Write};

use blockchain::Blockchain;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::serialization::read_varint;
#[cfg(target_os = "macos")]
use crate::utils::set_macos_rlimit;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use rusty_leveldb::{LdbIterator, Options, DB};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

const VERSION: &str = "1.0.0";
const FIELDS_ALLOWED: [&str; 10] = [
    "count", "txid", "vout", "height", "coinbase", "amount", "nsize", "script", "type", "address",
];

const DB_KEYS_OBFUSCATE_KEY_PREFIX: [u8; 2] = [0x0e, 0x00];
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
    #[arg(short = 'd', long = "db", value_hint = clap::ValueHint::DirPath)]
    chainstate: PathBuf,

    /// Name of file to dump utxo list to
    #[arg(short = 'o', long = "output", default_value = "chainstate_utxos.csv")]
    output_file: String,

    /// Fields to include in the output
    #[arg(
        short = 'f',
        long = "fields",
        default_value = "height,txid,vout,amount,type,address,script,coinbase,nsize"
    )]
    fields: String,

    /// Blockchain (bitcoin, dogecoin)
    #[arg(short = 'b', long = "blockchain")]
    blockchain: BlockchainKind,

    /// Is the chainstate leveldb for testnet?
    #[arg(long = "testnet")]
    testnet: bool,

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
                if self.testnet || self.chainstate.display().to_string().contains("testnet") {
                    Blockchain::Bitcoin(BtcNetwork::Testnet)
                } else {
                    Blockchain::Bitcoin(BtcNetwork::Bitcoin)
                }
            }
            BlockchainKind::Dogecoin => {
                if self.testnet || self.chainstate.display().to_string().contains("testnet") {
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

    #[cfg(target_os = "macos")]
    set_macos_rlimit(&args)?;

    if !Path::new(&args.chainstate).exists() {
        anyhow::bail!("Couldn't find {}", args.chainstate.display());
    }

    let blockchain = args.to_blockchain();

    let fields_selected = validate_and_parse_fields(&args.fields)?;

    // Helper closure to check if a field is selected
    let is_selected = |field: &str| *fields_selected.get(field).unwrap_or(&false);

    let options = Options::default();
    let mut database = DB::open(&args.chainstate, options).context("Couldn't open LevelDB")?;

    let output_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args.output_file)
        .with_context(|| {
            format!(
                "Output file {} already exists or cannot be created",
                args.output_file
            )
        })?;

    let mut writer = BufWriter::new(output_file);

    if !args.quiet {
        println!(
            "Processing {} and writing results to {}",
            args.chainstate.display(),
            args.output_file
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
    let mut script_type_count: BTreeMap<&str, u32> = BTreeMap::new();
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
        signals.wait();
        r.store(false, Ordering::SeqCst);
    });

    let mut db_iter = database.new_iter()?;

    // Read first value: obfuscation key
    db_iter.reset();
    let (key, value) = db_iter.next().unwrap();
    let (prefix, key) = key.split_at(2);
    let key_str =
        String::from_utf8(key.to_vec()).context("Failed to convert key bytes to valid UTF-8")?;
    let obfuscate_key = if prefix == DB_KEYS_OBFUSCATE_KEY_PREFIX
        && key_str == DB_KEYS_OBFUSCATE_KEY
    {
        if !args.quiet {
            println!(">>> Obfuscation key: {}", hex::encode(&value[1..]));
        }
        value[1..].to_vec() // Ignore first byte (size of the obfuscate key)
    } else {
        anyhow::bail!(
            "No obfuscation key found in chainstate database.\n\
             This database may be corrupted or from an older version that doesn't use obfuscation.\n\
             Cannot process UTXO values without the obfuscation key."
        );
    };

    while db_iter.valid() {
        if !running.load(Ordering::SeqCst) {
            if !args.quiet {
                println!("Interrupt signal caught. Shutting down gracefully.");
            }
            break;
        }

        if let Some((key, mut value)) = db_iter.next() {
            let prefix = key[0];
            if prefix == blockchain.utxo_key_prefix() {
                // -----------------------
                // DB Key for Bitcoin UTXO
                // -----------------------

                //      430000155b9869d56c66d9e86e3c01de38e3892a42b99949fe109ac034fff6583900
                //      <><--------------------------------------------------------------><>
                //      /                               |                                  \
                //  prefix                      txid (little-endian)                 vout (varint)

                // ------------------------
                // DB Key for Dogecoin UTXO
                // ------------------------

                // Ref: <https://en.bitcoin.it/wiki/Bitcoin_Core_0.11_(ch_2):_Data_Storage>

                //      6300007c264b7c8b886b0c7d086d7b42e1737d6c3fb94b85eaecda45be1eb3b6a0
                //      <><-------------------------------------------------------------->
                //      /                               |
                //  prefix                      txid (little-endian)

                // XOR deobfuscate the value in place
                for (i, v) in value.iter_mut().enumerate() {
                    *v ^= obfuscate_key[i % obfuscate_key.len()];
                }
                let deobfuscated_value = value;

                let mut csv_output = HashMap::new();

                // Deserialize UTXO value
                let outputs = blockchain.deserialize_db_utxo(deobfuscated_value)?;

                for output in outputs {
                    // txid
                    if is_selected("txid") {
                        let mut txid = key[1..33].to_vec();
                        txid.reverse(); // Reverse byte order (little-endian to big-endian)
                        csv_output.insert("txid", hex::encode(txid));
                    }

                    // vout
                    if is_selected("vout") {
                        if key.len() >= 34 {
                            // Modern: vout is encoded in the key
                            anyhow::ensure!(
                                matches!(blockchain, Blockchain::Bitcoin(_)),
                                "Expected Bitcoin blockchain for modern vout encoding"
                            );
                            let vout_bytes = &key[33..];
                            let mut cursor = Cursor::new(vout_bytes);
                            let vout = read_varint(&mut cursor)?;
                            csv_output.insert("vout", vout.to_string());
                        } else if key.len() == 33 {
                            // Legacy: vout is encoded in the value
                            anyhow::ensure!(
                                matches!(blockchain, Blockchain::Dogecoin(_)),
                                "Expected Dogecoin blockchain for legacy vout encoding"
                            );
                            let vout = output
                                .vout
                                .ok_or_else(|| anyhow::anyhow!("vout is missing in the output"))?;
                            csv_output.insert("vout", vout.to_string());
                        } else {
                            anyhow::bail!("Invalid key length: {}", key.len());
                        }
                    }

                    // coinbase
                    if is_selected("coinbase") {
                        csv_output.insert("coinbase", output.coinbase.to_string());
                    }

                    // height
                    if is_selected("height") {
                        csv_output.insert("height", output.height.to_string());
                    }

                    // amount
                    if is_selected("amount") {
                        let amount = output.txout.amount;
                        csv_output.insert("amount", amount.to_string());
                        total_amount += amount;
                    }

                    // nsize
                    if is_selected("nsize") {
                        csv_output.insert("nsize", output.txout.nsize.to_string());
                    }

                    // address and script type processing
                    if is_selected("address") || is_selected("type") {
                        let script_type = output.txout.script_type;
                        if let Some(count) = script_type_count.get_mut(script_type.as_str()) {
                            *count += 1;
                        }
                        csv_output.insert("address", output.txout.address);
                        csv_output.insert("type", script_type);
                    }

                    if is_selected("script") {
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
                    if !args.quiet && utxo_count > 0 && utxo_count % 100000 == 0 {
                        println!("{} utxos processed", utxo_count);
                    }
                }
            }
        } else {
            break;
        }
    }
    writer.flush()?;

    println!("\nTotal UTXOs: {}", utxo_count);

    if is_selected("amount") {
        println!(
            "Total {}:   {:.8}",
            blockchain.ticker(),
            total_amount as f64 / 100_000_000.0
        );
    }

    if is_selected("type") {
        println!("Script Types:");
        for (script_type, count) in script_type_count {
            println!(" {:<12} {}", script_type, count);
        }
    }

    Ok(())
}

fn validate_and_parse_fields(fields_str: &str) -> Result<BTreeMap<String, bool>> {
    let mut fields_selected = BTreeMap::new();
    for field in &FIELDS_ALLOWED {
        fields_selected.insert(field.to_string(), false);
    }

    for field in fields_str.split(',') {
        let field = field.trim();
        if !FIELDS_ALLOWED.contains(&field) {
            anyhow::bail!(
                "'{}' is not a field you can use for the output.\nChoose from the following: {}",
                field,
                FIELDS_ALLOWED.join(",")
            );
        }
        fields_selected.insert(field.to_string(), true);
    }

    Ok(fields_selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainstate::deserialize_db_utxo_legacy;
    use bitcoin::PubkeyHash;
    use std::str::FromStr;

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
        let outputs =
            deserialize_db_utxo_legacy(&blockchain, hex::decode(hex_data).unwrap()).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].vout.unwrap(), 1);
        assert_eq!(outputs[0].coinbase, 0);
        assert_eq!(outputs[0].txout.amount, 60000000000);
        assert_eq!(outputs[0].txout.script_type, "p2pkh");
        let pubkey_hash = PubkeyHash::from_str("816115944e077fe7c803cfa57f29b36bf87c1d35").unwrap();
        assert_eq!(
            outputs[0].txout.address,
            blockchain.p2pkh_address(pubkey_hash)
        );
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
        let outputs =
            deserialize_db_utxo_legacy(&blockchain, hex::decode(hex_data).unwrap()).unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].vout.unwrap(), 4);
        assert_eq!(outputs[0].coinbase, 1);
        assert_eq!(outputs[0].txout.amount, 234925952);
        assert_eq!(outputs[0].txout.script_type, "p2pkh");
        let pubkey_hash_0 =
            PubkeyHash::from_str("61b01caab50f1b8e9c50a5057eb43c2d9563a4ee").unwrap();
        assert_eq!(
            outputs[0].txout.address,
            blockchain.p2pkh_address(pubkey_hash_0)
        );
        assert_eq!(outputs[0].txout.nsize, 0);
        assert_eq!(outputs[0].height, 120891);
        assert_eq!(outputs[1].vout.unwrap(), 16);
        assert_eq!(outputs[1].coinbase, 1);
        assert_eq!(outputs[1].txout.amount, 110397);
        assert_eq!(outputs[1].txout.script_type, "p2pkh");
        let pubkey_hash_1 =
            PubkeyHash::from_str("8c988f1a4a4de2161e0f50aac7f17e7f9555caa4").unwrap();
        assert_eq!(
            outputs[1].txout.address,
            blockchain.p2pkh_address(pubkey_hash_1)
        );
        assert_eq!(outputs[1].txout.nsize, 0);
        assert_eq!(outputs[1].height, 120891);
    }
}
