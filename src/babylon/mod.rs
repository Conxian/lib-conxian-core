//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP-21 and G-43

use crate::adapters::{TxParams, UniversalChainAdapter};
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
        Chain::Bitcoin // Babylon settles on Bitcoin L1
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.starts_with("bc1") {
            Ok(())
        } else {
            Err("Invalid Babylon/Bitcoin address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        // Babylon staking requires a standard Bitcoin tx plus a small staking metadata fee
        Ok(1500)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
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
    fn test_babylon_intent_creation() {
        let intent = StakingIntent {
            staker_pubkey: vec![0x02; 33],
            finality_provider_pubkey: vec![0x03; 33],
            amount_sats: 10_000_000,
            lock_time_blocks: 1000,
        };
        assert_eq!(intent.amount_sats, 10_000_000);
    }
}
