//! Liquid (Elements) Sidechain Adapter
//! Aligned with CXIP-21 and CON-710

use crate::adapters::{StateProofError, TxParams, UniversalChainAdapter};
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

    /// Rejects Liquid evidence until an Elements consensus/proof verifier is
    /// wired into the downstream runtime.
    ///
    /// The parser validates the envelope shape and requested-root binding only;
    /// it does not mistake those checks for confidential transaction or Merkle
    /// proof verification.
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        if state_root.trim().is_empty() {
            return Err(StateProofError::MissingStateRoot);
        }
        let mut parts = proof.split(':');
        let block_hash = parts.next().ok_or_else(|| StateProofError::InvalidProof {
            reason: "Liquid proof is missing its block hash".to_string(),
        })?;
        let proof_root = parts.next().ok_or_else(|| StateProofError::InvalidProof {
            reason: "Liquid proof is missing its Merkle root".to_string(),
        })?;
        let blinded_proof = parts.next().ok_or_else(|| StateProofError::InvalidProof {
            reason: "Liquid proof is missing its confidential proof data".to_string(),
        })?;
        if parts.next().is_some()
            || block_hash.trim().is_empty()
            || proof_root.trim().is_empty()
            || blinded_proof.trim().is_empty()
        {
            return Err(StateProofError::InvalidProof {
                reason: "Liquid proof must be exactly <block_hash>:<merkle_root>:<proof>"
                    .to_string(),
            });
        }
        if proof_root != state_root {
            return Err(StateProofError::MismatchedStateRoot {
                expected: state_root.to_string(),
                actual: proof_root.to_string(),
            });
        }

        Err(StateProofError::Unsupported {
            chain: Chain::Liquid,
            reason: "Liquid/Elements proof verification is unavailable in core".to_string(),
        })
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        Err(StateProofError::Unavailable {
            chain: Chain::Liquid,
            reason: "Liquid state roots require a verified downstream source".to_string(),
        })
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
    fn test_liquid_verify_state_proof_fails_closed() {
        let adapter = LiquidAdapter;
        let well_formed = "hash:root:blinded";
        assert!(matches!(
            adapter.verify_state_proof("root", well_formed),
            Err(StateProofError::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", ""),
            Err(StateProofError::InvalidProof { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", "hash:other:blinded"),
            Err(StateProofError::MismatchedStateRoot { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", "hash:root"),
            Err(StateProofError::InvalidProof { .. })
        ));
        assert!(matches!(
            adapter.get_state_root(),
            Err(StateProofError::Unavailable { .. })
        ));
    }
}
