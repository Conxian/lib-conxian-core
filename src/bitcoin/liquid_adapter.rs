//! Liquid (Elements) Sidechain Adapter
//! Aligned with CXIP-21, CON-710, and CON-712

use crate::adapters::{
    reject_unverified_state_proof, unavailable_state_root, StateProofError, TxParams,
    UniversalChainAdapter,
};
use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Typed failure returned when Liquid sidechain or peg operations fail.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum LiquidError {
    InvalidAddress,
    InvalidAmount,
    InvalidAssetId,
    InvalidTxid,
    InvalidProof,
    PegInFailed(String),
    PegOutFailed(String),
    StatusUnavailable,
    UnknownIntent,
}

impl fmt::Display for LiquidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress => write!(
                f,
                "Invalid Liquid address: expected Elements bech32 format (ex1/tlq1)"
            ),
            Self::InvalidAmount => write!(
                f,
                "Invalid transaction or peg amount: must be non-zero satoshis"
            ),
            Self::InvalidAssetId => write!(
                f,
                "Invalid Elements asset ID: expected 32-byte hex string or asset tag"
            ),
            Self::InvalidTxid => write!(f, "Invalid Bitcoin or Elements transaction ID"),
            Self::InvalidProof => write!(f, "Invalid state or confidential transaction proof"),
            Self::PegInFailed(msg) => write!(f, "Liquid peg-in failed: {msg}"),
            Self::PegOutFailed(msg) => write!(f, "Liquid peg-out failed: {msg}"),
            Self::StatusUnavailable => write!(f, "Liquid peg status evidence is unavailable"),
            Self::UnknownIntent => write!(f, "Unknown or empty Liquid intent ID"),
        }
    }
}

impl std::error::Error for LiquidError {}

/// Lifecycle states for Liquid sidechain peg operations.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiquidPegState {
    Pending,
    BitcoinConfirmed,
    ElementsPegInMined,
    ElementsPegOutMined,
    Finalized,
    Failed,
}

/// Intent data structure for Liquid peg-in and peg-out operations.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LiquidPegIntent {
    pub intent_id: String,
    pub amount_sats: u64,
    pub asset_id: String,
    pub liquid_address: String,
    pub bitcoin_txid: Option<String>,
    pub state: LiquidPegState,
    pub created_at_epoch: u64,
}

impl LiquidPegIntent {
    /// Enforces fail-closed parameter validation for Liquid peg intents.
    pub fn validate(&self) -> Result<(), LiquidError> {
        if self.intent_id.trim().is_empty() {
            return Err(LiquidError::UnknownIntent);
        }
        if self.amount_sats == 0 {
            return Err(LiquidError::InvalidAmount);
        }
        if self.asset_id.trim().is_empty() {
            return Err(LiquidError::InvalidAssetId);
        }
        if self.liquid_address.trim().is_empty()
            || (!self.liquid_address.starts_with("ex1") && !self.liquid_address.starts_with("tlq1"))
            || self.liquid_address.len() < 39
        {
            return Err(LiquidError::InvalidAddress);
        }
        if let Some(ref txid) = self.bitcoin_txid {
            if txid.trim().is_empty() {
                return Err(LiquidError::InvalidTxid);
            }
        }
        Ok(())
    }
}

/// Core interface for Liquid peg-in and peg-out operations.
pub trait LiquidPegAdapter {
    /// Initiates a peg-in (BTC -> L-BTC / Liquid Asset)
    fn initiate_peg_in(
        &self,
        amount_sats: u64,
        asset_id: &str,
        btc_txid: &str,
    ) -> Result<LiquidPegIntent, LiquidError>;

    /// Initiates a peg-out (Liquid Asset -> BTC)
    fn initiate_peg_out(
        &self,
        amount_sats: u64,
        asset_id: &str,
        liquid_address: &str,
    ) -> Result<LiquidPegIntent, LiquidError>;

    /// Verifies the status of an ongoing peg intent
    fn get_peg_status(&self, intent_id: &str) -> Result<LiquidPegState, LiquidError>;
}

/// Bridge provider implementation for Liquid sidechain operations.
pub struct LiquidBridge;

impl Default for LiquidBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidBridge {
    pub fn new() -> Self {
        Self
    }
}

impl LiquidPegAdapter for LiquidBridge {
    fn initiate_peg_in(
        &self,
        amount_sats: u64,
        asset_id: &str,
        btc_txid: &str,
    ) -> Result<LiquidPegIntent, LiquidError> {
        if amount_sats == 0 {
            return Err(LiquidError::InvalidAmount);
        }
        if asset_id.trim().is_empty() {
            return Err(LiquidError::InvalidAssetId);
        }
        if btc_txid.trim().is_empty() {
            return Err(LiquidError::InvalidTxid);
        }

        let intent = LiquidPegIntent {
            intent_id: format!("liquid-pegin-{}", btc_txid.trim()),
            amount_sats,
            asset_id: asset_id.trim().to_string(),
            liquid_address: "ex1q_placeholder_liquid_address_39_chars_min".to_string(),
            bitcoin_txid: Some(btc_txid.trim().to_string()),
            state: LiquidPegState::BitcoinConfirmed,
            created_at_epoch: 1718363200,
        };

        intent.validate()?;
        Ok(intent)
    }

    fn initiate_peg_out(
        &self,
        amount_sats: u64,
        asset_id: &str,
        liquid_address: &str,
    ) -> Result<LiquidPegIntent, LiquidError> {
        if amount_sats == 0 {
            return Err(LiquidError::InvalidAmount);
        }
        if asset_id.trim().is_empty() {
            return Err(LiquidError::InvalidAssetId);
        }
        if liquid_address.trim().is_empty()
            || (!liquid_address.starts_with("ex1") && !liquid_address.starts_with("tlq1"))
            || liquid_address.len() < 39
        {
            return Err(LiquidError::InvalidAddress);
        }

        let intent = LiquidPegIntent {
            intent_id: format!("liquid-pegout-{}", liquid_address.trim()),
            amount_sats,
            asset_id: asset_id.trim().to_string(),
            liquid_address: liquid_address.trim().to_string(),
            bitcoin_txid: None,
            state: LiquidPegState::Pending,
            created_at_epoch: 1718363200,
        };

        intent.validate()?;
        Ok(intent)
    }

    fn get_peg_status(&self, intent_id: &str) -> Result<LiquidPegState, LiquidError> {
        if intent_id.trim().is_empty() {
            return Err(LiquidError::UnknownIntent);
        }
        Err(LiquidError::StatusUnavailable)
    }
}

/// Adapter for the Liquid Network (Elements sidechain).
pub struct LiquidAdapter;

impl UniversalChainAdapter for LiquidAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::Federation
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

    #[test]
    fn test_liquid_peg_in_interface() {
        let bridge = LiquidBridge::new();
        let intent = bridge
            .initiate_peg_in(1_000_000, "6f02...lbtc", "btc_txid_123")
            .unwrap();
        assert_eq!(intent.amount_sats, 1_000_000);
        assert_eq!(intent.asset_id, "6f02...lbtc");
        assert_eq!(intent.state, LiquidPegState::BitcoinConfirmed);
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_liquid_peg_out_interface() {
        let bridge = LiquidBridge::new();
        let addr = "ex1q_valid_liquid_address_for_peg_out_39_chars";
        let intent = bridge
            .initiate_peg_out(500_000, "6f02...lbtc", addr)
            .unwrap();
        assert_eq!(intent.liquid_address, addr);
        assert_eq!(intent.state, LiquidPegState::Pending);
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_liquid_invalid_parameters_and_zero_amount() {
        let bridge = LiquidBridge::new();
        assert_eq!(
            bridge.initiate_peg_in(0, "asset_id", "txid"),
            Err(LiquidError::InvalidAmount)
        );
        assert_eq!(
            bridge.initiate_peg_in(100, "", "txid"),
            Err(LiquidError::InvalidAssetId)
        );
        assert_eq!(
            bridge.initiate_peg_in(100, "asset_id", ""),
            Err(LiquidError::InvalidTxid)
        );
        assert_eq!(
            bridge.initiate_peg_out(100, "asset_id", "invalid_address"),
            Err(LiquidError::InvalidAddress)
        );
    }

    #[test]
    fn test_liquid_intent_validation() {
        let valid = LiquidPegIntent {
            intent_id: "liquid-intent-1".to_string(),
            amount_sats: 100_000,
            asset_id: "lbtc-asset-tag".to_string(),
            liquid_address: "ex1q_valid_liquid_address_for_peg_out_39_chars".to_string(),
            bitcoin_txid: Some("btc_txid_456".to_string()),
            state: LiquidPegState::Pending,
            created_at_epoch: 1718363200,
        };
        assert!(valid.validate().is_ok());

        let mut zero_amount = valid.clone();
        zero_amount.amount_sats = 0;
        assert_eq!(zero_amount.validate(), Err(LiquidError::InvalidAmount));

        let mut empty_id = valid.clone();
        empty_id.intent_id = "".to_string();
        assert_eq!(empty_id.validate(), Err(LiquidError::UnknownIntent));

        let mut empty_asset = valid.clone();
        empty_asset.asset_id = "".to_string();
        assert_eq!(empty_asset.validate(), Err(LiquidError::InvalidAssetId));

        let mut invalid_addr = valid.clone();
        invalid_addr.liquid_address = "short".to_string();
        assert_eq!(invalid_addr.validate(), Err(LiquidError::InvalidAddress));

        let mut empty_txid = valid.clone();
        empty_txid.bitcoin_txid = Some("".to_string());
        assert_eq!(empty_txid.validate(), Err(LiquidError::InvalidTxid));
    }

    #[test]
    fn test_liquid_status_requires_authoritative_evidence() {
        let bridge = LiquidBridge::new();
        assert_eq!(
            bridge.get_peg_status("liquid-intent-1"),
            Err(LiquidError::StatusUnavailable)
        );
        assert_eq!(bridge.get_peg_status(""), Err(LiquidError::UnknownIntent));
    }
}
