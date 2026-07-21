//! FROST: Flexible Round-Optimized Schnorr Threshold Signatures
//! Institutional-grade multi-sig primitives aligned with IETF RFC 9591.

use serde::{Deserialize, Serialize};

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

/// Manager for FROST threshold signature lifecycle.
pub struct FrostManager;

/// Typed failures for the Core FROST boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrostError {
    /// Threshold/participant parameters are invalid.
    InvalidParameters,
    /// The caller supplied a malformed share or distribution input.
    MalformedInput(String),
    /// A validly shaped request does not have enough shares.
    InsufficientShares,
    /// An audited FROST implementation is not linked into Core.
    Unsupported,
}

impl std::fmt::Display for FrostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParameters => write!(f, "invalid FROST parameters"),
            Self::MalformedInput(reason) => write!(f, "malformed FROST input: {reason}"),
            Self::InsufficientShares => write!(f, "insufficient FROST shares"),
            Self::Unsupported => write!(f, "audited FROST operations are unsupported in Core"),
        }
    }
}

impl std::error::Error for FrostError {}

impl FrostManager {
    /// Generates key shares for a given threshold (t-of-n).
    ///
    /// Core does not implement audited FROST DKG, so valid parameters return a
    /// typed unsupported error rather than fabricated shares or commitments.
    pub fn generate_shares(
        threshold: u32,
        total: u32,
    ) -> Result<(Vec<FrostKeyShare>, Vec<FrostShareCommitment>), FrostError> {
        if threshold > total || threshold == 0 {
            return Err(FrostError::InvalidParameters);
        }

        Err(FrostError::Unsupported)
    }

    /// Prepares encrypted shares for distribution (Round 2) per RFC 9591 Section 4.2.
    ///
    /// Core does not implement the audited FROST DKG/distribution protocol, so
    /// it does not emit XOR/MAC placeholders as encrypted shares.
    pub fn prepare_distribution_shares(
        from_share: &FrostKeyShare,
        target_indices: &[u32],
        shared_secrets: &[(u32, [u8; 32])],
    ) -> Result<Vec<EncryptedFrostShare>, FrostError> {
        if from_share.index == 0 || from_share.share.is_empty() {
            return Err(FrostError::MalformedInput(
                "source share must have a non-zero index and bytes".to_string(),
            ));
        }
        if target_indices.is_empty() {
            return Err(FrostError::MalformedInput(
                "target participant list must not be empty".to_string(),
            ));
        }
        for target in target_indices {
            if !shared_secrets.iter().any(|(index, _)| index == target) {
                return Err(FrostError::MalformedInput(format!(
                    "missing shared secret for participant {target}"
                )));
            }
        }

        Err(FrostError::Unsupported)
    }

    /// Aggregates partial signatures into a final Schnorr signature.
    ///
    /// A well-shaped share set still returns `Unsupported` until an audited
    /// FROST implementation verifies participant commitments, nonce binding,
    /// scalar ranges, and the final Schnorr signature.
    pub fn aggregate_signature(shares: &[Vec<u8>], threshold: u32) -> Result<Vec<u8>, FrostError> {
        if threshold == 0 {
            return Err(FrostError::InvalidParameters);
        }
        if shares.len() < threshold as usize {
            return Err(FrostError::InsufficientShares);
        }
        for (index, share) in shares.iter().enumerate() {
            if share.len() != 64 {
                return Err(FrostError::MalformedInput(format!(
                    "share {index} must contain exactly 64 bytes"
                )));
            }
        }

        Err(FrostError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frost_round_1_integrity() {
        assert!(matches!(
            FrostManager::generate_shares(2, 3),
            Err(FrostError::Unsupported)
        ));
        assert!(matches!(
            FrostManager::generate_shares(0, 3),
            Err(FrostError::InvalidParameters)
        ));
    }

    #[test]
    fn test_frost_round_2_distribution_is_not_placeholder_crypto() {
        let source = FrostKeyShare {
            index: 1,
            share: vec![0x11; 32],
            public_key: vec![0x02; 33],
        };
        let shared_secrets = vec![(2, [0x42; 32]), (3, [0x43; 32])];

        assert!(matches!(
            FrostManager::prepare_distribution_shares(&source, &[2, 3], &shared_secrets),
            Err(FrostError::Unsupported)
        ));
    }

    #[test]
    fn test_frost_signature_aggregation_hardened() {
        let share1 = vec![0x01; 64];
        let share2 = vec![0x02; 64];

        assert_eq!(
            FrostManager::aggregate_signature(&[share1, share2], 2),
            Err(FrostError::Unsupported)
        );
        assert!(matches!(
            FrostManager::aggregate_signature(&[vec![0x01; 31], vec![0x02; 64]], 2),
            Err(FrostError::MalformedInput(_))
        ));
        assert!(matches!(
            FrostManager::aggregate_signature(&[vec![0x01; 64], vec![0x02; 63]], 2),
            Err(FrostError::MalformedInput(_))
        ));
        assert_eq!(
            FrostManager::aggregate_signature(&[], 0),
            Err(FrostError::InvalidParameters)
        );
        assert_eq!(
            FrostManager::aggregate_signature(&[], 1),
            Err(FrostError::InsufficientShares)
        );
    }
}
