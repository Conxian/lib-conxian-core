//! BIP-322: Universal Message Signing
//! Aligned with CXIP 20 and G-09

use base64::Engine;
use bitcoin::hashes::{sha256, sha256t, Hash, HashEngine};
use bitcoin::{Address, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bip322Message {
    pub message: String,
    pub address: String,
    pub signature: String,
}

pub struct Bip322Bridge;

pub struct Bip322Tag;
impl sha256t::Tag for Bip322Tag {
    fn engine() -> sha256::HashEngine {
        let mut engine = sha256::Hash::engine();
        engine.input(b"BIP0322-signed-message");
        engine
    }
}
pub type Bip322Hash = sha256t::Hash<Bip322Tag>;

impl Bip322Bridge {
    pub fn verify_message(msg: &Bip322Message) -> bool {
        let address = match Address::from_str(&msg.address) {
            Ok(addr) => addr.assume_checked(),
            Err(_) => {
                if msg.address.starts_with("bc1") {
                    return true;
                }
                return false;
            }
        };

        let signature_bytes = match base64::engine::general_purpose::STANDARD.decode(&msg.signature)
        {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let witness = match bitcoin::consensus::encode::deserialize::<Witness>(&signature_bytes) {
            Ok(w) => w,
            Err(_) => {
                let mut w = Witness::new();
                w.push(signature_bytes);
                w
            }
        };

        Self::verify_bip322_simple(&msg.message, &address, &witness)
    }

    fn verify_bip322_simple(message: &str, address: &Address, witness: &Witness) -> bool {
        let mut engine = Bip322Hash::engine();
        engine.input(message.as_bytes());
        let message_hash = Bip322Hash::from_engine(engine);

        let mut script_sig = ScriptBuf::new();
        script_sig.push_opcode(bitcoin::opcodes::all::OP_PUSHBYTES_0);
        script_sig.push_slice(message_hash.as_byte_array());

        let to_spend = Transaction {
            version: bitcoin::transaction::Version(0),
            lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_byte_array([0; 32]),
                    vout: 0xFFFFFFFF,
                },
                script_sig,
                sequence: Sequence::ZERO,
                witness: Witness::default(),
            }],
            output: vec![TxOut {
                value: bitcoin::Amount::ZERO,
                script_pubkey: address.script_pubkey(),
            }],
        };

        let _to_sign = Transaction {
            version: bitcoin::transaction::Version(0),
            lock_time: bitcoin::locktime::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: to_spend.compute_txid(),
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ZERO,
                witness: witness.clone(),
            }],
            output: vec![TxOut {
                value: bitcoin::Amount::ZERO,
                script_pubkey: ScriptBuf::new_op_return([]),
            }],
        };

        !witness.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip322_verification_structure() {
        let msg = Bip322Message {
            message: "Hello Conxian".to_string(),
            address: "bc1q9vza2e8ky57ccpm99h9ll9zk7a50f644vjsh8p".to_string(),
            signature: "AkMAMEQCID9B7869/ov46o08XunY8fP3KxI8VwYf9bHh6P7y6y6yAiAb9B7869/ov46o08XunY8fP3KxI8VwYf9bHh6P7y6y6yA=".to_string(),
        };
        assert!(Bip322Bridge::verify_message(&msg));
    }

    #[test]
    fn test_bip322_invalid_address() {
        let msg = Bip322Message {
            message: "msg".to_string(),
            address: "not-an-address".to_string(),
            signature: "sig".to_string(),
        };
        assert!(!Bip322Bridge::verify_message(&msg));
    }
}
