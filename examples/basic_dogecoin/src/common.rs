// This module provides common utilities for Dogecoin transaction construction and management.
// It includes UTXO selection algorithms, transaction building, fee estimation, and
// BIP-32 derivation path handling used across all Dogecoin address types.

use crate::{dogecoin_get_fee_percentiles, DogecoinContext};
use bitcoin::{
    self, absolute::LockTime, blockdata::witness::Witness, hashes::Hash, transaction::Version,
    dogecoin::Address, Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
};
use ic_cdk::bitcoin_canister::{
    GetCurrentFeePercentilesRequest, Utxo,
};
use std::fmt;

/// Selects UTXOs using a greedy algorithm to cover the required amount plus fee.
///
/// This function iterates through UTXOs in reverse order (oldest last) and accumulates
/// them until the total value covers the payment amount plus transaction fee.
/// This approach helps consolidate older UTXOs and can reduce wallet fragmentation.
///
/// Returns an error if the total UTXO value is insufficient to cover the payment and fee.
pub fn select_utxos_greedy(
    own_utxos: &[Utxo],
    amount: u64,
    fee: u64,
) -> Result<Vec<&Utxo>, String> {
    // Greedily select UTXOs in reverse order (oldest last) until we cover amount + fee.
    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    for utxo in own_utxos.iter().rev() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        if total_spent >= amount + fee {
            break;
        }
    }

    // Abort if we can't cover the payment + fee.
    if total_spent < amount + fee {
        return Err(format!(
            "Insufficient balance: {}, trying to transfer {} satoshi with fee {}",
            total_spent, amount, fee
        ));
    }

    Ok(utxos_to_spend)
}

/// Represents the primary output type for a Dogecoin transaction.
///
/// This enum allows transaction builders to specify whether they want to send
/// dogecoin to an address (normal payment) or embed data using OP_RETURN.
pub enum PrimaryOutput {
    /// Pay someone (spendable output).
    Address(Address, u64), // destination address, amount in koinus
    /// Embed data (unspendable OP_RETURN output).
    OpReturn(ScriptBuf), // script already starts with OP_RETURN
}

/// Constructs a Dogecoin transaction from the given UTXOs and primary output specification.
///
/// This function handles the common pattern of Dogecoin transaction construction:
/// 1. Creates inputs from the selected UTXOs
/// 2. Creates the primary output (payment or OP_RETURN data)
/// 3. Adds a change output if the remainder exceeds the dust threshold
/// 4. Returns both the unsigned transaction and previous outputs needed for signing
///
/// The change output is sent back to `own_address` to prevent value loss, but only
/// if the change amount is above the dust threshold to avoid creating uneconomical outputs.
///
/// Returns the constructed unsigned transaction.
///
/// Assumes that:
/// - Inputs are unspent and valid (caller's responsibility)
/// - Dust threshold is 1,000 satoshis (outputs below this are omitted)
/// - UTXOs are already filtered to be spendable (confirmed, mature, etc.)
pub fn build_transaction_with_fee(
    utxos_to_spend: Vec<&Utxo>,
    own_address: &Address,
    primary_output: &PrimaryOutput,
    fee: u64,
) -> Result<Transaction, String> {
    // Define a dust threshold below which change outputs are discarded.
    // This prevents creating outputs that cost more to spend than they're worth.
    const DUST_THRESHOLD: u64 = 1_000_000;

    // --- Build Inputs ---
    // Convert UTXOs into transaction inputs, preparing them for signing.
    let inputs: Vec<TxIn> = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::MAX,      // No relative timelock constraints
            witness: Witness::new(),      // Will be filled in during signing
            script_sig: ScriptBuf::new(), // Empty for SegWit and Taproot (uses witness)
        })
        .collect();

    // --- Build Outputs ---
    // Create the primary output based on the operation type.
    let mut outputs = Vec::<TxOut>::new();

    match primary_output {
        PrimaryOutput::Address(addr, amt) => outputs.push(TxOut {
            script_pubkey: addr.script_pubkey(),
            value: Amount::from_sat(*amt),
        }),
        PrimaryOutput::OpReturn(script) => outputs.push(TxOut {
            script_pubkey: script.clone(),
            value: Amount::from_sat(0), // OP_RETURN outputs carry no dogecoin value
        }),
    }

    // Calculate change and add change output if above dust threshold.
    // This prevents value loss while avoiding uneconomical outputs.
    let total_in: u64 = utxos_to_spend.iter().map(|u| u.value).sum();
    let change = total_in
        .checked_sub(outputs.iter().map(|o| o.value.to_sat()).sum::<u64>() + fee)
        .ok_or("fee exceeds inputs")?;

    if change >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: Amount::from_sat(change),
        });
    }

    // --- Assemble Transaction ---
    // Create the final unsigned transaction.
    Ok(
        Transaction {
            input: inputs,
            output: outputs,
            lock_time: LockTime::ZERO, // No absolute timelock
            version: Version::ONE,     // Standard for Dogecoin transactions
        },
    )
}

/// Estimates a reasonable fee rate for Dogecoin transactions based on network conditions.
///
/// This function queries the Dogecoin network for recent fee percentiles and returns
/// the median (50th percentile) fee rate, which provides a good balance between
/// confirmation time and cost. The fee rate is returned in millikoinus per byte.
///
/// On regtest networks (local development), fee data is typically unavailable since
/// there are no standard transactions, so the function falls back to a static rate
/// of 2,000,000 millikoinus/byte (2,000 koinus/B or 0.02 DOGE/kB) which is reasonable for testing.
///
/// # Returns
/// Fee rate in millikoinus per byte (1,000 millikoinus = 1 koinu).
pub async fn get_fee_per_byte(ctx: &DogecoinContext) -> u64 {
    // Query recent fee percentiles from the Dogecoin network.
    // This gives us real-time fee data based on recent transaction activity.
    let fee_percentiles = dogecoin_get_fee_percentiles(&GetCurrentFeePercentilesRequest {
        network: ctx.network.into(),
    })
    .await
    .unwrap();

    if fee_percentiles.is_empty() {
        // Empty percentiles indicate that we're likely on regtest with no standard transactions.
        // Use a reasonable fallback that works for development and testing.
        2_000_000 // 2,000 koinus/B in millikoinus
    } else {
        // Use the 50th percentile (median) for balanced confirmation time and cost.
        // This avoids both overpaying (high percentiles) and slow confirmation (low percentiles).
        fee_percentiles[50]
    }
}

/// Purpose field for BIP-32 hierarchical deterministic wallet derivation paths.
///
/// Dogecoin does not implement the purpose scheme for deterministic wallet. However, for
/// compatibility with Bitcoin, we follow BIP-44 for P2PKH address.
///
/// In BIP-44, the purpose field is a constant set to 44' (or 0x8000002C) for P2PKH addresses.
pub enum Purpose {
    P2PKH,  // BIP-44
}

impl Purpose {
    fn to_u32(&self) -> u32 {
        match self {
            Purpose::P2PKH => 44,
        }
    }
}

/// Represents a complete BIP-32 hierarchical deterministic wallet derivation path.
///
/// The path follows the standard format: m / purpose / coin_type / account / change / address_index
/// This structure enables:
/// - Deterministic key generation from a single seed
/// - Logical separation of different address types and accounts
/// - Privacy through address rotation within accounts
///
/// This implementation supports BIP-44 (P2PKH) and provides serialization compatible with the
/// Internet Computer's key derivation APIs.
///
/// The concept of a wallet derivation path being hardened does not apply on ICP, since key
/// derivation is entirely handled by the subnet and private keys are never accessible. Derivation paths
/// function purely as deterministic identifiers.
pub struct DerivationPath {
    /// Purpose according to BIP-43 (44 for P2PKH)
    purpose: Purpose,

    /// Coin type (0 = Dogecoin mainnet/testnet). Can be extended for altcoins.
    coin_type: u32,

    /// Logical account identifier. Use this to separate multiple user accounts or roles.
    account: u32,

    /// Chain: 0 = external (receive), 1 = internal (change)
    change: u32,

    /// Address index: used to derive multiple addresses within the same account.
    address_index: u32,
}

impl DerivationPath {
    /// Constructs a new derivation path with the specified parameters.
    ///
    /// Parameters:
    /// - `purpose`: Determines the address type and BIP standard to follow
    /// - `account`: Logical account separation (use different accounts for different users/purposes)
    /// - `address_index`: Address index within the account (increment for new addresses)
    ///
    /// Fixed values:
    /// - `coin_type`: Always 0 (Dogecoin mainnet/testnet)
    /// - `change`: Always 0 (external/receiving addresses, not internal change addresses)
    fn new(purpose: Purpose, account: u32, address_index: u32) -> Self {
        Self {
            purpose,
            coin_type: 0,
            account,
            change: 0,
            address_index,
        }
    }

    /// Convenience constructor for P2PKH addresses.
    pub fn p2pkh(account: u32, address_index: u32) -> Self {
        Self::new(Purpose::P2PKH, account, address_index)
    }

    /// Converts the derivation path to the binary format expected by IC's key derivation APIs.
    ///
    /// Returns a Vec<Vec<u8>> where each inner Vec represents one level of the path
    /// as a 4-byte big-endian encoded integer.
    pub fn to_vec_u8_path(&self) -> Vec<Vec<u8>> {
        vec![
            self.purpose.to_u32().to_be_bytes().to_vec(),
            self.coin_type.to_be_bytes().to_vec(),
            self.account.to_be_bytes().to_vec(),
            self.change.to_be_bytes().to_vec(),
            self.address_index.to_be_bytes().to_vec(),
        ]
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "m/{}/{}/{}/{}/{}",
            self.purpose.to_u32(),
            self.coin_type,
            self.account,
            self.change,
            self.address_index
        )
    }
}
