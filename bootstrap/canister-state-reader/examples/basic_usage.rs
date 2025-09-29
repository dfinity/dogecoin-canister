use canister_state_reader::UtxoReader;
use std::env;

/// Example showing basic usage of the canister state reader crate
/// 
/// Usage: cargo run --example basic_usage -- path/to/canister_state.bin
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <canister_state.bin>", args[0]);
        std::process::exit(1);
    }
    
    let canister_state_path = &args[1];
    println!("Reading canister state from: {}", canister_state_path);
    
    // Create the UTXO reader
    let reader = UtxoReader::new(canister_state_path)?;
    
    // Extract all UTXOs
    println!("Extracting UTXOs from stable memory regions...");
    let utxos = reader.extract_utxos()?;
    
    println!("Found {} total UTXOs", utxos.len());
    
    // Show first few UTXOs as example
    println!("\nFirst 5 UTXOs:");
    for (i, utxo) in utxos.iter().take(5).enumerate() {
        let txid_hex = {
            let mut txid_bytes = utxo.outpoint.txid.as_bytes().to_vec();
            txid_bytes.reverse(); // Bitcoin displays txids in reverse byte order
            hex::encode(txid_bytes)
        };
        
        let value_doge = utxo.txout.value as f64 / 100_000_000.0;
        println!("  {}: {}:{} - {:.8} DOGE (height: {})", 
                 i + 1, 
                 txid_hex,
                 utxo.outpoint.vout,
                 value_doge,
                 utxo.height);
    }
    
    // Compute and display the hash
    let hash = UtxoReader::compute_utxo_set_hash(&utxos);
    println!("\nUTXO Set Hash (SHA256): {}", hash);
    
    // Calculate total value
    let total_value: u64 = utxos.iter().map(|u| u.txout.value).sum();
    let total_doge = total_value as f64 / 100_000_000.0;
    
    println!("Total Value: {} satoshis ({:.8} DOGE)", total_value, total_doge);
    
    Ok(())
}
