use serde::{Deserialize, Serialize};

use super::bip110::Bip110Limits;

/// BIP-110 Compliance Constants
///
/// BIP-110 (Reduced Data Temporary Softfork) limits data embedding in Bitcoin transactions:
/// - Max 256-byte pushdata per element
/// - 83-byte OP_RETURN output
/// - 34-byte ScriptPubKey for standard P2PKH/P2WPKH
///
/// See [docs/BIP110_ALIGNMENT.md](https://github.com/Conxian/lib-conxian-core/blob/main/docs/BIP110_ALIGNMENT.md)
pub mod bip110 {
    /// Maximum size of a single pushdata element in bytes
    pub const MAX_PUSHDATA_BYTES: usize = 256;

    /// Maximum OP_RETURN output size in bytes (standard policy under BIP-110)
    pub const MAX_OP_RETURN_BYTES: usize = 83;

    /// Maximum ScriptPubKey size for standard addresses (P2PKH/P2WPKH) in bytes
    pub const MAX_SCRIPT_PUBKEY_BYTES: usize = 34;

    /// Maximum witness element size in bytes
    pub const MAX_WITNESS_ELEMENT_BYTES: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Bitcoin,
    Stacks,
    Liquid,
    Lightning,
    Babylon,
    Bob,
    Mezo,
    Citrea,
    Botanix,
    Ethereum,
    Base,
    Arbitrum,
    Optimism,
    Polygon,
    CosmosHub,
    Osmosis,
    Celestia,
    Solana,
    Eclipse,
    Aptos,
    Sui,
    Polkadot,
    Kusama,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    BitcoinUtxo,
    Evm,
    CosmosIbc,
    SolanaSvm,
    Move,
    Substrate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Strict,
    Managed,
    Expedient,
    ObserverOnly,
}

impl TrustTier {
    pub fn is_production_allowed(&self) -> bool {
        matches!(
            self,
            TrustTier::Strict | TrustTier::Managed | TrustTier::Expedient
        )
    }

    pub fn requires_light_client(&self) -> bool {
        matches!(self, TrustTier::Strict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationClass {
    LightClient,
    ExternalQuorum,
    AppDefinedMultiVerifier,
    SharedPos,
    NativeObservation,
    ZkVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalityClass {
    Economic,
    Probabilistic,
    Deterministic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSystem {
    Ibc,
    WormholeNtt,
    Hyperlane,
    LayerZeroV2,
    Axelar,
    ChainlinkCcip,
    NearChainSignatures,
    CircleCctp,
    NexusZkVM,
    Bitvm2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEnvelope {
    pub system: BridgeSystem,
    pub system_version: String,
    pub trust_tier: TrustTier,
    pub verification_class: VerificationClass,
    pub source_chain_id: String,
    pub destination_chain_id: String,
    pub finality_class: FinalityClass,
    pub min_confirmations: u32,
    pub observed_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub proof_ref: String,
    pub evidence_hash: String,
    pub evidence_uri: Option<String>,
    pub verifier_set_ref: String,
    pub security_params: serde_json::Value,
    pub verification_status: VerificationStatus,
    pub verification_reason: Option<String>,
}

pub fn validate_trust_tier_policy(
    tier: TrustTier,
    verification: VerificationClass,
) -> Result<(), String> {
    if !tier.is_production_allowed() {
        return Err(format!(
            "Trust tier {:?} is not allowed in production",
            tier
        ));
    }

    if tier.requires_light_client() && verification != VerificationClass::LightClient {
        return Err(format!(
            "Strict trust tier requires light_client verification, but found {:?}",
            verification
        ));
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RiskAssessment {
    pub overall_level: String,
    pub da_score: u32,
    pub settlement_score: u32,
    pub bridge_score: u32,
    pub exit_mechanism_score: u32,
    pub operators_score: u32,
    pub decentralization_score: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailTrustAssumptions {
    pub security_anchor: String,
    pub operator_dependency: String,
    pub liveness_assumption: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailFinalitySemantics {
    pub confirmation_model: String,
    pub settlement_layer: String,
    pub typical_finality_window: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailCustodyModel {
    pub asset_control_model: String,
    pub signer_architecture: String,
    pub redemption_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailComplianceConstraints {
    pub baseline_controls: Vec<String>,
    pub jurisdictional_scope: String,
    pub monitoring_requirements: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailOperationalCapabilities {
    pub supported_flows: Vec<String>,
    pub integration_modes: Vec<String>,
    pub resilience_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailMetadata {
    pub rail_family: ChainFamily,
    pub trust_assumptions: RailTrustAssumptions,
    pub finality_semantics: RailFinalitySemantics,
    pub custody_model: RailCustodyModel,
    pub compliance_constraints: RailComplianceConstraints,
    pub operational_capabilities: RailOperationalCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionLifecycleStatus {
    PendingAttestation,
    Active,
    Suspended,
    Revoked,
    Expired,
}

/// BIP-110 compliance validation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110ValidationResult {
    pub is_compliant: bool,
    pub violations: Vec<Bip110Violation>,
}

impl Bip110ValidationResult {
    pub fn compliant() -> Self {
        Self {
            is_compliant: true,
            violations: Vec::new(),
        }
    }

    pub fn non_compliant(violations: Vec<Bip110Violation>) -> Self {
        Self {
            is_compliant: false,
            violations,
        }
    }
}

/// Specific BIP-110 violation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Bip110Violation {
    PushdataExceedsLimit { size: usize, max: usize },
    OpReturnExceedsLimit { size: usize, max: usize },
    ScriptPubKeyExceedsLimit { size: usize, max: usize },
    WitnessElementExceedsLimit { size: usize, max: usize },
}

impl std::fmt::Display for Bip110Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PushdataExceedsLimit { size, max } => {
                write!(
                    f,
                    "Pushdata size {} exceeds BIP-110 limit of {} bytes",
                    size, max
                )
            }
            Self::OpReturnExceedsLimit { size, max } => {
                write!(
                    f,
                    "OP_RETURN size {} exceeds BIP-110 limit of {} bytes",
                    size, max
                )
            }
            Self::ScriptPubKeyExceedsLimit { size, max } => {
                write!(
                    f,
                    "ScriptPubKey size {} exceeds BIP-110 limit of {} bytes",
                    size, max
                )
            }
            Self::WitnessElementExceedsLimit { size, max } => {
                write!(
                    f,
                    "Witness element size {} exceeds BIP-110 limit of {} bytes",
                    size, max
                )
            }
        }
    }
}

impl std::error::Error for Bip110Violation {}

/// BIP-110 Compliance struct for validating Bitcoin transaction data sizes.
///
/// BIP-110 (Reduced Data Temporary Softfork) enforces strict limits on data embedding:
/// - Maximum 256-byte pushdata per element
/// - Maximum 83-byte OP_RETURN output
/// - Maximum 34-byte ScriptPubKey for standard addresses
///
/// This struct provides validation helpers aligned with the `TrustTier::Strict` (T1)
/// trust tier for Bitcoin bridges, ensuring monetary use cases are prioritized
/// over data storage.
///
/// # Example
///
/// ```rust
/// use lib_conxian_core::control_model::{Bip110Compliance, TrustTier};
///
/// let compliance = Bip110Compliance::new();
/// let result = compliance.validate_pushdata(100);
/// assert!(result.is_compliant);
///
/// let large_data = vec![0u8; 300];
/// let result = compliance.validate_pushdata(large_data.len());
/// assert!(!result.is_compliant);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Bip110Compliance {
    enabled: bool,
    limits: Bip110Limits,
}

impl Bip110Compliance {
    pub fn new() -> Self {
        Self {
            enabled: true,
            limits: Bip110Limits::canonical(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            limits: Bip110Limits::canonical(),
        }
    }

    /// Creates an enabled validator with an explicit set of size limits.
    pub fn with_limits(limits: Bip110Limits) -> Self {
        Self {
            enabled: true,
            limits,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the size limits used by this validator.
    pub fn limits(&self) -> &Bip110Limits {
        &self.limits
    }

    /// Validates pushdata size against BIP-110 limit (256 bytes)
    pub fn validate_pushdata(&self, size: usize) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        if size > self.limits.max_pushdata_bytes {
            Bip110ValidationResult::non_compliant(vec![Bip110Violation::PushdataExceedsLimit {
                size,
                max: self.limits.max_pushdata_bytes,
            }])
        } else {
            Bip110ValidationResult::compliant()
        }
    }

    /// Validates OP_RETURN output size against BIP-110 limit (83 bytes)
    pub fn validate_op_return(&self, size: usize) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        if size > self.limits.max_op_return_bytes {
            Bip110ValidationResult::non_compliant(vec![Bip110Violation::OpReturnExceedsLimit {
                size,
                max: self.limits.max_op_return_bytes,
            }])
        } else {
            Bip110ValidationResult::compliant()
        }
    }

    /// Validates ScriptPubKey size against BIP-110 limit (34 bytes)
    pub fn validate_script_pubkey(&self, size: usize) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        if size > self.limits.max_script_pubkey_bytes {
            Bip110ValidationResult::non_compliant(vec![Bip110Violation::ScriptPubKeyExceedsLimit {
                size,
                max: self.limits.max_script_pubkey_bytes,
            }])
        } else {
            Bip110ValidationResult::compliant()
        }
    }

    /// Validates witness element size against BIP-110 limit (256 bytes)
    pub fn validate_witness_element(&self, size: usize) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        if size > self.limits.max_witness_element_bytes {
            Bip110ValidationResult::non_compliant(vec![
                Bip110Violation::WitnessElementExceedsLimit {
                    size,
                    max: self.limits.max_witness_element_bytes,
                },
            ])
        } else {
            Bip110ValidationResult::compliant()
        }
    }

    /// Validates a complete transaction against all BIP-110 limits
    pub fn validate_transaction(
        &self,
        pushdatas: &[usize],
        op_return_size: Option<usize>,
        script_pubkey_size: usize,
        witness_elements: &[usize],
    ) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        let mut violations = Vec::new();

        for &size in pushdatas.iter() {
            if size > self.limits.max_pushdata_bytes {
                violations.push(Bip110Violation::PushdataExceedsLimit {
                    size,
                    max: self.limits.max_pushdata_bytes,
                });
            }
        }

        if let Some(size) = op_return_size {
            if size > self.limits.max_op_return_bytes {
                violations.push(Bip110Violation::OpReturnExceedsLimit {
                    size,
                    max: self.limits.max_op_return_bytes,
                });
            }
        }

        if script_pubkey_size > self.limits.max_script_pubkey_bytes {
            violations.push(Bip110Violation::ScriptPubKeyExceedsLimit {
                size: script_pubkey_size,
                max: self.limits.max_script_pubkey_bytes,
            });
        }

        for &size in witness_elements {
            if size > self.limits.max_witness_element_bytes {
                violations.push(Bip110Violation::WitnessElementExceedsLimit {
                    size,
                    max: self.limits.max_witness_element_bytes,
                });
            }
        }

        if violations.is_empty() {
            Bip110ValidationResult::compliant()
        } else {
            Bip110ValidationResult::non_compliant(violations)
        }
    }
}
