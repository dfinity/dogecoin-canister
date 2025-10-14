use clap::{Parser, ValueEnum};
use separator::Separatable;
use std::{fs::File, path::PathBuf, collections::HashMap};
use std::collections::HashSet;
use state_reader::{CanisterData, Utxo, UtxoReader, hash, set_logging_quiet, log};
use ic_doge_types::BlockHash;
use ic_stable_structures::Storable;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum DataType {
    Utxos,
    Balances,
    Headers,
}


#[derive(Parser, Debug)]
#[command(name = "state-reader")]
#[command(about = "A CLI tool to read and analyze all data from a Dogecoin canister state file")]
struct Args {
    /// Path to the canister_state.bin file
    #[arg(short, long, value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,

    /// Only output the combined canister state hash
    #[arg(short, long)]
    quiet: bool,

    /// Select which data types to process (default: all)
    #[arg(long, value_enum, value_delimiter = ',')]
    data: Option<Vec<DataType>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    set_logging_quiet(args.quiet);
    
    // Determine which data types to process (default: all)
    let data_types = args.data.unwrap_or_else(|| vec![DataType::Utxos, DataType::Balances, DataType::Headers]);
    let process_utxos = data_types.contains(&DataType::Utxos);
    let process_balances = data_types.contains(&DataType::Balances);
    let process_headers = data_types.contains(&DataType::Headers);
    
    log!("Processing data types: {:?}", data_types);

    if !args.input.exists() {
        eprintln!("Error: Input file '{}' does not exist", args.input.display());
        std::process::exit(1);
    }

    log!("Reading canister state from: {}", args.input.display());

    // Set up access to the canister memory region from the state file
    ic_doge_canister::memory::set_memory(ic_stable_structures::FileMemory::new(
        File::open(&args.input)?
    ));
    
    // Create a fresh empty state
    ic_doge_canister::init(ic_doge_interface::InitConfig::default());
    
    // Deserialize the state from upgrade memory region 0 (including large UTXOs)
    ic_doge_canister::post_upgrade(None);

    let reader = UtxoReader::new(&args.input)?;

    log!("Extracting data from stable memory...");

    let mut canister_data = reader.read_data(process_utxos, process_balances, process_headers);

    // Extract large UTXOs from the deserialized canister state (only if processing UTXOs)
    let mut utxos = canister_data.utxos.clone();
    if process_utxos {
        log!("Extracting large UTXOs from canister state...");
        let large_utxos = ic_doge_canister::with_state(|state| {
            state.utxos.utxos.large_utxos.clone()
        });
        let large_utxo_count = large_utxos.len();
        for (outpoint, (txout, height)) in large_utxos {
            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });
        }
        log!("Extracted {} large UTXOs from canister state", large_utxo_count);
    }

    // TODO(XC-501): temporary workaround to remove unspendable UTXO from Genesis block.
    // Note that we don't need to remove the corresponding entry from the balance map and
    // address UTXOs map because the script is P2PK and is never translated into an address.
    if process_utxos {
        log!("Removing unspendable UTXO from Genesis block...");
        utxos.retain(|utxo| utxo.height != 0);
    }

    // TODO(XC-505): there is a discrepancy between addresses and UTXOs for 0-amount UTXOs. Ignore addresses with 0 balance for now.
    if process_balances {
        log!("Filtering out addresses with 0 balance...");
        let initial_balance_count = canister_data.balances.len();
        canister_data.balances.retain(|(_address, balance)| *balance != 0);
        log!("Filtered out {} addresses with 0 balance (kept {})", 
                              initial_balance_count - canister_data.balances.len(), 
                              canister_data.balances.len());
    }

    // Sort the data for deterministic hashing
    log!("Sorting data for deterministic hashing...");
    log!("  Sorting {} UTXOs...", utxos.len());
    utxos.sort();
    
    log!("  Sorting {} address UTXOs...", canister_data.address_utxos.len());
    canister_data.address_utxos.sort_by(|a, b| {
        a.address.to_string()
            .cmp(&b.address.to_string())
            .then(a.height.cmp(&b.height))
            .then(a.outpoint.cmp(&b.outpoint))
    });
    
    log!("  Sorting {} address balances...", canister_data.balances.len());
    canister_data.balances.sort_by(|a, b| {
        a.0.cmp(&b.0).then(a.1.cmp(&b.1))
    });
    
    log!("  Sorting {} block headers...", canister_data.block_headers.len());
    canister_data.block_headers.sort_by(|a, b| {
        a.0.cmp(&b.0)
    });
    
    log!("  Sorting {} block heights...", canister_data.block_heights.len());
    canister_data.block_heights.sort_by(|a, b| {
        a.0.cmp(&b.0)
    });
    

    log!("Validating data consistency...");
    if let Err(error) = check_invariants(&canister_data, &utxos) {
        eprintln!("Data consistency check failed: {}", error);
        std::process::exit(1);
    }

    if !args.quiet {
        print_statistics(&canister_data, &utxos);
    }

    log!("Computing data hashes...");
    
    let utxo_hash = if process_utxos {
        log!("  Computing UTXO set hash ({} entries)...", utxos.len());
        hash::compute_utxo_set_hash(&utxos)
    } else {
        "0".repeat(64)  // Empty hash for skipped data
    };
    
    let (address_utxos_hash, address_balance_hash) = if process_balances {
        log!("  Computing address UTXOs hash ({} entries)...", canister_data.address_utxos.len());
        let addr_utxos_hash = hash::compute_address_utxos_hash(&canister_data.address_utxos);
        
        log!("  Computing address balances hash ({} entries)...", canister_data.balances.len());
        let addr_balance_hash = hash::compute_address_balances_hash(&canister_data.balances);
        
        (addr_utxos_hash, addr_balance_hash)
    } else {
        ("0".repeat(64), "0".repeat(64))  // Empty hashes for skipped data
    };
    
    let (block_headers_hash, block_heights_hash) = if process_headers {
        log!("  Computing block headers hash ({} entries)...", canister_data.block_headers.len());
        let headers_hash = hash::compute_block_headers_hash(&canister_data.block_headers);
        
        log!("  Computing block heights hash ({} entries)...", canister_data.block_heights.len());
        let heights_hash = hash::compute_block_heights_hash(&canister_data.block_heights);
        
        (headers_hash, heights_hash)
    } else {
        ("0".repeat(64), "0".repeat(64))  // Empty hashes for skipped data
    };

    log!("  Computing combined hash...");
    let hash_data = hash::compute_combined_hash(&[
        &utxo_hash,
        &address_utxos_hash,
        &address_balance_hash,
        &block_headers_hash,
        &block_heights_hash,
    ]);

    if !args.quiet {
        println!("{}", "═".repeat(120));
        println!("{:^120}", "DATA HASHES (SHA256)");
        println!("{}", "═".repeat(120));
        
        println!("\n{:<16}: {}", "UTXO Set", utxo_hash);
        println!("{:<16}: {}", "Address UTXOs", address_utxos_hash);
        println!("{:<16}: {}", "Address Balance", address_balance_hash);
        println!("{:<16}: {}", "Block Headers", block_headers_hash);
        println!("{:<16}: {}", "Block Heights", block_heights_hash);

        println!("\n{:<16}: {}", "Combined hash", hash_data);
    } else {
        println!("{}", hash_data);
    }

    Ok(())
}

/// Validates the integrity and consistency of canister data
/// 
/// This function checks several invariants that should hold for valid canister state:
/// - No UTXOs should exist at height 0 (they are unspendable by consensus rules)
/// - Block headers should be at least 80 bytes
/// - Block headers should not be all zeros (happens for AuxPow header when the daemon was run in pruned mode)
/// - Block headers count should match block heights count
/// - Block headers and heights should have no duplicated entries
/// - Block heights should have no missing blocks in the height range
fn check_invariants(data: &CanisterData, utxos: &[Utxo]) -> Result<(), String> {
    if data.block_heights.is_empty() {
        return if data.address_utxos.is_empty() && data.balances.is_empty() {
            Ok(())
        } else {
            Err("Found UTXOs/balances without blocks".to_string())
        };
    }

    // Check for UTXOs at height 0 (genesis does not have spendable UTXOs)
    let utxo_height_zero_count = utxos.iter().filter(|addr_utxo| addr_utxo.height == 0).count();
    if utxo_height_zero_count > 0 {
        return Err(format!("Found {} UTXOs at height 0, expected none", utxo_height_zero_count));
    }
    log!("  ✓ No UTXOs at height 0");

    // Check for undersized block headers
    let header_sizes: Vec<usize> = data.block_headers.iter()
        .map(|(_, blob)| blob.as_slice().len())
        .collect();
    let undersized_count = header_sizes.iter().filter(|&&size| size < 80).count();
    if undersized_count > 0 {
        return Err(format!("Found {} block headers smaller than 80 bytes, expected none", undersized_count));
    }
    log!("  ✓ All block headers are properly sized");

    // Check for zero block headers
    let zero_hash = BlockHash::from(vec![0u8; 32]);
    let zero_header_count = data.block_headers.iter()
        .filter(|(block_hash, _)| *block_hash == zero_hash)
        .count();
    if zero_header_count > 0 {
        return Err(format!("Found {} zero block headers, expected none", zero_header_count));
    }
    log!("  ✓ No all-zeros block headers found");

    // Check header/height consistency
    let headers_count = data.block_headers.len();
    let heights_count = data.block_heights.len();
    
    if headers_count != heights_count {
        return Err(format!(
            "Header count mismatch: {} block headers entries vs {} block heights entries",
            headers_count, heights_count
        ));
    }
    log!("  ✓ Header and height counts match ({} entries)", headers_count);

    let mut heights: Vec<u32> = data.block_heights.iter().map(|(height, _)| *height).collect();
    heights.sort_unstable();

    // Check duplicate block height entries
    let unique_heights: HashSet<u32> = heights.iter().cloned().collect();
    if unique_heights.len() != heights.len() {
        return Err(format!("Found {} duplicate block heights",
                           heights.len() - unique_heights.len()));
    }
    log!("  ✓ No duplicate block heights found");

    // Check duplicate block headers entries
    let unique_hashes: HashSet<&BlockHash> = data.block_headers.iter().map(|(blockhash, _)| blockhash).collect();
    if unique_hashes.len() != data.block_headers.len() {
        return Err(format!("Found {} duplicate block hashes",
                           data.block_headers.len() - unique_hashes.len()));
    }
    log!("  ✓ No duplicate block headers found");

    // Check block height continuity
    let min_height = *heights.first().unwrap();
    let max_height = *heights.last().unwrap();
    let height_span = max_height - min_height + 1;

    if height_span as usize != heights.len() {
        return Err(format!("Missing blocks: expected {} blocks from {} to {}, found {}",
                           height_span, min_height, max_height, heights.len()));
    }
    log!("  ✓ Block height continuity verified ({} to {} = {} blocks)",
         min_height, max_height, height_span);

    log!("All data consistency checks passed ✓");
    Ok(())
}

fn print_section_header(section_num: usize, title: &str) {
    let total_width = 120;
    let title_with_spaces = format!(" {}: {} ", section_num, title);
    let padding = (total_width - title_with_spaces.len()) / 2;
    let left_border = "═".repeat(padding);
    let right_border = "═".repeat(total_width - padding - title_with_spaces.len());
    
    println!("\n{}{}{}", left_border, title_with_spaces, right_border);
}

fn print_statistics(data: &CanisterData, utxos: &[Utxo]) {
    print_section_header(1, "UTXOs");
    if !utxos.is_empty() {
        println!("\nFirst {} UTXO Details:", std::cmp::min(20, utxos.len()));
        println!("{:<8} {:<66} {:<5} {:<20} {:<12} {}",
                 "Index", "Txid", "Vout", "Value (DOGE)", "Height", "Script Size");
        println!("{}", "-".repeat(120));

        for (i, utxo) in utxos.iter().take(20).enumerate() {
            let txid_hex = {
                let mut txid_bytes = utxo.outpoint.txid.as_bytes().to_vec();
                txid_bytes.reverse();
                hex::encode(txid_bytes)
            };

            let value_doge = utxo.txout.value as f64 / 100_000_000.0;

            println!("{:<8} {:<66} {:<5} {:<20} {:<12} {}",
                     i + 1,
                     txid_hex,
                     utxo.outpoint.vout,
                     value_doge,
                     utxo.height,
                     utxo.txout.script_pubkey.len()
            );
        }

        let total_value: u64 = utxos.iter().map(|u| u.txout.value).sum();
        let total_value_doge = total_value as f64 / 100_000_000.0;

        println!("\n  Total UTXOs: {}", utxos.len().separated_string());
        println!("  Total Value: {} DOGE", total_value_doge.separated_string());

        let min_height = utxos.iter().map(|u| u.height).min().unwrap();
        let max_height = utxos.iter().map(|u| u.height).max().unwrap();
        println!("  UTXO Height Range: {} - {}", min_height.separated_string(), max_height.separated_string());

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

        println!("  Value Distribution (DOGE):");
        println!("    Min:     {:.8}", min_value);
        println!("    25th %:  {}", p25.separated_string());
        println!("    Median:  {}", median.separated_string());
        println!("    Mean:    {}", mean_value.separated_string());
        println!("    75th %:  {}", p75.separated_string());
        println!("    90th %:  {}", p90.separated_string());
        println!("    95th %:  {}", p95.separated_string());
        println!("    99th %:  {}", p99.separated_string());
        println!("    Max:     {}\n", max_value.separated_string());

        let zero_utxos_count = utxos.iter().filter(|u| u.txout.value == 0).count();
        println!("    Number of UTXOs with 0 amount: {}\n", zero_utxos_count);
    }

    print_section_header(2, "Address UTXOs");
    println!("\n  Total Address UTXOs entries: {}", data.address_utxos.len().separated_string());
    
    if !data.address_utxos.is_empty() {
        println!("\nFirst {} Address UTXO Details:", std::cmp::min(20, data.address_utxos.len()));
        println!("{:<8} {:<40} {:<66} {:<5} {}",
                 "Index", "Address", "Txid", "Vout", "Height");
        println!("{}", "-".repeat(120));

        for (i, addr_utxo) in data.address_utxos.iter().take(20).enumerate() {
            let txid_hex = {
                let mut txid_bytes = addr_utxo.outpoint.txid.as_bytes().to_vec();
                txid_bytes.reverse();
                hex::encode(txid_bytes)
            };

            println!("{:<8} {:<40} {:<66} {:<5} {}",
                     i + 1,
                     addr_utxo.address.to_string(),
                     txid_hex,
                     addr_utxo.outpoint.vout,
                     addr_utxo.height
            );
        }

        let mut address_counts: HashMap<String, usize> = HashMap::new();
        let mut heights: Vec<u32> = Vec::new();

        for addr_utxo in &data.address_utxos {
            *address_counts.entry(addr_utxo.address.to_string()).or_insert(0) += 1;
            heights.push(addr_utxo.height);
        }

        let unique_addresses = address_counts.len();
        let total_entries = data.address_utxos.len();
        
        // UTXO count distribution
        let mut counts: Vec<usize> = address_counts.values().cloned().collect();
        counts.sort_unstable();
        
        let min_utxos_per_addr = *counts.first().unwrap_or(&0);
        let max_utxos_per_addr = *counts.last().unwrap_or(&0);
        let avg_utxos_per_addr = total_entries as f64 / unique_addresses as f64;
        let median_utxos_per_addr = if counts.is_empty() { 0 } else { counts[counts.len() / 2] };

        println!("\n  Unique addresses: {}", unique_addresses.separated_string());
        println!("  UTXOs per address - Min: {}, Max: {}, Avg: {:.1}, Median: {}",
                 min_utxos_per_addr, max_utxos_per_addr, avg_utxos_per_addr, median_utxos_per_addr);

        // Address reuse patterns
        let single_utxo_addresses = counts.iter().filter(|&&count| count == 1).count();
        let multi_utxo_addresses = unique_addresses - single_utxo_addresses;
        
        println!("  Single-use addresses: {} ({:.2}%)",
                 single_utxo_addresses.separated_string(),
                 (single_utxo_addresses as f64 / unique_addresses as f64) * 100.0);
        println!("  Reused addresses: {} ({:.2}%)",
                 multi_utxo_addresses.separated_string(),
                 (multi_utxo_addresses as f64 / unique_addresses as f64) * 100.0);

        // Top addresses by UTXO count
        let mut sorted_addresses: Vec<_> = address_counts.iter().collect();
        sorted_addresses.sort_by(|a, b| b.1.cmp(a.1));
        println!("\n  Top 5 Addresses by UTXO Count:");
        for (i, (address, count)) in sorted_addresses.iter().take(5).enumerate() {
            println!("    {}: {} ({} UTXOs)", i + 1, address, count.separated_string());
        }

        // Height distribution
        heights.sort_unstable();
        let min_height = *heights.first().unwrap_or(&0);
        let max_height = *heights.last().unwrap_or(&0);

        println!("\n  Height range: {} - {}",
                 min_height.separated_string(), max_height.separated_string());

        // Address type analysis
        let mut p2pkh_count = 0;
        let mut p2sh_count = 0;
        let mut other_count = 0;
        
        for addr_utxo in &data.address_utxos {
            let addr_str = addr_utxo.address.to_string();
            if addr_str.starts_with('D') {
                p2pkh_count += 1;
            } else if addr_str.starts_with('A') || addr_str.starts_with('9') {
                p2sh_count += 1;
            } else {
                other_count += 1;
            }
        }
        
        println!("\n  Address Type Distribution:");
        println!("    P2PKH (D*):    {} ({:.2}%)",
                 p2pkh_count.separated_string(),
                 (p2pkh_count as f64 / total_entries as f64) * 100.0);
        println!("    P2SH (A*/9*):  {} ({:.2}%)",
                 p2sh_count.separated_string(),
                 (p2sh_count as f64 / total_entries as f64) * 100.0);
        if other_count > 0 {
            println!("    Other formats:   {} ({:.2}%)",
                     other_count.separated_string(),
                     (other_count as f64 / total_entries as f64) * 100.0);
        }
    }

    print_section_header(3, "Address Balance");
    let balance_count = data.balances.len();
    println!("\n  Total Address Balances entries: {}", balance_count.separated_string());
    
    if !data.balances.is_empty() {
        let mut balances_satoshis: Vec<u128> = data.balances.iter().map(|(_, balance)| *balance).collect();
        let mut balances_doge: Vec<f64> = balances_satoshis.iter().map(|&b| b as f64 / 100_000_000.0).collect();
        balances_satoshis.sort_unstable();
        balances_doge.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let total_supply: u128 = balances_satoshis.iter().sum();
        let total_supply_doge = (total_supply as f64 / 100_000_000.0) as u128;
        let mean_balance = total_supply as f64 / balances_satoshis.len() as f64;

        println!("\n  Total Supply: {} DOGE", total_supply_doge.separated_string());

        println!("\n  Balance Distribution (Non-zero addresses):");
        println!("    Min:     {:.8} DOGE", balances_doge[0]);
        println!("    Median:  {:.8} DOGE", balances_doge[balances_doge.len() / 2]);
        println!("    Mean:    {:.8} DOGE", mean_balance / 100_000_000.0);
        println!("    Max:     {:.8} DOGE", *balances_doge.last().unwrap());

        let p25 = percentile(&balances_doge, 25.0);
        let p75 = percentile(&balances_doge, 75.0);
        let p90 = percentile(&balances_doge, 90.0);
        let p95 = percentile(&balances_doge, 95.0);
        let p99 = percentile(&balances_doge, 99.0);

        println!("    25th %:  {:.8} DOGE", p25);
        println!("    75th %:  {:.8} DOGE", p75);
        println!("    90th %:  {:.8} DOGE", p90);
        println!("    95th %:  {:.8} DOGE", p95);
        println!("    99th %:  {:.8} DOGE", p99);

        let dust_threshold = 10_000_000u128; // 0.1 DOGE
        let small_threshold = 10_000_000_000u128; // 100 DOGE
        let medium_threshold = 1_000_000_000_000u128; // 10,000 DOGE
        let large_threshold = 1_000_000_000_000_000u128; // 10,000,000 DOGE

        let dust_count = balances_satoshis.iter().filter(|&&b| b > 0 && b < dust_threshold).count();
        let small_count = balances_satoshis.iter().filter(|&&b| b >= dust_threshold && b < small_threshold).count();
        let medium_count = balances_satoshis.iter().filter(|&&b| b >= small_threshold && b < medium_threshold).count();
        let large_count = balances_satoshis.iter().filter(|&&b| b >= medium_threshold && b < large_threshold).count();
        let whale_count = balances_satoshis.iter().filter(|&&b| b >= large_threshold).count();

        println!("\n  Balance Range Distribution:");
        println!("    Dust (<0.1 DOGE):      {} ({:.2}%)",
                 dust_count.separated_string(),
                 (dust_count as f64 / balance_count as f64) * 100.0);
        println!("    Small (0.1-100 DOGE):  {} ({:.2}%)",
                 small_count.separated_string(),
                 (small_count as f64 / balance_count as f64) * 100.0);
        println!("    Medium (100-10K DOGE): {} ({:.2}%)",
                 medium_count.separated_string(),
                 (medium_count as f64 / balance_count as f64) * 100.0);
        println!("    Large (10K-10M DOGE):  {} ({:.2}%)",
                 large_count.separated_string(),
                 (large_count as f64 / balance_count as f64) * 100.0);
        println!("    Whale (>10M DOGE):     {} ({:.2}%)",
                 whale_count.separated_string(),
                 (whale_count as f64 / balance_count as f64) * 100.0);

        // Zero balance addresses
        let zero_balance_count = balances_satoshis.iter().filter(|&&balance| balance == 0).count();
        println!("\n  Number of addresses with zero balance: {}", zero_balance_count);

        // Top addresses by balance
        let mut sorted_balances: Vec<_> = data.balances.iter().collect();
        sorted_balances.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\n  Top 10 Addresses by Balance:");
        for (i, (address, balance)) in sorted_balances.iter().take(10).enumerate() {
            let balance_doge = (*balance as f64 / 100_000_000.0) as u64;
            let percentage = (*balance as f64 / total_supply as f64) * 100.0;
            println!("    {}: {} = {} DOGE ({:.4}% of supply)",
                     i + 1, address, balance_doge.separated_string(), percentage);
        }

        // Wealth concentration analysis
        let top_1_percent = std::cmp::max(1, balance_count / 100);
        let top_5_percent = std::cmp::max(1, balance_count * 5 / 100);
        let top_10_percent = std::cmp::max(1, balance_count / 10);

        let top_1_wealth: u128 = balances_satoshis.iter().rev().take(top_1_percent).sum();
        let top_5_wealth: u128 = balances_satoshis.iter().rev().take(top_5_percent).sum();
        let top_10_wealth: u128 = balances_satoshis.iter().rev().take(top_10_percent).sum();

        println!("\n  Wealth Concentration:");
        println!("    Top 1% of addresses hold: {:.2}% of total supply",
                 (top_1_wealth as f64 / total_supply as f64) * 100.0);
        println!("    Top 5% of addresses hold: {:.2}% of total supply",
                 (top_5_wealth as f64 / total_supply as f64) * 100.0);
        println!("    Top 10% of addresses hold: {:.2}% of total supply",
                 (top_10_wealth as f64 / total_supply as f64) * 100.0);

    }

    print_section_header(4, "Block Headers");
    let headers_count = data.block_headers.len();
    let heights_count = data.block_heights.len();
    
    println!("\n  Total block headers entries: {}", headers_count.separated_string());
    println!("  Total block heights entries: {}", heights_count.separated_string());

    if !data.block_headers.is_empty() && !data.block_heights.is_empty() {
        let mut heights: Vec<u32> = data.block_heights.iter().map(|(height, _)| *height).collect();
        heights.sort_unstable();

        let min_height = *heights.first().unwrap();
        let max_height = *heights.last().unwrap();
        let height_span = max_height - min_height + 1;

        println!("\n  Block Height Analysis:");
        println!("    Height range: {} - {} (span: {} blocks)", 
                 min_height.separated_string(), max_height.separated_string(), height_span.separated_string());

        let mut header_sizes: Vec<usize> = data.block_headers.iter()
            .map(|(_, blob)| blob.as_slice().len())
            .collect();
        header_sizes.sort_unstable();
        println!("\n  Block Header Size Analysis:");
        // Standard header (pure header) is 80 bytes, AuxPow header is larger than 80 bytes
        let standard_size_count = header_sizes.iter().filter(|&&size| size == 80).count();
        let auxpow_sizes: Vec<usize> = header_sizes.into_iter().filter(|&size| size > 80).collect();
        let auxpow_count = auxpow_sizes.len();

        println!("    Standard Header (80 bytes): {} ({:.2}%)",
                 standard_size_count.separated_string(),
                 (standard_size_count as f64 / headers_count as f64) * 100.0);
        println!("    AuxPow Header (>80 bytes):  {} ({:.2}%)",
                 auxpow_count.separated_string(),
                 (auxpow_count as f64 / headers_count as f64) * 100.0);

        // AuxPow size distribution analysis
        if auxpow_count > 0 {
            let mut sorted_auxpow_sizes = auxpow_sizes.clone();
            sorted_auxpow_sizes.sort_unstable();

            let min_auxpow = *sorted_auxpow_sizes.first().unwrap();
            let max_auxpow = *sorted_auxpow_sizes.last().unwrap();
            let mean_auxpow = auxpow_sizes.iter().sum::<usize>() as f64 / auxpow_sizes.len() as f64;
            let median_auxpow = sorted_auxpow_sizes[sorted_auxpow_sizes.len() / 2];

            println!("\n  AuxPow Size Distribution Analysis:");
            println!("    AuxPow data size range: {} - {} bytes",
                     min_auxpow.separated_string(), max_auxpow.separated_string());
            println!("    Mean AuxPow size: {:.1} bytes", mean_auxpow);
            println!("    Median AuxPow size: {} bytes", median_auxpow.separated_string());

            // Size range buckets for AuxPow data
            let small_auxpow = auxpow_sizes.iter().filter(|&&size| size < 500).count();
            let medium_auxpow = auxpow_sizes.iter().filter(|&&size| size >= 500 && size < 1000).count();
            let large_auxpow = auxpow_sizes.iter().filter(|&&size| size >= 1000 && size < 2000).count();
            let xlarge_auxpow = auxpow_sizes.iter().filter(|&&size| size >= 2000).count();

            println!("\n    AuxPow Size Range Distribution:");
            println!("      Small (<500 bytes):     {} ({:.2}%)",
                     small_auxpow.separated_string(),
                     (small_auxpow as f64 / auxpow_count as f64) * 100.0);
            println!("      Medium (500-999 bytes): {} ({:.2}%)",
                     medium_auxpow.separated_string(),
                     (medium_auxpow as f64 / auxpow_count as f64) * 100.0);
            println!("      Large (1-2KB):          {} ({:.2}%)",
                     large_auxpow.separated_string(),
                     (large_auxpow as f64 / auxpow_count as f64) * 100.0);
            println!("      X-Large (>2KB):         {} ({:.2}%)",
                     xlarge_auxpow.separated_string(),
                     (xlarge_auxpow as f64 / auxpow_count as f64) * 100.0);

            // Percentile analysis for AuxPow sizes
            let auxpow_f64: Vec<f64> = sorted_auxpow_sizes.iter().map(|&x| x as f64).collect();
            let p25_auxpow = percentile(&auxpow_f64, 25.0) as usize;
            let p75_auxpow = percentile(&auxpow_f64, 75.0) as usize;
            let p90_auxpow = percentile(&auxpow_f64, 90.0) as usize;
            let p95_auxpow = percentile(&auxpow_f64, 95.0) as usize;
            let p99_auxpow = percentile(&auxpow_f64, 99.0) as usize;

            println!("\n    AuxPow Size Percentiles:");
            println!("      25th percentile: {} bytes", p25_auxpow.separated_string());
            println!("      75th percentile: {} bytes", p75_auxpow.separated_string());
            println!("      90th percentile: {} bytes", p90_auxpow.separated_string());
            println!("      95th percentile: {} bytes", p95_auxpow.separated_string());
            println!("      99th percentile: {} bytes", p99_auxpow.separated_string());
        }
        
        // Show last 5 block headers
        println!("\n  Last {} Block Headers Details:", std::cmp::min(5, data.block_headers.len()));
        println!("{:<64} {}",
                 "Block Hash", "Height");
        println!("{}", "-".repeat(100));

        for h in heights.iter().rev().take(5) {
            let hash: BlockHash = data.block_heights.iter()
                .find(|(height, _)| height == h)
                .map(|(_ , hash)| hash.clone())
                .unwrap();
            let hash_hex = {
                let mut hash_bytes = hash.to_bytes().to_vec();
                hash_bytes.reverse();
                hex::encode(hash_bytes)
            };
            println!("{:<64} {}",
                     hash_hex,
                     h.separated_string(),
            );
        }
    }

    println!();
}

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