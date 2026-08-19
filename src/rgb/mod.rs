//! Client-Side Validation: RGB Protocol Integration
//! Aligned with CXIP 20 Section 6.0 and CON-1407

use rgb::ContractId;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Failure taxonomy for RGB operations.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RGBError {
    /// Invalid contract identifier.
    InvalidContractId,
    /// Schema validation failed.
    SchemaMismatch,
    /// Transition validation failed via AluVM.
    TransitionValidationFailed(String),
    /// Single-use seal verification failed.
    SealVerificationFailed,
    /// Contract lookup failed (node-backed).
    ContractNotFound(String),
    /// Operation gated by current rollout mode.
    GatedByRolloutMode,
    /// No audited RGB transition/seal verifier is available in Core.
    VerificationUnavailable,
    /// Shadow observations are explicitly non-authoritative.
    NonAuthoritativeShadow,
    /// Persistence layer error.
    PersistenceError(String),
}

impl std::fmt::Display for RGBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidContractId => write!(f, "Invalid RGB contract ID"),
            Self::SchemaMismatch => write!(f, "RGB schema mismatch"),
            Self::TransitionValidationFailed(msg) => {
                write!(f, "RGB transition validation failed: {msg}")
            }
            Self::SealVerificationFailed => write!(f, "RGB seal verification failed"),
            Self::ContractNotFound(id) => write!(f, "RGB contract not found: {id}"),
            Self::GatedByRolloutMode => write!(f, "RGB operation gated by rollout mode"),
            Self::VerificationUnavailable => {
                write!(f, "RGB verification is unavailable in Core")
            }
            Self::NonAuthoritativeShadow => {
                write!(f, "RGB shadow observation is non-authoritative")
            }
            Self::PersistenceError(msg) => write!(f, "RGB persistence error: {msg}"),
        }
    }
}

impl std::error::Error for RGBError {}

/// Rollout modes for RGB lane execution (CON-767).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RGBExecutionMode {
    /// Adapter is inactive; all calls return errors.
    Disabled,
    /// Adapter observations are collected but are never authoritative.
    /// This mode cannot authorize a production flow.
    Shadow,
    /// Adapter is fully active and enforced by a real downstream verifier.
    Active,
}

/// Core interface for the RGB Protocol Adapter (CON-767).
/// This trait defines the expected behavior for any node-backed RGB integration.
pub trait RGBAdapter {
    /// Validates a state transition against the contract schema.
    fn validate_transition(&self, transition_hex: &str) -> Result<bool, RGBError>;

    /// Verifies a single-use seal anchored to a Bitcoin UTXO.
    fn verify_seal(&self, utxo_txid: &str, seal_commitment: &str) -> Result<bool, RGBError>;

    /// Performs a node-backed lookup for a contract by ID.
    fn get_contract_details(&self, contract_id: &str) -> Result<String, RGBError>;
}

/// In-memory RGB adapter with stock-style contract membership (CON-1407).
pub struct RGBStockAdapter {
    pub contract_ids: Vec<ContractId>,
}

impl RGBStockAdapter {
    pub fn new() -> Self {
        Self {
            contract_ids: Vec::new(),
        }
    }
}

impl Default for RGBStockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RGBAdapter for RGBStockAdapter {
    fn validate_transition(&self, transition_hex: &str) -> Result<bool, RGBError> {
        if transition_hex.is_empty() {
            return Err(RGBError::TransitionValidationFailed(
                "Empty transition".to_string(),
            ));
        }
        Err(RGBError::VerificationUnavailable)
    }

    fn verify_seal(&self, utxo_txid: &str, seal_commitment: &str) -> Result<bool, RGBError> {
        if utxo_txid.is_empty() || seal_commitment.is_empty() {
            return Err(RGBError::SealVerificationFailed);
        }
        Err(RGBError::VerificationUnavailable)
    }

    fn get_contract_details(&self, contract_id: &str) -> Result<String, RGBError> {
        if contract_id.trim().is_empty() {
            return Err(RGBError::InvalidContractId);
        }
        let cid = ContractId::from_str(contract_id).map_err(|_| RGBError::InvalidContractId)?;
        if self.contract_ids.contains(&cid) {
            Ok(format!("Contract details for {}", contract_id))
        } else {
            Err(RGBError::ContractNotFound(contract_id.to_string()))
        }
    }
}

/// A skeleton implementation of RGBAdapter for PoC/Research purposes (CON-768).
pub struct RGBSkeletonAdapter;

impl RGBAdapter for RGBSkeletonAdapter {
    fn validate_transition(&self, transition_hex: &str) -> Result<bool, RGBError> {
        if transition_hex.is_empty() {
            return Err(RGBError::TransitionValidationFailed(
                "Empty transition".to_string(),
            ));
        }
        Err(RGBError::VerificationUnavailable)
    }

    fn verify_seal(&self, utxo_txid: &str, seal_commitment: &str) -> Result<bool, RGBError> {
        if utxo_txid.is_empty() || seal_commitment.is_empty() {
            return Err(RGBError::SealVerificationFailed);
        }
        Err(RGBError::VerificationUnavailable)
    }

    fn get_contract_details(&self, contract_id: &str) -> Result<String, RGBError> {
        if contract_id.trim().is_empty() {
            return Err(RGBError::InvalidContractId);
        }
        Err(RGBError::VerificationUnavailable)
    }
}

/// Runtime coordinator for RGB operations.
pub struct RGBRuntime<A: RGBAdapter> {
    pub mode: RGBExecutionMode,
    pub adapter: A,
}

impl<A: RGBAdapter> RGBRuntime<A> {
    pub fn new(mode: RGBExecutionMode, adapter: A) -> Self {
        Self { mode, adapter }
    }

    /// Validates a transition, respecting the execution mode (CON-767/CON-768).
    pub fn validate_transition(&self, transition_hex: &str) -> Result<bool, RGBError> {
        match self.mode {
            RGBExecutionMode::Disabled => Err(RGBError::GatedByRolloutMode),
            RGBExecutionMode::Shadow => match self.adapter.validate_transition(transition_hex) {
                Err(error @ RGBError::TransitionValidationFailed(_)) => Err(error),
                Err(RGBError::VerificationUnavailable) | Ok(_) => {
                    Err(RGBError::NonAuthoritativeShadow)
                }
                Err(error) => Err(error),
            },
            RGBExecutionMode::Active => self.adapter.validate_transition(transition_hex),
        }
    }

    /// Verifies a seal, respecting the execution mode.
    pub fn verify_seal(&self, utxo_txid: &str, seal_commitment: &str) -> Result<bool, RGBError> {
        match self.mode {
            RGBExecutionMode::Disabled => Err(RGBError::GatedByRolloutMode),
            RGBExecutionMode::Shadow => {
                match self.adapter.verify_seal(utxo_txid, seal_commitment) {
                    Err(error @ RGBError::SealVerificationFailed) => Err(error),
                    Err(RGBError::VerificationUnavailable) | Ok(_) => {
                        Err(RGBError::NonAuthoritativeShadow)
                    }
                    Err(error) => Err(error),
                }
            }
            RGBExecutionMode::Active => self.adapter.verify_seal(utxo_txid, seal_commitment),
        }
    }

    /// Performs a node-backed lookup for a contract, respecting the execution mode.
    pub fn get_contract_details(&self, contract_id: &str) -> Result<String, RGBError> {
        match self.mode {
            RGBExecutionMode::Disabled => Err(RGBError::GatedByRolloutMode),
            _ => self.adapter.get_contract_details(contract_id),
        }
    }

    /// Validates an RGB Contract ID.
    pub fn validate_contract_id(&self, id_hex: &str) -> Result<ContractId, RGBError> {
        ContractId::from_str(id_hex).map_err(|_| RGBError::InvalidContractId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_execution_modes() {
        let disabled = RGBRuntime::new(RGBExecutionMode::Disabled, RGBSkeletonAdapter);
        let shadow = RGBRuntime::new(RGBExecutionMode::Shadow, RGBSkeletonAdapter);
        let active = RGBRuntime::new(RGBExecutionMode::Active, RGBSkeletonAdapter);

        assert_eq!(
            disabled.validate_transition("abc"),
            Err(RGBError::GatedByRolloutMode)
        );

        assert_eq!(
            shadow.validate_transition("abc"),
            Err(RGBError::NonAuthoritativeShadow)
        );

        assert_eq!(
            active.validate_transition("abc"),
            Err(RGBError::VerificationUnavailable)
        );
        assert_eq!(
            active.validate_transition(""),
            Err(RGBError::TransitionValidationFailed(
                "Empty transition".to_string()
            ))
        );
        assert_eq!(
            shadow.verify_seal("utxo", "commitment"),
            Err(RGBError::NonAuthoritativeShadow)
        );
    }

    #[test]
    fn test_rgb_stock_adapter_persistence() {
        let adapter = RGBStockAdapter::new();
        assert_eq!(adapter.get_contract_details(""), Err(RGBError::InvalidContractId));
        assert_eq!(adapter.get_contract_details("   "), Err(RGBError::InvalidContractId));
        assert!(adapter
            .get_contract_details("rgb:2PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X")
            .is_err());
    }
}
