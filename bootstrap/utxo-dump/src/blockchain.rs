use bitcoin::{PubkeyHash, ScriptHash};
use crate::chainstate::{deserialize_db_utxo_legacy, deserialize_db_utxo_modern, DBUtxoValue};
use bitcoin::{Address as BtcAddress, dogecoin::Address as DogeAddress, Network as BtcNetwork, dogecoin::Network as DogeNetwork};

const DB_KEYS_UTXOS: char = 'C'; // 0x43 = 67
const DB_KEYS_UTXOS_LEGACY: char = 'c'; // 0x63 = 99

#[derive(Debug)]
pub(crate) enum Blockchain {
    Bitcoin(BtcNetwork),
    Dogecoin(DogeNetwork),
}

impl Blockchain {
    pub(crate) fn utxo_key_prefix(&self) -> u8 {
        match self {
            Blockchain::Bitcoin(_) => DB_KEYS_UTXOS as u8,
            Blockchain::Dogecoin(_) => DB_KEYS_UTXOS_LEGACY as u8,
        }
    }

    pub(crate) fn ticker(&self) -> &str {
        match self {
            Blockchain::Bitcoin(_) => "BTC",
            Blockchain::Dogecoin(_) => "DOGE",
        }
    }

    pub(crate) fn p2pkh_address(&self, pubkey_hash: PubkeyHash) -> String {
        match self {
            Blockchain::Bitcoin(network) => {
                BtcAddress::p2pkh(pubkey_hash, *network).to_string()
            }
            Blockchain::Dogecoin(network) => {
                DogeAddress::p2pkh(pubkey_hash, *network).to_string()
            }
        }
    }

    pub(crate) fn p2sh_address(&self, script_hash: ScriptHash) -> String {
        match self {
            Blockchain::Bitcoin(network) => {
                BtcAddress::p2sh_from_hash(script_hash, *network).to_string()
            }
            Blockchain::Dogecoin(network) => {
                DogeAddress::p2sh_from_hash(script_hash, *network).to_string()
            }
        }
    }

    pub(crate) fn deserialize_db_utxo(&self, value: Vec<u8>) -> anyhow::Result<Vec<DBUtxoValue>> {
        match self {
            Blockchain::Bitcoin(_) => {
                Ok(deserialize_db_utxo_modern(self, value)?)
            }
            Blockchain::Dogecoin(_) => {
                Ok(deserialize_db_utxo_legacy(self, value)?)
            }
        }
    }
}