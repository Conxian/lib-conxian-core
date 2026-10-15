//! FROST: Flexible Round-Optimized Schnorr Threshold Signatures
//! Institutional-grade multi-sig primitives aligned with IETF RFC 9591.

use serde::{Deserialize, Serialize};

/// Represents a secret key share in the FROST threshold scheme.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostKeyShare {
    /// Participant index (1-indexed per RFC 9591 Section 4.1).
    pub index: u32,
    /// Secret share of the group key (32 bytes scalar).
    pub share: Vec<u8>,
    /// Public key corresponding to this share (33-byte compressed or 32-byte x-only).
    pub public_key: Vec<u8>,
}

impl FrostKeyShare {
    /// Validates the structure and parameter constraints of a FROST key share per RFC 9591.
    pub fn validate(&self) -> Result<(), FrostError> {
        if self.index == 0 {
            return Err(FrostError::MalformedInput(
                "participant index must be 1-indexed (non-zero)".to_string(),
            ));
        }
        if self.share.len() != 32 || self.share.iter().all(|&b| b == 0) {
            return Err(FrostError::MalformedInput(
                "secret share scalar must be exactly 32 non-zero bytes".to_string(),
            ));
        }
        if self.public_key.len() != 33 && self.public_key.len() != 32 {
            return Err(FrostError::MalformedInput(
                "public key must be 33-byte secp256k1 compressed or 32-byte x-only".to_string(),
            ));
        }
        Ok(())
    }
}

/// A commitment to a polynomial used in VSS (Verifiable Secret Sharing).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostShareCommitment {
    /// Participant index (1-indexed).
    pub index: u32,
    /// Vector of public commitment points corresponding to polynomial coefficients.
    pub commitment_points: Vec<Vec<u8>>,
}

impl FrostShareCommitment {
    /// Validates the structure and commitment points per RFC 9591 Section 4.1.
    pub fn validate(&self) -> Result<(), FrostError> {
        if self.index == 0 {
            return Err(FrostError::MalformedInput(
                "participant index must be 1-indexed (non-zero)".to_string(),
            ));
        }
        if self.commitment_points.is_empty() {
            return Err(FrostError::MalformedInput(
                "commitment points vector must not be empty".to_string(),
            ));
        }
        for (i, point) in self.commitment_points.iter().enumerate() {
            if point.len() != 33 && point.len() != 32 {
                return Err(FrostError::MalformedInput(format!(
                    "commitment point {i} must be 33-byte compressed or 32-byte x-only"
                )));
            }
        }
        Ok(())
    }
}

/// An encrypted share for distribution during Round 2.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedFrostShare {
    pub from_index: u32,
    pub to_index: u32,
    pub encrypted_payload: Vec<u8>,
    pub mac: [u8; 32],
}

impl EncryptedFrostShare {
    /// Validates the encrypted distribution share fields per RFC 9591 Section 4.2.
    pub fn validate(&self) -> Result<(), FrostError> {
        if self.from_index == 0 || self.to_index == 0 {
            return Err(FrostError::MalformedInput(
                "from_index and to_index must be non-zero participant indices".to_string(),
            ));
        }
        if self.from_index == self.to_index {
            return Err(FrostError::MalformedInput(
                "from_index and to_index must be distinct participants".to_string(),
            ));
        }
        if self.encrypted_payload.is_empty() {
            return Err(FrostError::MalformedInput(
                "encrypted payload must not be empty".to_string(),
            ));
        }
        if self.mac.iter().all(|&b| b == 0) {
            return Err(FrostError::MalformedInput(
                "MAC tag must be a non-zero 32-byte authenticating tag".to_string(),
            ));
        }
        Ok(())
    }
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
    /// Core performs structural parameter validation on inputs prior to returning `Unsupported`.
    pub fn prepare_distribution_shares(
        from_share: &FrostKeyShare,
        target_indices: &[u32],
        shared_secrets: &[(u32, [u8; 32])],
    ) -> Result<Vec<EncryptedFrostShare>, FrostError> {
        from_share.validate()?;

        if target_indices.is_empty() {
            return Err(FrostError::MalformedInput(
                "target participant list must not be empty".to_string(),
            ));
        }
        for target in target_indices {
            if *target == 0 {
                return Err(FrostError::MalformedInput(
                    "target participant index must be non-zero".to_string(),
                ));
            }
            if *target == from_share.index {
                return Err(FrostError::MalformedInput(
                    "source participant cannot distribute shares to self".to_string(),
                ));
            }
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
    fn test_frost_key_share_validation() {
        let valid_share = FrostKeyShare {
            index: 1,
            share: vec![0x11; 32],
            public_key: vec![0x02; 33],
        };
        assert!(valid_share.validate().is_ok());

        let zero_index_share = FrostKeyShare {
            index: 0,
            share: vec![0x11; 32],
            public_key: vec![0x02; 33],
        };
        assert!(matches!(
            zero_index_share.validate(),
            Err(FrostError::MalformedInput(_))
        ));

        let invalid_share_len = FrostKeyShare {
            index: 1,
            share: vec![0x11; 31],
            public_key: vec![0x02; 33],
        };
        assert!(matches!(
            invalid_share_len.validate(),
            Err(FrostError::MalformedInput(_))
        ));

        let invalid_pubkey_len = FrostKeyShare {
            index: 1,
            share: vec![0x11; 32],
            public_key: vec![0x02; 20],
        };
        assert!(matches!(
            invalid_pubkey_len.validate(),
            Err(FrostError::MalformedInput(_))
        ));
    }

    #[test]
    fn test_frost_commitment_validation() {
        let valid_commitment = FrostShareCommitment {
            index: 1,
            commitment_points: vec![vec![0x02; 33], vec![0x03; 33]],
        };
        assert!(valid_commitment.validate().is_ok());

        let zero_index = FrostShareCommitment {
            index: 0,
            commitment_points: vec![vec![0x02; 33]],
        };
        assert!(matches!(
            zero_index.validate(),
            Err(FrostError::MalformedInput(_))
        ));

        let empty_points = FrostShareCommitment {
            index: 1,
            commitment_points: vec![],
        };
        assert!(matches!(
            empty_points.validate(),
            Err(FrostError::MalformedInput(_))
        ));
    }

    #[test]
    fn test_encrypted_frost_share_validation() {
        let valid_encrypted = EncryptedFrostShare {
            from_index: 1,
            to_index: 2,
            encrypted_payload: vec![0xaa; 48],
            mac: [0xbb; 32],
        };
        assert!(valid_encrypted.validate().is_ok());

        let same_index = EncryptedFrostShare {
            from_index: 1,
            to_index: 1,
            encrypted_payload: vec![0xaa; 48],
            mac: [0xbb; 32],
        };
        assert!(matches!(
            same_index.validate(),
            Err(FrostError::MalformedInput(_))
        ));

        let zero_mac = EncryptedFrostShare {
            from_index: 1,
            to_index: 2,
            encrypted_payload: vec![0xaa; 48],
            mac: [0x00; 32],
        };
        assert!(matches!(
            zero_mac.validate(),
            Err(FrostError::MalformedInput(_))
        ));
    }

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
