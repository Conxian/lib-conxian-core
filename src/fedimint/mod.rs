//! Fedimint deterministic primitives and provider boundary
//! Aligned with CXIP 20 and G-16

use secp256k1::{PublicKey, Scalar, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FedimintMint {
    pub mint_id: String,
    pub community_name: String,
    pub total_liquidity_sats: u64,
}

pub struct FedimintAdapter;

/// Typed failures for Fedimint note construction and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FedimintError {
    /// A mint identifier is empty or contains only whitespace.
    MalformedMintId,
    /// Authenticated provider-backed mint status is not available in Core.
    StatusUnavailable,
    /// A byte input does not have the required exact length.
    InvalidLength {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// An input cannot represent a valid secp256k1 scalar.
    InvalidScalar(&'static str),
    /// An input cannot represent a valid compressed secp256k1 point.
    InvalidPoint,
    /// A required secret/evidence input is empty.
    EmptyInput(&'static str),
}

impl std::fmt::Display for FedimintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedMintId => write!(f, "mint ID must not be empty"),
            Self::StatusUnavailable => {
                write!(
                    f,
                    "authenticated Fedimint mint status is unavailable in Core"
                )
            }
            Self::InvalidLength {
                field,
                expected,
                actual,
            } => write!(
                f,
                "invalid {field} length: expected {expected}, got {actual}"
            ),
            Self::InvalidScalar(field) => write!(f, "invalid scalar for {field}"),
            Self::InvalidPoint => write!(f, "invalid compressed secp256k1 point"),
            Self::EmptyInput(field) => write!(f, "{field} must not be empty"),
        }
    }
}

impl std::error::Error for FedimintError {}

impl FedimintAdapter {
    /// Returns authenticated provider-backed status for a mint.
    ///
    /// Core validates the identifier shape but does not have a Fedimint
    /// provider. It therefore returns a typed unavailable error for every
    /// non-empty identifier instead of fabricating community or liquidity
    /// data.
    pub fn get_mint_status(&self, mint_id: &str) -> Result<FedimintMint, FedimintError> {
        if mint_id.trim().is_empty() {
            return Err(FedimintError::MalformedMintId);
        }

        Err(FedimintError::StatusUnavailable)
    }

    /// Performs deterministic secp256k1 point reconstruction for an e-cash
    /// note primitive (G-16). This is not provider-backed mint verification.
    /// Uses ECC point addition: blinded_note = H(secret)*G + r*G
    pub fn blind_note(secret: &[u8], blinding_factor: &[u8]) -> Result<Vec<u8>, FedimintError> {
        let secp = Secp256k1::new();

        if secret.is_empty() {
            return Err(FedimintError::EmptyInput("secret"));
        }
        if blinding_factor.len() != 32 {
            return Err(FedimintError::InvalidLength {
                field: "blinding factor",
                expected: 32,
                actual: blinding_factor.len(),
            });
        }

        let mut hasher = Sha256::new();
        hasher.update(b"FEDIMINT-SECRET");
        hasher.update(secret);
        let secret_hash: [u8; 32] = hasher.finalize().into();

        let secret_scalar = match Scalar::from_be_bytes(secret_hash) {
            Ok(s) => s,
            Err(_) => return Err(FedimintError::InvalidScalar("secret hash")),
        };

        let bf_bytes: [u8; 32] =
            blinding_factor
                .try_into()
                .map_err(|_| FedimintError::InvalidLength {
                    field: "blinding factor",
                    expected: 32,
                    actual: blinding_factor.len(),
                })?;
        if bf_bytes.iter().all(|byte| *byte == 0) {
            return Err(FedimintError::InvalidScalar("blinding factor"));
        }

        let bf_scalar = match Scalar::from_be_bytes(bf_bytes) {
            Ok(s) => s,
            Err(_) => return Err(FedimintError::InvalidScalar("blinding factor")),
        };

        // note_point = secret * G
        let sk = match SecretKey::from_byte_array(secret_scalar.to_be_bytes()) {
            Ok(k) => k,
            Err(_) => return Err(FedimintError::InvalidScalar("secret hash")),
        };
        let note_point = PublicKey::from_secret_key(&secp, &sk);

        // blinded_point = note_point + bf * G
        let blinded_point = match note_point.add_exp_tweak(&secp, &bf_scalar) {
            Ok(p) => p,
            Err(_) => return Err(FedimintError::InvalidPoint),
        };

        Ok(blinded_point.serialize().to_vec())
    }

    /// Verifies the unblinding of an e-cash note with real point equality.
    pub fn verify_unblinded_checked(
        blinded: &[u8],
        blinding_factor: &[u8],
        secret: &[u8],
    ) -> Result<bool, FedimintError> {
        if blinded.len() != 33 {
            return Err(FedimintError::InvalidLength {
                field: "blinded note",
                expected: 33,
                actual: blinded.len(),
            });
        }
        PublicKey::from_slice(blinded).map_err(|_| FedimintError::InvalidPoint)?;
        let reconstructed = Self::blind_note(secret, blinding_factor)?;
        Ok(reconstructed == blinded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fedimint_blinding_determinism() {
        let secret = b"my-ecash-secret-32-bytes-long-now";
        let mut bf = [0u8; 32];
        bf[31] = 1;

        let blinded1 = FedimintAdapter::blind_note(secret, &bf).unwrap();
        let blinded2 = FedimintAdapter::blind_note(secret, &bf).unwrap();

        assert_eq!(blinded1, blinded2);
        assert!(FedimintAdapter::verify_unblinded_checked(&blinded1, &bf, secret).unwrap());
        let mut mutated = blinded1.clone();
        mutated[0] ^= 1;
        assert_eq!(
            FedimintAdapter::verify_unblinded_checked(&mutated, &bf, secret),
            Ok(false)
        );
    }

    #[test]
    fn test_fedimint_rejects_malformed_blinding_inputs() {
        let secret = b"my-ecash-secret";
        assert!(matches!(
            FedimintAdapter::blind_note(secret, &[0x01; 31]),
            Err(FedimintError::InvalidLength { .. })
        ));
        assert_eq!(
            FedimintAdapter::blind_note(secret, &[0x00; 32]),
            Err(FedimintError::InvalidScalar("blinding factor"))
        );
        assert!(matches!(
            FedimintAdapter::verify_unblinded_checked(&[0x00; 33], &[0x01; 32], secret),
            Err(FedimintError::InvalidPoint)
        ));
        assert!(matches!(
            FedimintAdapter::verify_unblinded_checked(&[0x01; 32], &[0x01; 32], secret),
            Err(FedimintError::InvalidLength { .. })
        ));
    }

    #[test]
    fn test_fedimint_mint_status_rejects_empty_id() {
        let adapter = FedimintAdapter;

        assert_eq!(
            adapter.get_mint_status(" \t\n").unwrap_err(),
            FedimintError::MalformedMintId
        );
    }

    #[test]
    fn test_fedimint_mint_status_requires_authenticated_provider() {
        let adapter = FedimintAdapter;

        assert_eq!(
            adapter.get_mint_status("fedimint://community").unwrap_err(),
            FedimintError::StatusUnavailable
        );
    }
}
