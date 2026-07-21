//! FROST: Flexible Round-Optimized Schnorr Threshold Signatures.
//!
//! This module keeps the protocol data shapes used by the core API, but does
//! not claim to implement FROST. Production FROST DKG and signing belong in
//! the audited enclave SDK. Every authorization-shaped operation therefore
//! fails closed rather than emitting fabricated key material or signatures.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Typed failures returned by the unsupported core FROST boundary.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FrostError {
    InvalidParameters { reason: String },
    InvalidShare { reason: String },
    InsufficientShares { required: u32, provided: usize },
    Unsupported { operation: String, reason: String },
}

impl fmt::Display for FrostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidParameters { reason } => write!(f, "invalid FROST parameters: {reason}"),
            Self::InvalidShare { reason } => write!(f, "invalid FROST share: {reason}"),
            Self::InsufficientShares { required, provided } => write!(
                f,
                "insufficient FROST shares: required {required}, provided {provided}"
            ),
            Self::Unsupported { operation, reason } => {
                write!(f, "unsupported FROST operation {operation}: {reason}")
            }
        }
    }
}

impl std::error::Error for FrostError {}

/// Represents a secret key share in the FROST threshold scheme.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostKeyShare {
    /// Participant index (1-indexed).
    pub index: u32,
    /// Secret share of the group key.
    pub share: Vec<u8>,
    /// Public key corresponding to this share.
    pub public_key: Vec<u8>,
}

/// A commitment to a polynomial used in VSS (Verifiable Secret Sharing).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostShareCommitment {
    pub index: u32,
    pub commitment_points: Vec<Vec<u8>>,
}

/// An encrypted share for distribution during Round 2.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedFrostShare {
    pub from_index: u32,
    pub to_index: u32,
    pub encrypted_payload: Vec<u8>,
    pub mac: [u8; 32],
}

/// Status of a FROST signing session.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FrostSessionStatus {
    Open,
    Committed,
    Signed,
    Aborted,
}

/// Manager for the FROST protocol boundary.
pub struct FrostManager;

impl FrostManager {
    /// Returns typed unsupported instead of fabricating shares or commitments.
    pub fn generate_shares(
        threshold: u32,
        total: u32,
    ) -> Result<(Vec<FrostKeyShare>, Vec<FrostShareCommitment>), FrostError> {
        if threshold == 0 || total == 0 || threshold > total {
            return Err(FrostError::InvalidParameters {
                reason: "threshold and total must be non-zero, with threshold <= total".to_string(),
            });
        }

        Err(FrostError::Unsupported {
            operation: "generate_shares".to_string(),
            reason: "standards-compliant FROST DKG is owned by the audited enclave SDK".to_string(),
        })
    }

    /// Returns typed unsupported instead of using XOR or an unauthenticated
    /// digest as share encryption/authentication.
    pub fn prepare_distribution_shares(
        from_share: &FrostKeyShare,
        target_indices: &[u32],
        shared_secrets: &[(u32, [u8; 32])],
    ) -> Result<Vec<EncryptedFrostShare>, FrostError> {
        if from_share.index == 0 || from_share.share.len() != 32 {
            return Err(FrostError::InvalidShare {
                reason: "a FROST scalar share must be exactly 32 bytes".to_string(),
            });
        }
        if target_indices.is_empty() {
            return Err(FrostError::InvalidParameters {
                reason: "at least one target participant is required".to_string(),
            });
        }
        if target_indices
            .iter()
            .any(|index| *index == 0 || !shared_secrets.iter().any(|(known, _)| known == index))
        {
            return Err(FrostError::InvalidParameters {
                reason: "every target participant must have a non-zero shared secret".to_string(),
            });
        }

        Err(FrostError::Unsupported {
            operation: "prepare_distribution_shares".to_string(),
            reason: "FROST share encryption and authentication are not implemented in core"
                .to_string(),
        })
    }

    /// Returns typed unsupported instead of performing byte-wise signature
    /// aggregation that is not Schnorr scalar arithmetic.
    pub fn aggregate_signature(shares: &[Vec<u8>], threshold: u32) -> Result<Vec<u8>, FrostError> {
        if threshold == 0 {
            return Err(FrostError::InvalidParameters {
                reason: "threshold must be non-zero".to_string(),
            });
        }
        if shares.len() < threshold as usize {
            return Err(FrostError::InsufficientShares {
                required: threshold,
                provided: shares.len(),
            });
        }
        if shares.iter().any(|share| share.len() != 64) {
            return Err(FrostError::InvalidShare {
                reason: "each partial Schnorr signature must be exactly 64 bytes".to_string(),
            });
        }

        Err(FrostError::Unsupported {
            operation: "aggregate_signature".to_string(),
            reason: "FROST nonce binding, participant checks, and scalar aggregation are not"
                .to_string()
                + " implemented in core",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frost_generation_is_typed_unsupported() {
        assert!(matches!(
            FrostManager::generate_shares(2, 3),
            Err(FrostError::Unsupported { .. })
        ));
        assert!(matches!(
            FrostManager::generate_shares(0, 3),
            Err(FrostError::InvalidParameters { .. })
        ));
    }

    #[test]
    fn test_frost_distribution_never_returns_xor_ciphertext() {
        let share = FrostKeyShare {
            index: 1,
            share: vec![0x11; 32],
            public_key: vec![0x02; 33],
        };
        let result = FrostManager::prepare_distribution_shares(
            &share,
            &[2, 3],
            &[(2, [0x42; 32]), (3, [0x43; 32])],
        );
        assert!(matches!(result, Err(FrostError::Unsupported { .. })));

        let malformed = FrostKeyShare {
            share: vec![0x11; 31],
            ..share
        };
        assert!(matches!(
            FrostManager::prepare_distribution_shares(&malformed, &[2], &[(2, [0x42; 32])]),
            Err(FrostError::InvalidShare { .. })
        ));
    }

    #[test]
    fn test_frost_aggregation_rejects_placeholder_inputs() {
        let shares = vec![vec![0x01; 64], vec![0x02; 64]];
        assert!(matches!(
            FrostManager::aggregate_signature(&shares, 2),
            Err(FrostError::Unsupported { .. })
        ));
        assert!(matches!(
            FrostManager::aggregate_signature(&[vec![0x01; 31]], 1),
            Err(FrostError::InvalidShare { .. })
        ));
        assert!(matches!(
            FrostManager::aggregate_signature(&[], 1),
            Err(FrostError::InsufficientShares { .. })
        ));
    }
}
