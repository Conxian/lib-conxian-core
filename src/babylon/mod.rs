//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP-21 and G-43

use crate::adapters::{StateProofError, TxParams, UniversalChainAdapter};
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

    /// Rejects Babylon evidence until a real EOTS/finality verifier is wired.
    ///
    /// The parser only establishes that the input is well-formed enough to
    /// classify. It never treats a height/signature-shaped string as verified.
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        if state_root.trim().is_empty() {
            return Err(StateProofError::MissingStateRoot);
        }
        let mut parts = proof.split(':');
        let height = parts.next().ok_or_else(|| StateProofError::InvalidProof {
            reason: "Babylon proof is missing its height".to_string(),
        })?;
        let signature = parts.next().ok_or_else(|| StateProofError::InvalidProof {
            reason: "Babylon proof is missing its EOTS signature".to_string(),
        })?;
        if parts.next().is_some() || height.trim().is_empty() || signature.trim().is_empty() {
            return Err(StateProofError::InvalidProof {
                reason: "Babylon proof must be exactly <height>:<signature_hex>".to_string(),
            });
        }
        height
            .parse::<u64>()
            .map_err(|_| StateProofError::InvalidProof {
                reason: "Babylon proof height must be an unsigned integer".to_string(),
            })?;
        let signature_bytes =
            hex::decode(signature).map_err(|_| StateProofError::InvalidProof {
                reason: "Babylon EOTS signature must be hexadecimal".to_string(),
            })?;
        if signature_bytes.len() != 64 {
            return Err(StateProofError::InvalidProof {
                reason: "Babylon EOTS signature must contain 64 bytes".to_string(),
            });
        }

        Err(StateProofError::Unsupported {
            chain: Chain::Bitcoin,
            reason: "Babylon EOTS/finality verification is unavailable in core".to_string(),
        })
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        Err(StateProofError::Unavailable {
            chain: Chain::Bitcoin,
            reason: "Babylon state roots require a verified downstream source".to_string(),
        })
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
    fn test_babylon_verify_state_proof_fails_closed() {
        let adapter = BabylonAdapter;
        let well_formed = format!("840000:{}", "ab".repeat(64));
        assert!(matches!(
            adapter.verify_state_proof("root", &well_formed),
            Err(StateProofError::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", ""),
            Err(StateProofError::InvalidProof { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", "invalid_format"),
            Err(StateProofError::InvalidProof { .. })
        ));
        assert!(matches!(
            adapter.verify_state_proof("root", "840000:not-hex"),
            Err(StateProofError::InvalidProof { .. })
        ));
        assert!(matches!(
            adapter.get_state_root(),
            Err(StateProofError::Unavailable { .. })
        ));
    }
}
