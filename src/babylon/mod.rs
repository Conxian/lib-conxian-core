//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP-21 and G-43

use crate::adapters::{
    reject_unverified_state_proof, unavailable_state_root, StateProofError, TxParams,
    UniversalChainAdapter,
};
use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Specific error variants for Babylon Bitcoin Staking operations.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum BabylonError {
    InvalidStakerPubkey,
    InvalidFinalityProviderPubkey,
    InvalidAmount,
    InvalidLockTime,
}

impl fmt::Display for BabylonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStakerPubkey => write!(
                f,
                "Invalid staker public key dimension (must be 32 or 33 bytes)"
            ),
            Self::InvalidFinalityProviderPubkey => write!(
                f,
                "Invalid finality provider public key dimension (must be 32 or 33 bytes)"
            ),
            Self::InvalidAmount => write!(f, "Staking satoshis amount must be greater than zero"),
            Self::InvalidLockTime => write!(f, "Lock time must be at least 10 block confirmations"),
        }
    }
}

impl std::error::Error for BabylonError {}

/// Represents a Bitcoin Staking Intent via Babylon with fail-closed validation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StakingIntent {
    pub staker_pubkey: Vec<u8>,
    pub finality_provider_pubkey: Vec<u8>,
    pub amount_sats: u64,
    pub lock_time_blocks: u32,
}

impl StakingIntent {
    /// Minimum block lock time for Babylon Bitcoin staking.
    pub const MIN_LOCK_TIME_BLOCKS: u32 = 10;

    /// Validates the staking intent parameters fail-closed.
    pub fn validate(&self) -> Result<(), BabylonError> {
        if self.staker_pubkey.len() != 32 && self.staker_pubkey.len() != 33 {
            return Err(BabylonError::InvalidStakerPubkey);
        }
        if self.finality_provider_pubkey.len() != 32 && self.finality_provider_pubkey.len() != 33 {
            return Err(BabylonError::InvalidFinalityProviderPubkey);
        }
        if self.amount_sats == 0 {
            return Err(BabylonError::InvalidAmount);
        }
        if self.lock_time_blocks < Self::MIN_LOCK_TIME_BLOCKS {
            return Err(BabylonError::InvalidLockTime);
        }
        Ok(())
    }
}

/// Adapter for the Babylon Bitcoin Staking protocol.
pub struct BabylonAdapter;

impl UniversalChainAdapter for BabylonAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::BPoS
    }

    fn chain(&self) -> Chain {
        Chain::Babylon
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.starts_with("bc1") {
            Ok(())
        } else {
            Err("Invalid Babylon/Bitcoin address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        // Babylon-specific staking fee estimation
        let base_fee = 1500u64;
        let staking_data_overhead = 100u64;
        Ok(base_fee + staking_data_overhead)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    /// No audited Babylon EOTS/header verifier lives in Core.
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof("babylon", state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root("babylon")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_babylon_adapter_trait() {
        let adapter = BabylonAdapter;
        assert_eq!(adapter.family(), ChainFamily::BPoS);
        assert!(adapter.validate_address("bc1q_babylon").is_ok());
    }

    #[test]
    fn test_babylon_verify_state_proof_hardened() {
        let adapter = BabylonAdapter;
        assert!(matches!(
            adapter.verify_state_proof("root", "840000:abc123def"),
            Err(StateProofError::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("wrong-root", "840000:abc123def"),
            Err(StateProofError::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", ""),
            Err(StateProofError::MalformedInput(_))
        ));
        assert!(matches!(
            adapter.get_state_root(),
            Err(StateProofError::Unavailable { .. })
        ));
    }

    #[test]
    fn test_staking_intent_validation() {
        let valid_intent = StakingIntent {
            staker_pubkey: vec![0u8; 32],
            finality_provider_pubkey: vec![1u8; 33],
            amount_sats: 100_000,
            lock_time_blocks: 144,
        };
        assert!(valid_intent.validate().is_ok());

        let invalid_staker = StakingIntent {
            staker_pubkey: vec![0u8; 16],
            ..valid_intent.clone()
        };
        assert_eq!(
            invalid_staker.validate(),
            Err(BabylonError::InvalidStakerPubkey)
        );

        let invalid_finality = StakingIntent {
            finality_provider_pubkey: vec![1u8; 64],
            ..valid_intent.clone()
        };
        assert_eq!(
            invalid_finality.validate(),
            Err(BabylonError::InvalidFinalityProviderPubkey)
        );

        let invalid_amount = StakingIntent {
            amount_sats: 0,
            ..valid_intent.clone()
        };
        assert_eq!(invalid_amount.validate(), Err(BabylonError::InvalidAmount));

        let invalid_lock = StakingIntent {
            lock_time_blocks: 5,
            ..valid_intent.clone()
        };
        assert_eq!(invalid_lock.validate(), Err(BabylonError::InvalidLockTime));
    }
}
