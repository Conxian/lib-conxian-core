//! BIP-322: Universal Message Signing
//! Aligned with CXIP 20 and G-09

use base64::Engine;
use bitcoin::hashes::{sha256, sha256t, Hash, HashEngine};
use bitcoin::{Address, Witness};
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

/// Typed result for the Core BIP-322 boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bip322VerificationError {
    /// The address is not a valid Bitcoin address encoding.
    MalformedAddress,
    /// The signature is not valid base64/witness encoding or contains no items.
    MalformedSignature,
    /// Core has no audited script/signature execution provider.
    Unsupported,
}

impl std::fmt::Display for Bip322VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedAddress => write!(f, "malformed Bitcoin address"),
            Self::MalformedSignature => write!(f, "malformed BIP-322 signature witness"),
            Self::Unsupported => write!(f, "BIP-322 cryptographic verification is unsupported"),
        }
    }
}

impl std::error::Error for Bip322VerificationError {}

impl Bip322Bridge {
    /// Performs structural validation and reports whether an audited verifier
    /// is available. Core does not execute Bitcoin scripts or verify the
    /// witness signature, so structurally valid messages return `Unsupported`.
    pub fn verify_message_checked(msg: &Bip322Message) -> Result<bool, Bip322VerificationError> {
        let _address = Address::from_str(&msg.address)
            .map_err(|_| Bip322VerificationError::MalformedAddress)?;

        if msg.signature.trim().is_empty() {
            return Err(Bip322VerificationError::MalformedSignature);
        }
        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&msg.signature)
            .map_err(|_| Bip322VerificationError::MalformedSignature)?;
        let witness = bitcoin::consensus::encode::deserialize::<Witness>(&signature_bytes)
            .map_err(|_| Bip322VerificationError::MalformedSignature)?;
        if witness.is_empty() {
            return Err(Bip322VerificationError::MalformedSignature);
        }

        Err(Bip322VerificationError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_witness() -> String {
        encoded_witness_with_item(&[0x30, 0x01])
    }

    fn encoded_witness_with_item(item: &[u8]) -> String {
        let mut witness = Witness::new();
        witness.push(item);
        base64::engine::general_purpose::STANDARD.encode(bitcoin::consensus::serialize(&witness))
    }

    #[test]
    fn test_bip322_checked_api_is_unsupported_without_script_verifier() {
        let msg = Bip322Message {
            message: "Hello Conxian".to_string(),
            address: "1BitcoinEaterAddressDontSendf59kuE".to_string(),
            signature: encoded_witness(),
        };
        assert_eq!(
            Bip322Bridge::verify_message_checked(&msg),
            Err(Bip322VerificationError::Unsupported)
        );
    }

    #[test]
    fn test_bip322_mutated_message_remains_unsupported_after_structural_parsing() {
        let msg = Bip322Message {
            message: "Hello Conxian (mutated)".to_string(),
            address: "1BitcoinEaterAddressDontSendf59kuE".to_string(),
            signature: encoded_witness(),
        };

        assert_eq!(
            Bip322Bridge::verify_message_checked(&msg),
            Err(Bip322VerificationError::Unsupported)
        );
    }

    #[test]
    fn test_bip322_mutated_witness_contents_remain_unsupported_after_structural_parsing() {
        let msg = Bip322Message {
            message: "Hello Conxian".to_string(),
            address: "1BitcoinEaterAddressDontSendf59kuE".to_string(),
            signature: encoded_witness_with_item(&[0x31, 0x01]),
        };

        assert_eq!(
            Bip322Bridge::verify_message_checked(&msg),
            Err(Bip322VerificationError::Unsupported)
        );
    }

    #[test]
    fn test_bip322_rejects_invalid_bc1_address() {
        let msg = Bip322Message {
            message: "msg".to_string(),
            address: "bc1not-an-address".to_string(),
            signature: encoded_witness(),
        };
        assert_eq!(
            Bip322Bridge::verify_message_checked(&msg),
            Err(Bip322VerificationError::MalformedAddress)
        );
    }

    #[test]
    fn test_bip322_rejects_malformed_witness_without_fallback() {
        let msg = Bip322Message {
            message: "msg".to_string(),
            address: "1BitcoinEaterAddressDontSendf59kuE".to_string(),
            signature: base64::engine::general_purpose::STANDARD.encode([0xff]),
        };
        assert_eq!(
            Bip322Bridge::verify_message_checked(&msg),
            Err(Bip322VerificationError::MalformedSignature)
        );
    }
}
