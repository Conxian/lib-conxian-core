//! Liquid (Elements) Sidechain Adapter
//! Aligned with CXIP-21 and CON-710

use crate::adapters::{
    reject_unverified_state_proof, unavailable_state_root, StateProofError, TxParams,
    UniversalChainAdapter,
};
use crate::control_model::{Chain, ChainFamily, TrustTier};

/// Adapter for the Liquid Network (Elements sidechain).
pub struct LiquidAdapter;

impl UniversalChainAdapter for LiquidAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::BitcoinUtxo
    }

    fn chain(&self) -> Chain {
        Chain::Liquid
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        // Liquid addresses use bech32: [prefix]1[38+ chars]
        if (address.starts_with("ex1") || address.starts_with("tlq1")) && address.len() >= 39 {
            Ok(())
        } else {
            Err("Invalid Liquid address: expected Elements bech32 format (ex1/tlq1)".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(500) // Liquid fees are typically lower than L1
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    /// No audited Elements/confidential proof verifier lives in Core.
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof("liquid", state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root("liquid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquid_adapter_trait() {
        let adapter = LiquidAdapter;
        assert_eq!(adapter.chain(), Chain::Liquid);
        assert!(adapter
            .validate_address("ex1_liquid_address_is_long_enough_39_chars")
            .is_ok());
    }

    #[test]
    fn test_liquid_verify_state_proof_hardened() {
        let adapter = LiquidAdapter;
        let valid_proof = "hash:root:blinded";
        assert!(matches!(
            adapter.verify_state_proof("root", valid_proof),
            Err(StateProofError::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("wrong-root", valid_proof),
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
