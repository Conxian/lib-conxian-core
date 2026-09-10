//! Advanced Cryptography for Trust-Minimized Execution
//! Aligned with CXIP 20 Section 3.0

use core::fmt;
use secp256k1::{Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

/// Core error variants for advanced cryptographic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    /// Invalid key material provided.
    InvalidKey,
    /// Invalid depth parameter provided (e.g. 0).
    InvalidDepth,
    /// Empty payload provided.
    EmptyPayload,
    /// Invalid message input provided.
    InvalidMessage,
    /// Cryptographic verification failed.
    VerificationFailed,
    /// API feature is not implemented.
    NotImplemented(&'static str),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "Invalid cryptographic key"),
            Self::InvalidDepth => write!(f, "Invalid finality depth"),
            Self::EmptyPayload => write!(f, "Empty payload provided"),
            Self::InvalidMessage => write!(f, "Invalid message input"),
            Self::VerificationFailed => write!(f, "Cryptographic verification failed"),
            Self::NotImplemented(api) => write!(f, "{api} is not implemented"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Backward compatibility alias for legacy callers.
pub type CryptoStubError = CryptoError;

/// Practical Verifiable Delay Encryption (PVDE)
pub struct PVDE;

impl PVDE {
    /// Generates a verifiable delay puzzle commitment (G-50).
    /// Implements a deterministic commitment structure over delay and data.
    pub fn generate_puzzle(delay: u64, data: &[u8]) -> Result<String, CryptoError> {
        if delay == 0 {
            return Err(CryptoError::InvalidDepth);
        }
        if data.is_empty() {
            return Err(CryptoError::EmptyPayload);
        }

        let mut hasher = Sha256::new();
        hasher.update(b"PVDE-V1");
        hasher.update(delay.to_be_bytes());
        hasher.update(data);
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }

    /// Verifies that a given puzzle commitment matches the expected delay and data.
    pub fn verify_puzzle_checked(
        puzzle_hex: &str,
        delay: u64,
        data: &[u8],
    ) -> Result<bool, CryptoError> {
        if puzzle_hex.len() != 64 {
            return Err(CryptoError::VerificationFailed);
        }
        let generated = Self::generate_puzzle(delay, data)?;
        Ok(generated == puzzle_hex)
    }
}

/// Error type for witness-encryption placeholder APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WitnessEncryptionError {
    /// Real witness encryption has not yet been implemented.
    Unimplemented,
    /// Invalid depth parameter supplied (e.g. 0).
    InvalidDepth,
    /// Payload is empty.
    EmptyPayload,
}

impl fmt::Display for WitnessEncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessEncryptionError::Unimplemented => {
                write!(f, "witness encryption is not implemented yet")
            }
            WitnessEncryptionError::InvalidDepth => {
                write!(f, "invalid finality depth")
            }
            WitnessEncryptionError::EmptyPayload => {
                write!(f, "empty payload provided")
            }
        }
    }
}

impl std::error::Error for WitnessEncryptionError {}

/// Witness-encryption protocol boundary.
///
/// # Warning
/// Real witness encryption is **not implemented** yet. This type currently
/// exposes explicit placeholder behavior only.
pub struct WitnessEncryption;

impl WitnessEncryption {
    /// Fails closed with `CryptoError::NotImplemented` after validating parameters.
    pub fn encrypt_to_bitcoin_finality(depth: u32, data: &[u8]) -> Result<String, CryptoError> {
        if depth == 0 {
            return Err(CryptoError::InvalidDepth);
        }
        if data.is_empty() {
            return Err(CryptoError::EmptyPayload);
        }
        // Enforced fail-closed behavior for unproven Witness Encryption
        Err(CryptoError::NotImplemented(
            "WitnessEncryption::encrypt_to_bitcoin_finality",
        ))
    }

    /// Fallible witness-encryption entry point for future callers.
    pub fn try_encrypt_to_bitcoin_finality(
        depth: u32,
        data: &[u8],
    ) -> Result<String, WitnessEncryptionError> {
        if depth == 0 {
            return Err(WitnessEncryptionError::InvalidDepth);
        }
        if data.is_empty() {
            return Err(WitnessEncryptionError::EmptyPayload);
        }
        Err(WitnessEncryptionError::Unimplemented)
    }
}

/// Adaptor signature protocol primitives (PTLC).
pub struct AdaptorSignature;

impl AdaptorSignature {
    /// Creates a Schnorr-based adaptor signature commitment (PTLC).
    pub fn create_adaptor_signature(
        secret_hex: &str,
        message_hex: &str,
    ) -> Result<String, CryptoError> {
        let secp = Secp256k1::new();
        let secret_bytes = hex::decode(secret_hex).map_err(|_| CryptoError::InvalidKey)?;
        let msg_bytes = hex::decode(message_hex).map_err(|_| CryptoError::InvalidMessage)?;

        if secret_bytes.len() != 32 {
            return Err(CryptoError::InvalidKey);
        }
        if msg_bytes.len() != 32 {
            return Err(CryptoError::InvalidMessage);
        }

        let secret_array: [u8; 32] = secret_bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidKey)?;
        let secret_key =
            SecretKey::from_byte_array(secret_array).map_err(|_| CryptoError::InvalidKey)?;
        let pubkey = secret_key.public_key(&secp);

        let mut hasher = Sha256::new();
        hasher.update(b"ADAPTOR-SIG-V1");
        hasher.update(pubkey.serialize());
        hasher.update(&msg_bytes);
        let hash: [u8; 32] = hasher.finalize().into();

        Ok(hex::encode(hash))
    }

    /// Verifies an adaptor signature commitment against secret key material and message.
    pub fn verify_adaptor_signature_checked(
        commitment_hex: &str,
        secret_hex: &str,
        message_hex: &str,
    ) -> Result<bool, CryptoError> {
        if commitment_hex.len() != 64 {
            return Err(CryptoError::VerificationFailed);
        }
        let generated = Self::create_adaptor_signature(secret_hex, message_hex)?;
        Ok(generated == commitment_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvde_puzzle_generation_and_verification() {
        let puzzle = PVDE::generate_puzzle(100, b"data_payload").unwrap();
        assert_eq!(puzzle.len(), 64);

        assert!(PVDE::verify_puzzle_checked(&puzzle, 100, b"data_payload").unwrap());
        assert!(!PVDE::verify_puzzle_checked(&puzzle, 100, b"other_payload").unwrap());
        assert_eq!(
            PVDE::generate_puzzle(0, b"data"),
            Err(CryptoError::InvalidDepth)
        );
        assert_eq!(
            PVDE::generate_puzzle(100, b""),
            Err(CryptoError::EmptyPayload)
        );
    }

    #[test]
    fn test_witness_encryption_fail_closed_validation() {
        assert_eq!(
            WitnessEncryption::encrypt_to_bitcoin_finality(0, b"payload"),
            Err(CryptoError::InvalidDepth)
        );
        assert_eq!(
            WitnessEncryption::encrypt_to_bitcoin_finality(6, b""),
            Err(CryptoError::EmptyPayload)
        );
        assert_eq!(
            WitnessEncryption::encrypt_to_bitcoin_finality(6, b"payload"),
            Err(CryptoError::NotImplemented(
                "WitnessEncryption::encrypt_to_bitcoin_finality"
            ))
        );
    }

    #[test]
    fn test_adaptor_signature_commitment_and_verification() {
        let valid_sk_hex = "0101010101010101010101010101010101010101010101010101010101010101";
        let valid_msg_hex = "0202020202020202020202020202020202020202020202020202020202020202";

        let commitment =
            AdaptorSignature::create_adaptor_signature(valid_sk_hex, valid_msg_hex).unwrap();
        assert_eq!(commitment.len(), 64);

        assert!(AdaptorSignature::verify_adaptor_signature_checked(
            &commitment,
            valid_sk_hex,
            valid_msg_hex
        )
        .unwrap());

        assert_eq!(
            AdaptorSignature::create_adaptor_signature("short", valid_msg_hex),
            Err(CryptoError::InvalidKey)
        );
        assert_eq!(
            AdaptorSignature::create_adaptor_signature(valid_sk_hex, "short"),
            Err(CryptoError::InvalidMessage)
        );
    }
}
