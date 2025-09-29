use clap::Parser;
use separator::Separatable;
use std::{fs::File, path::PathBuf};
use canister_state_reader::{Utxo, UtxoReader};

/// Calculate percentile from a sorted vector
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let p = p.clamp(0.0, 100.0);
    let index = (p / 100.0) * (sorted_values.len() - 1) as f64;
    let lower_index = index.floor() as usize;
    let upper_index = index.ceil() as usize;

    if lower_index == upper_index {
        sorted_values[lower_index]
    } else {
        let weight = index - lower_index as f64;
        sorted_values[lower_index] * (1.0 - weight) + sorted_values[upper_index] * weight
    }
}

#[derive(Parser, Debug)]
#[command(name = "canister-state-reader")]
#[command(about = "A CLI tool to read and analyze all data from a Dogecoin canister state file")]
struct Args {
    /// Path to the canister_state.bin file
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,

    /// Only output the UTXO hash (quiet mode)
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: Input file '{}' does not exist", args.input.display());
        std::process::exit(1);
    }

    if !args.quiet {
        println!("Reading canister state from: {}", args.input.display());
    }

    // Set up access to the canister memory region from the state file
    ic_doge_canister::memory::set_memory(ic_stable_structures::FileMemory::new(
        File::open(&args.input)?
    ));
    
    // Create a fresh empty state
    ic_doge_canister::init(ic_doge_interface::InitConfig::default());
    
    // Deserialize the state from upgrade memory region 0 (including large UTXOs)
    ic_doge_canister::post_upgrade(None);

    let reader = UtxoReader::new(&args.input)?;

    if !args.quiet {
        println!("Extracting UTXOs from stable memory...");
    }

    let canister_data = reader.extract_state_data()?;

    let mut utxos = canister_data.utxos.clone();

    // Extract large UTXOs from the deserialized canister state
    let large_utxos = ic_doge_canister::with_state(|state| {
        state.utxos.utxos.large_utxos.clone()
    });
    for (outpoint, (txout, height)) in large_utxos {
        utxos.push(Utxo {
            outpoint,
            txout,
            height,
        });
    }

    utxos.sort();

    if !args.quiet {
        // Show details of first 20 UTXOs
        if !utxos.is_empty() {
            println!("\nFirst {} UTXO Details:", std::cmp::min(20, utxos.len()));
            println!("{:<8} {:<66} {:<5} {:<20} {:<15} {}",
                     "Index", "Txid", "Vout", "Value (DOGE)", "Height", "Script Size");
            println!("{}", "-".repeat(132));

            for (i, utxo) in utxos.iter().take(20).enumerate() {
                let txid_hex = {
                    let mut txid_bytes = utxo.outpoint.txid.as_bytes().to_vec();
                    txid_bytes.reverse();
                    hex::encode(txid_bytes)
                };

                let value_doge = utxo.txout.value as f64 / 100_000_000.0;

                println!("{:<8} {:<66} {:<5} {:<20} {:<15} {}",
                         i + 1,
                         txid_hex,
                         utxo.outpoint.vout,
                         value_doge,
                         utxo.height,
                         utxo.txout.script_pubkey.len()
                );
            }
            println!();
        }
    }

    let hash = UtxoReader::compute_utxo_set_hash(&utxos);

    if !args.quiet {
        let total_value: u64 = utxos.iter().map(|u| u.txout.value).sum();
        let total_value_doge = total_value as f64 / 100_000_000.0;

        println!("\nUTXOs Statistics:");
        println!("  Total UTXOs: {}", utxos.len().separated_string());
        println!("  Total Value: {} DOGE ({} satoshis)", total_value_doge.separated_string(), total_value);

        if !utxos.is_empty() {
            let min_height = utxos.iter().map(|u| u.height).min().unwrap();
            let max_height = utxos.iter().map(|u| u.height).max().unwrap();
            println!("  Height Range: {} - {}", min_height.separated_string(), max_height.separated_string());

            let script_sizes: Vec<usize> = utxos.iter().map(|u| u.txout.script_pubkey.len()).collect();
            let small_count = script_sizes.iter().filter(|&&size| size <= 25).count();
            let medium_count = script_sizes.iter().filter(|&&size| size > 25 && size <= 201).count();
            let large_count = script_sizes.iter().filter(|&&size| size > 201).count();

            let avg_script_size = script_sizes.iter().sum::<usize>() as f64 / script_sizes.len() as f64;
            let min_script_size = *script_sizes.iter().min().unwrap();
            let max_script_size = *script_sizes.iter().max().unwrap();

            println!("  Script Size Range: {} - {} bytes (avg: {:.1})",
                     min_script_size, max_script_size, avg_script_size);
            
            println!("  Script Size Distribution:");
            println!("    Small (≤25 bytes):     {} ({:.2}%)", small_count.separated_string(), 
                     (small_count as f64 / utxos.len() as f64) * 100.0);
            println!("    Medium (26-201 bytes): {} ({:.2}%)", medium_count.separated_string(), 
                     (medium_count as f64 / utxos.len() as f64) * 100.0);
            println!("    Large (>201 bytes):    {} ({:.2}%)", large_count.separated_string(), 
                     (large_count as f64 / utxos.len() as f64) * 100.0);

            let mut values_doge: Vec<f64> = utxos.iter()
                .map(|u| u.txout.value as f64 / 100_000_000.0)
                .collect();
            values_doge.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min_value = values_doge[0];
            let max_value = values_doge[values_doge.len() - 1] as u64;
            let mean_value = (total_value_doge / values_doge.len() as f64) as u64;

            let median = percentile(&values_doge, 50.0) as u64;
            let p25 = percentile(&values_doge, 25.0) as u64;
            let p75 = percentile(&values_doge, 75.0) as u64;
            let p90 = percentile(&values_doge, 90.0) as u64;
            let p95 = percentile(&values_doge, 95.0) as u64;
            let p99 = percentile(&values_doge, 99.0) as u64;

            println!("\n  Value Distribution (DOGE):");
            println!("    Min:     {:.8}", min_value);
            println!("    25th %:  {}", p25.separated_string());
            println!("    Median:  {}", median.separated_string());
            println!("    Mean:    {}", mean_value.separated_string());
            println!("    75th %:  {}", p75.separated_string());
            println!("    90th %:  {}", p90.separated_string());
            println!("    95th %:  {}", p95.separated_string());
            println!("    99th %:  {}", p99.separated_string());
            println!("    Max:     {}\n", max_value.separated_string());
        }
    }

    println!("Address UTXOs Index: {} entries", canister_data.address_utxos.len().separated_string());
    if !canister_data.address_utxos.is_empty() && canister_data.address_utxos.len() <= 10 {
        println!("  Sample entries:");
        for (i, addr_utxo) in canister_data.address_utxos.iter().take(5).enumerate() {
            let txid_hex = {
                let mut txid_bytes = addr_utxo.outpoint.txid.as_bytes().to_vec();
                txid_bytes.reverse();
                hex::encode(txid_bytes)
            };
            println!("    {}: {} -> {}:{} (height {})",
                     i + 1, addr_utxo.address, txid_hex, addr_utxo.outpoint.vout, addr_utxo.height);
        }
    }

    println!("\nBalances: {} addresses", canister_data.balances.len().separated_string());
    if !canister_data.balances.is_empty() && canister_data.balances.len() <= 10 {
        println!("  Sample balances:");
        for (i, (address, balance)) in canister_data.balances.iter().take(5).enumerate() {
            let balance_doge = *balance as f64 / 100_000_000.0;
            println!("    {}: {} = {:.8} DOGE", i + 1, address, balance_doge);
        }
    }

    println!("\nBlock Headers: {} entries", canister_data.block_headers.len().separated_string());
    if !canister_data.block_headers.is_empty() && canister_data.block_headers.len() <= 10 {
        println!("  Sample block headers:");
        for (i, (block_hash, header_blob)) in canister_data.block_headers.iter().take(5).enumerate() {
            println!("    {}: {} ({} bytes)", i + 1, block_hash, header_blob.as_slice().len());
        }
    }

    println!("\nBlock Heights: {} entries", canister_data.block_heights.len().separated_string());
    if !canister_data.block_heights.is_empty() && canister_data.block_heights.len() <= 10 {
        println!("  Sample height mappings:");
        for (i, (height, block_hash)) in canister_data.block_heights.iter().take(5).enumerate() {
            println!("    {}: {} -> {}", i + 1, height.separated_string(), block_hash);
        }
    }

    if !args.quiet {
        println!("UTXO Set Hash (SHA256): {}", hash);
        // TODO: add rest
    }

    // TODO: if args.quiet, only print the hash of all the information

    Ok(())
}
