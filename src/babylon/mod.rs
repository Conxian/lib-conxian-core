//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP-21 and G-43

use crate::adapters::{
    reject_unverified_state_proof, unavailable_state_root, StateProofError, TxParams,
    UniversalChainAdapter,
};
use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};

/// Represents a Bitcoin Staking Intent via Babylon.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StakingIntent {
    pub staker_pubkey: Vec<u8>,
    pub finality_provider_pubkey: Vec<u8>,
    pub amount_sats: u64,
    pub lock_time_blocks: u32,
}

/// Adapter for the Babylon Bitcoin Staking protocol.
pub struct BabylonAdapter;

impl UniversalChainAdapter for BabylonAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::BitcoinUtxo
    }

    fn chain(&self) -> Chain {
        Chain::Bitcoin
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
        assert_eq!(adapter.family(), ChainFamily::BitcoinUtxo);
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
}
