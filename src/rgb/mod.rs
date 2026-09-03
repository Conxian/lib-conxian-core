//! Client-Side Validation: RGB Protocol Integration
//! Aligned with CXIP 20 Section 6.0 and CON-1407

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Self-contained RGB contract identifier (32 bytes).
///
/// Replaces the upstream `rgb-core`/`rgb-std` `ContractId` (RGB v0.12, a
/// non-production draft line). The RGB adapter is fail-closed and only consumes
/// the contract id for membership tracking and format validation, so a local
/// 32-byte identifier is sufficient. Accepted input is a 64-character
/// hexadecimal string (matching the `validate_contract_id` hex API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContractId([u8; 32]);

impl FromStr for ContractId {
    type Err = RGBError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(RGBError::InvalidContractId);
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            let hi = hex_nibble(s.as_bytes()[i * 2]).ok_or(RGBError::InvalidContractId)?;
            let lo = hex_nibble(s.as_bytes()[i * 2 + 1]).ok_or(RGBError::InvalidContractId)?;
            *byte = (hi << 4) | lo;
        }
        Ok(ContractId(bytes))
    }
}

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

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
        assert_eq!(
            adapter.get_contract_details(""),
            Err(RGBError::InvalidContractId)
        );
        assert_eq!(
            adapter.get_contract_details("   "),
            Err(RGBError::InvalidContractId)
        );
        assert!(adapter
            .get_contract_details("rgb:2PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X-98PrBy9X")
            .is_err());
    }

    #[test]
    fn test_contract_id_hex_parsing() {
        let hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let cid = ContractId::from_str(hex).unwrap();
        assert_eq!(cid.to_string(), hex);

        assert!(ContractId::from_str("rgb:2PrBy9X-98PrBy9X").is_err());
        assert!(ContractId::from_str("short").is_err());
        assert!(ContractId::from_str(&"0".repeat(63)).is_err());
        assert!(ContractId::from_str(&"g".repeat(64)).is_err());
    }
}
