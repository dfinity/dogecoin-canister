use std::io::{Cursor, Read};
use bitcoin::{PubkeyHash, PublicKey, ScriptBuf, ScriptHash};
use bitcoin::hashes::Hash;
use crate::blockchain::Blockchain;
use secp256k1::{PublicKey as Secp256k1Pk};
use crate::serialization::{decompress_amount, read_varint};

pub(crate) struct DBUtxoValue {
    pub coinbase: u8,
    pub height: u32,
    pub vout: Option<u32>,
    pub txout: TxOut,
}

#[derive(Debug, Clone)]
pub(crate) struct TxOut {
    pub amount: u64,
    pub script: ScriptBuf,
    pub nsize: usize,
    pub script_type: String,
    pub address: String
}

fn decode_header_code(code: u64) -> (u8, bool, bool, u32) {
    let coinbase = (code & 1) as u8;
    let vout0_unspent = (code & 2) != 0;
    let vout1_unspent = (code & 4) != 0;

    // Number of non-zero bytes in unspentness (nMaskCode in Dogecoin Core)
    let mask_nonzero_bytes = if vout0_unspent || vout1_unspent {
        code / 8
    } else {
        (code / 8) + 1
    };

    (coinbase, vout0_unspent, vout1_unspent, mask_nonzero_bytes as u32)
}

// TODO(XC-503): this could be made more efficient by storing only the output with their associated index that are unspent
fn read_unspentness_mask<R: Read>(reader: &mut R, mask_nonzero_bytes: u32) -> anyhow::Result<Vec<bool>> {
    let mut additional_unspent_outputs = vec![];

    let mut remaining_nonzero = mask_nonzero_bytes;

    while remaining_nonzero > 0 {
        let mut byte_val = [0u8; 1];
        reader.read_exact(&mut byte_val)?;
        let byte_val = byte_val[0];

        for bit_idx in 0..8 {
            let is_unspent = (byte_val & (1 << bit_idx)) != 0;
            additional_unspent_outputs.push(is_unspent);
        }

        // Only decrement counter for non-zero bytes
        if byte_val != 0 {
            remaining_nonzero -= 1;
        }
    }

    if let Some(last_true_pos) = additional_unspent_outputs.iter().rposition(|&x| x) {
        additional_unspent_outputs.truncate(last_true_pos + 1);
    } else {
        additional_unspent_outputs.clear();
    }

    Ok(additional_unspent_outputs)
}

fn deserialize_txout<R: Read>(reader: &mut R, blockchain: &Blockchain) -> anyhow::Result<TxOut> {
    // Examples from Dogecoin Core:

    // 86ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4ee
    // <-------------------------------------------------->
    //                           |
    //                        vout[4]
    //
    // - vout[4]: 86ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4ee
    //            * compressed_amount: 86ef97d579 (decompress: 234925952 = 2.35 DOGE)
    //            * nsize: 00 (P2PKH)
    //            * compressed scriptPubKey: 61b01caab50f1b8e9c50a5057eb43c2d9563a4ee

    // bbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa4
    // <---------------------------------------------->
    //                       |
    //                     vout[16]
    //
    // - vout[16]: bbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa4
    //             * compressed_amount: bbd123 (decompress: 110397 = 0.001 DOGE)
    //             * nsize: 00 (P2PKH)
    //             * compressed scriptPubKey: 8c988f1a4a4de2161e0f50aac7f17e7f9555caa4

    let compressed_amount = read_varint(reader)?;
    let amount = decompress_amount(compressed_amount)?;

    let (script, script_type, nsize, address) = deserialize_script(reader, blockchain)?;

    Ok(TxOut {
        amount,
        nsize,
        script_type,
        script,
        address
    })
}

fn deserialize_script<R: Read>(reader: &mut R, blockchain: &Blockchain) -> anyhow::Result<(ScriptBuf, String, usize, String)> {
    // nsize: byte to indicate the type or size of script
    // nsize  -     compressed script (in DB)    - script
    //   0    -            hash160 PK            - P2PKH
    //   1    -          hash160 script          - P2SH
    //   2    -          compressed PK           - P2PK 02publickey <- compressed PK, y:even - here and following P2PK: nsize makes up part of the compressed PK
    //   3    -          compressed PK           - P2PK 03publickey <- compressed PK, y:odd
    //   4    -          compressed PK           - P2PK 04publickey <- uncompressed PK, y:even
    //   5    -          compressed PK           - P2PK 04publickey <- uncompressed PK, y:odd
    //   6+   -       uncompressed script        - uncompressed script
    // For 6+, nsize is script size (subtract 6 to get the actual size in bytes, to account for the previous 5 script types already taken)
    let nsize = read_varint(reader)? as usize;

    let mut address = String::new();
    let script ;
    let script_type;

    if nsize < 6 {
        // Compressed script: nsize is the compression type (0-5)
        let compressed_data_size = match nsize {
            0 | 1 => 20,         // P2PKH, P2SH: 20 bytes
            2 | 3 | 4 | 5 => 33, // Compressed PK: 33 bytes
            _ => anyhow::bail!("Invalid compression type: {}", nsize),
        };

        let mut compressed_data = vec![0u8; compressed_data_size];

        if nsize > 1 && nsize < 6 {
            // nsize makes up part of the stored PK for P2PK scripts
            compressed_data[0] = nsize as u8;
            reader.read_exact(&mut compressed_data[1..])?;
        } else {
            reader.read_exact(&mut compressed_data)?;
        }

        match nsize {
            0 => {
                let pubkey_hash_bytes: [u8; 20] = compressed_data.try_into().expect("Must be 20 Bytes");
                let pubkey_hash = PubkeyHash::from_byte_array(pubkey_hash_bytes);
                address = blockchain.p2pkh_address(pubkey_hash);
                if blockchain.write_full_script() {
                    script = ScriptBuf::new_p2pkh(&pubkey_hash); // Write the full P2PKH script
                } else {
                    script = ScriptBuf::from_bytes(pubkey_hash_bytes.to_vec()); // Write the PK hash only
                }
                script_type = "p2pkh".to_string();
            },
            1 => {
                let script_hash_bytes: [u8; 20] = compressed_data.try_into().expect("Must be 20 Bytes");
                let script_hash = ScriptHash::from_byte_array(script_hash_bytes);
                address = blockchain.p2sh_address(script_hash);
                if blockchain.write_full_script() {
                    script = ScriptBuf::new_p2sh(&script_hash); // Write the full P2SH script
                } else {
                    script = ScriptBuf::from_bytes(script_hash_bytes.to_vec()); // Write the script hash only
                }
                script_type = "p2sh".to_string();
            },
            2 | 3 => {
                if blockchain.write_full_script() {
                    let pk = PublicKey::from_slice(&compressed_data)?;
                    script = ScriptBuf::new_p2pk(&pk); // Write the full P2PK with compressed PK
                } else {
                    script = ScriptBuf::from_bytes(compressed_data); // Write the compressed PK only
                }
                script_type = "p2pk".to_string();
            },
            4 | 5 => {
                compressed_data[0] -= 2; // 4 indicates y:even -> PK prefix must be 0x02, 5 indicates y:odd -> PK prefix must be 0x03
                let compressed_pk = Secp256k1Pk::from_slice(&compressed_data)?;
                let uncompressed = compressed_pk.serialize_uncompressed();
                let pk = PublicKey::from_slice(&uncompressed)?;

                if blockchain.write_full_script() {
                    script = ScriptBuf::new_p2pk(&pk); // Write the full P2PK script with uncompressed PK
                } else {
                    script = ScriptBuf::from_bytes(uncompressed.to_vec()); // Write the uncompressed PK only
                }
                script_type = "p2pk".to_string();
            },
            _ => unreachable!(),
        }
    } else {
        // Regular script: actual script size = nsize - 6
        let script_size = nsize - 6;
        let mut script_bytes = vec![0u8; script_size];
        reader.read_exact(&mut script_bytes)?;
        if script_size >= 36 && script_bytes.last() == Some(&174) { // 174 = 0xae = OP_CHECKMULTISIG
            script_type = "p2ms".to_string();
        } else {
            script_type = "non-standard".to_string();
        }
        script = ScriptBuf::from_bytes(script_bytes);
    };

    Ok((script, script_type, nsize, address))
}

pub(crate) fn deserialize_db_utxo_legacy(blockchain: &Blockchain, value: Vec<u8>) -> anyhow::Result<Vec<DBUtxoValue>> {
    let mut cursor = Cursor::new(value);

    // -------------------
    // UTXO Value (legacy)
    // -------------------

    // Ref: <https://en.bitcoin.it/wiki/Bitcoin_Core_0.11_(ch_2):_Data_Storage>
    //      <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L74>
    //      <https://github.com/dogecoin/dogecoin/blob/7dac1e5e9e887f5f6ff146e812a05bd3bf281eae/src/coins.h#L156>

    // Example: 0109044086ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4eebbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa486af3b <- deobfuscated value
    //          <><><--><--------------------------------------------------><----------------------------------------------><---->
    //         /   |    \                       |                                                  |                          |
    //     varint varint bitvector            vout[4]                                            vout[14]                   varint
    //       |     |       \                                                                                                  |
    //    version code     unspentness                                                                                      height
    //
    // - version = 1
    // - code = 9 (coinbase, neither vout[0] or vout[1] are unspent,
    //             2 (1, +1 because both bit 1 and bit 2 are unset) non-zero bitvector bytes follow)
    // - unspentness bitvector: bits 2 (0x04) and 14 (0x4000) are set, so vout[2+2] and vout[14+2] are unspent
    // - height = 120891

    // The code value consists of:
    //    - bit 0: IsCoinBase()
    //    - bit 1: vout[0] is not spent
    //    - bit 2: vout[1] is not spent
    //    - The higher bits encode N, the number of non-zero bytes in the following bitvector.
    //    - In case both bit 1 and bit 2 are unset, they encode N-1, as there must be at
    //      least one non-spent output.

    // First varint (version)
    let _version = read_varint(&mut cursor)?;

    // Second varint (code)
    let code = read_varint(&mut cursor)?;
    let (coinbase, vout0_unspent, vout1_unspent, mask_nonzero_bytes) = decode_header_code(code);

    let mut unspent_outputs = vec![vout0_unspent, vout1_unspent];

    if mask_nonzero_bytes > 0 {
        // Bitvector (unspentness)
        let additional_unspent_outputs = read_unspentness_mask(&mut cursor, mask_nonzero_bytes)?;
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

    for (vout, out) in outputs.into_iter().enumerate() {
        if let Some(txout) = out {
            let db_output = DBUtxoValue {
                coinbase,
                height,
                vout: Some(vout as u32),
                txout
            };
            db_outputs.push(db_output);
        }
    }

    Ok(db_outputs)
}

pub(crate) fn deserialize_db_utxo_modern(blockchain: &Blockchain, value: Vec<u8>) -> anyhow::Result<Vec<DBUtxoValue>> {
    let mut cursor = Cursor::new(value);

    // -------------------
    // UTXO Value (modern)
    // -------------------

    //          c0842680ed5900a38f35518de4487c108e3810e6794fb68b189d8b <- deobfuscated value
    //          <----><----><><-------------------------------------->
    //           /      |    \                   |
    //      varint   varint   varint          script <- P2PKH/P2SH hash160, P2PK public key, or complete script
    //         |        |     n_size
    //         |        |
    //         |     amount (compressed)
    //         |
    //  100000100001010100110
    //  <------------------> \
    //         height         coinbase

    // First varint (height, coinbase)
    let height_coinbase = read_varint(&mut cursor)?;

    let height = (height_coinbase >> 1) as u32;
    let coinbase = (height_coinbase & 1) as u8;

    // Second varint (amount compressed)
    let compressed_amount = read_varint(&mut cursor)?;
    let amount = decompress_amount(compressed_amount)?;

    let (script, script_type, nsize, address) = deserialize_script(&mut cursor, blockchain)?;

    let txout = TxOut {
        amount,
        nsize,
        script_type,
        script,
        address
    };

    Ok(vec![DBUtxoValue {
        coinbase,
        height,
        vout: None, // Encoded in the key, not in the value
        txout,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_code_decoding() {
        let (coinbase, vout0, vout1, mask_bytes) = decode_header_code(4);
        assert_eq!(coinbase, 0);
        assert_eq!(vout0, false);
        assert_eq!(vout1, true);
        assert_eq!(mask_bytes, 0);

        let (coinbase, vout0, vout1, mask_bytes) = decode_header_code(9);
        assert_eq!(coinbase, 1);
        assert_eq!(vout0, false);
        assert_eq!(vout1, false);
        assert_eq!(mask_bytes, 2);
    }

    #[test]
    fn test_unspentness_mask_reading() {
        let data = vec![0x05, 0x00, 0x01]; // Bits 0 and 2 in byte 0, bit 0 in byte 2
        let mut cursor = Cursor::new(&data);
        let unspent_outputs = read_unspentness_mask(&mut cursor, 2).unwrap();

        // Should only include up to the last set bit (index 16)
        assert_eq!(unspent_outputs.len(), 17); // 0-16 inclusive
        assert_eq!(unspent_outputs[0], true);  // vout[2 + 0]
        assert_eq!(unspent_outputs[1], false); // vout[2 + 1]
        assert_eq!(unspent_outputs[2], true);  // vout[2 + 2]
        assert_eq!(unspent_outputs[16], true); // vout[16 + 2]
    }
}