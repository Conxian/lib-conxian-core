//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP 21 (Universal Adapters) and G-43.

use crate::adapters::{TxParams, UniversalChainAdapter};
use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};

/// Represents a staking request for the Babylon protocol.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StakingIntent {
    pub intent_id: String,
    pub amount_sats: u64,
    pub locking_period_blocks: u32,
    pub status: StakingStatus,
    /// Finality provider public key.
    pub provider_pk: Option<Vec<u8>>,
}

/// Lifecycle states for a Babylon staking position.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StakingStatus {
    Proposed,
    Bonding,
    Locked,
    Slashed,
    Unbonding,
    Withdrawn,
}

/// Babylon Protocol Adapter.
pub struct BabylonAdapter;

impl BabylonAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Creates a new staking intent for institutional yield.
    pub fn create_staking_intent(&self, amount: u64, blocks: u32) -> StakingIntent {
        StakingIntent {
            intent_id: format!("babylon-intent-{}", amount),
            amount_sats: amount,
            locking_period_blocks: blocks,
            status: StakingStatus::Proposed,
            provider_pk: None,
        }
    }
}

impl Default for BabylonAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// Babylon will eventually implement UniversalChainAdapter as per CXIP-21
impl UniversalChainAdapter for BabylonAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::BitcoinUtxo
    }

    fn chain(&self) -> Chain {
        Chain::Babylon
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        // Babylon uses standard Bitcoin address formats for the vault
        if address.starts_with("bc1") || address.starts_with("1") || address.starts_with("3") {
            Ok(())
        } else {
            Err("Invalid Babylon/Bitcoin address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(1000) // Dummy fee for staking transaction
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_babylon_intent_creation() {
        let adapter = BabylonAdapter::new();
        let intent = adapter.create_staking_intent(100_000_000, 1000);
        assert_eq!(intent.amount_sats, 100_000_000);
        assert_eq!(intent.status, StakingStatus::Proposed);
    }

    #[test]
    fn test_babylon_adapter_trait() {
        let adapter = BabylonAdapter::new();
        assert_eq!(adapter.family(), ChainFamily::BitcoinUtxo);
        assert_eq!(adapter.chain(), Chain::Babylon);
        assert!(adapter.validate_address("bc1q_safe").is_ok());
    }
}
