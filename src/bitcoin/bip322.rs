//! BIP-322: Universal Message Signing
//! Aligned with CXIP 20 and G-09

use base64::Engine;
use bitcoin::{Address, Witness};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bip322Message {
    pub message: String,
    pub address: String,
    pub signature: String,
}

pub struct Bip322Bridge;

/// Fail-closed errors returned while parsing or classifying a BIP-322 input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bip322Error {
    InvalidAddress,
    InvalidSignatureEncoding,
    InvalidWitness,
    UnsupportedScriptType,
}

impl fmt::Display for Bip322Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid BIP-322 address"),
            Self::InvalidSignatureEncoding => write!(f, "invalid BIP-322 signature encoding"),
            Self::InvalidWitness => write!(f, "invalid BIP-322 witness"),
            Self::UnsupportedScriptType => write!(f, "unsupported BIP-322 script type"),
        }
    }
}

impl std::error::Error for Bip322Error {}

impl Bip322Bridge {
    /// Compatibility wrapper for callers that only accept a boolean.
    ///
    /// Any parse failure or unsupported script type is reported as `false`.
    /// Call [`Self::try_verify_message`] when the failure class is needed.
    pub fn verify_message(msg: &Bip322Message) -> bool {
        Self::try_verify_message(msg).is_ok_and(|verified| verified)
    }

    /// Parses BIP-322 input strictly, then returns typed unsupported because
    /// this crate intentionally exposes no script type until a real audited
    /// script/witness verifier is wired in.
    ///
    /// In particular, this method never substitutes a non-empty witness or a
    /// `bc1` prefix for signature verification.
    pub fn try_verify_message(msg: &Bip322Message) -> Result<bool, Bip322Error> {
        let address = Address::from_str(&msg.address)
            .map_err(|_| Bip322Error::InvalidAddress)?
            .assume_checked();

        let signature_bytes = base64::engine::general_purpose::STANDARD
            .decode(&msg.signature)
            .map_err(|_| Bip322Error::InvalidSignatureEncoding)?;
        if base64::engine::general_purpose::STANDARD.encode(&signature_bytes) != msg.signature {
            return Err(Bip322Error::InvalidSignatureEncoding);
        }

        let witness = bitcoin::consensus::encode::deserialize::<Witness>(&signature_bytes)
            .map_err(|_| Bip322Error::InvalidWitness)?;
        if witness.is_empty() || bitcoin::consensus::encode::serialize(&witness) != signature_bytes
        {
            return Err(Bip322Error::InvalidWitness);
        }

        // Force address/script parsing before classifying the script as
        // unsupported. No script type is claimed as verified by this crate.
        let _script_pubkey = address.script_pubkey();
        let _ = msg.message.as_bytes();
        Err(Bip322Error::UnsupportedScriptType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip322_verification_is_fail_closed() {
        let witness = Witness::from_slice(&[vec![0u8; 64]]);
        let msg = Bip322Message {
            message: "Hello Conxian".to_string(),
            address: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
            signature: base64::engine::general_purpose::STANDARD
                .encode(bitcoin::consensus::encode::serialize(&witness)),
        };
        assert_eq!(
            Bip322Bridge::try_verify_message(&msg),
            Err(Bip322Error::UnsupportedScriptType)
        );
        assert!(!Bip322Bridge::verify_message(&msg));
    }

    #[test]
    fn test_bip322_valid_segwit_address_does_not_use_prefix_fallback() {
        // This is a canonically encoded, structurally valid SegWit witness
        // stack. Its presence must not turn the valid `bc1` prefix into a
        // successful verification result.
        let witness = Witness::from_slice(&[vec![0u8; 64], vec![0x02; 33]]);
        let encoded_witness = bitcoin::consensus::encode::serialize(&witness);
        let msg = Bip322Message {
            message: "Hello Conxian".to_string(),
            address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            signature: base64::engine::general_purpose::STANDARD.encode(encoded_witness),
        };

        assert_eq!(
            Bip322Bridge::try_verify_message(&msg),
            Err(Bip322Error::UnsupportedScriptType)
        );
        assert!(!Bip322Bridge::verify_message(&msg));
    }

    #[test]
    fn test_bip322_invalid_address() {
        let msg = Bip322Message {
            message: "msg".to_string(),
            address: "not-an-address".to_string(),
            signature: "sig".to_string(),
        };
        assert!(!Bip322Bridge::verify_message(&msg));
        assert_eq!(
            Bip322Bridge::try_verify_message(&msg),
            Err(Bip322Error::InvalidAddress)
        );
    }

    #[test]
    fn test_bip322_rejects_malformed_encoding_and_witness() {
        let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let malformed_encoding = Bip322Message {
            message: "msg".to_string(),
            address: address.to_string(),
            signature: "not-base64".to_string(),
        };
        assert_eq!(
            Bip322Bridge::try_verify_message(&malformed_encoding),
            Err(Bip322Error::InvalidSignatureEncoding)
        );

        let malformed_witness = Bip322Message {
            message: "msg".to_string(),
            address: address.to_string(),
            signature: base64::engine::general_purpose::STANDARD.encode([1u8, 2, 3]),
        };
        assert_eq!(
            Bip322Bridge::try_verify_message(&malformed_witness),
            Err(Bip322Error::InvalidWitness)
        );

        let non_canonical_encoding = Bip322Message {
            message: "msg".to_string(),
            address: address.to_string(),
            signature: "AQI".to_string(),
        };
        assert_eq!(
            Bip322Bridge::try_verify_message(&non_canonical_encoding),
            Err(Bip322Error::InvalidSignatureEncoding)
        );
    }
}
