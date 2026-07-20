use serde::{Deserialize, Serialize};

use super::bip110::Bip110Limits;

/// BIP-110 size-policy constants used by the core contract.
///
/// BIP-110 (Reduced Data Temporary Softfork) proposes limits on data embedding in Bitcoin
/// transactions. These constants describe the explicit size-policy subset represented by this
/// crate; they are not a complete consensus or script-validation model:
/// - Max 256-byte pushdata payload per element
/// - 83-byte full output ScriptPubKey for each OP_RETURN output
/// - 34-byte ScriptPubKey policy limit for each non-OP_RETURN output
///
/// See [docs/BIP110_ALIGNMENT.md](https://github.com/Conxian/lib-conxian-core/blob/main/docs/BIP110_ALIGNMENT.md)
pub mod bip110 {
    /// Maximum payload size of one applicable pushdata element in bytes.
    pub const MAX_PUSHDATA_BYTES: usize = 256;

    /// Maximum full output ScriptPubKey size for an OP_RETURN output.
    pub const MAX_OP_RETURN_BYTES: usize = 83;

    /// Maximum ScriptPubKey size for a non-OP_RETURN output.
    pub const MAX_SCRIPT_PUBKEY_BYTES: usize = 34;

    /// Maximum size of one applicable witness stack element in bytes.
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

impl Chain {
    /// Returns the canonical coarse family for this chain.
    ///
    /// Bitcoin-anchored lanes such as Stacks, Liquid, Lightning, and Babylon
    /// intentionally reuse `BitcoinUtxo` in this shared taxonomy. Concrete
    /// adapters still advertise their chain-specific operations, address
    /// formats, and proof capabilities separately.
    pub fn family(&self) -> ChainFamily {
        match self {
            Self::Bitcoin | Self::Stacks | Self::Liquid | Self::Lightning | Self::Babylon => {
                ChainFamily::BitcoinUtxo
            }
            Self::Bob
            | Self::Mezo
            | Self::Citrea
            | Self::Botanix
            | Self::Ethereum
            | Self::Base
            | Self::Arbitrum
            | Self::Optimism
            | Self::Polygon => ChainFamily::Evm,
            Self::CosmosHub | Self::Osmosis | Self::Celestia => ChainFamily::CosmosIbc,
            Self::Solana | Self::Eclipse => ChainFamily::SolanaSvm,
            Self::Aptos | Self::Sui => ChainFamily::Move,
            Self::Polkadot | Self::Kusama => ChainFamily::Substrate,
        }
    }
}

/// Returns the canonical coarse [`Chain`] to [`ChainFamily`] mapping shared by
/// signing, verification, and downstream protocol contracts.
pub fn chain_family_for(chain: &Chain) -> ChainFamily {
    chain.family()
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

/// Result from the core BIP-110 size-policy validator.
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

/// Specific BIP-110 size-policy violation types.
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
                    "OP_RETURN output ScriptPubKey size {} exceeds BIP-110 limit of {} bytes",
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

/// Core BIP-110 size-policy validator for supplied Bitcoin transaction metadata.
///
/// BIP-110 (Reduced Data Temporary Softfork) proposes limits on data embedding:
/// - Maximum 256-byte pushdata payload per element
/// - Maximum 83-byte full output ScriptPubKey for each OP_RETURN output
/// - Maximum 34-byte ScriptPubKey for each non-OP_RETURN output
///
/// This struct validates only the size metadata supplied by a downstream adapter. The adapter is
/// responsible for parsing and classifying transaction/script context, applying any applicable
/// exceptions, and passing every constrained occurrence to the aggregate validator. This type is
/// not a raw transaction parser or a complete consensus/script verifier.
///
/// # Example
///
/// ```rust
/// use lib_conxian_core::control_model::Bip110Compliance;
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

    /// Validates one pushdata element against the configured size limit.
    pub fn validate_pushdata(&self, size: usize) -> Bip110ValidationResult {
        self.validate_single_size(
            size,
            self.limits.max_pushdata_bytes,
            Self::pushdata_violation,
        )
    }

    /// Validates one full OP_RETURN output ScriptPubKey size against the configured limit.
    pub fn validate_op_return(&self, size: usize) -> Bip110ValidationResult {
        self.validate_single_size(
            size,
            self.limits.max_op_return_bytes,
            Self::op_return_violation,
        )
    }

    /// Validates one non-OP_RETURN ScriptPubKey size against the configured limit.
    pub fn validate_script_pubkey(&self, size: usize) -> Bip110ValidationResult {
        self.validate_single_size(
            size,
            self.limits.max_script_pubkey_bytes,
            Self::script_pubkey_violation,
        )
    }

    /// Validates one witness element against the configured size limit.
    pub fn validate_witness_element(&self, size: usize) -> Bip110ValidationResult {
        self.validate_single_size(
            size,
            self.limits.max_witness_element_bytes,
            Self::witness_element_violation,
        )
    }

    /// Validates the legacy aggregate inputs against all configured size limits.
    ///
    /// The `op_return_size` value is treated as one full OP_RETURN output ScriptPubKey size, and
    /// `script_pubkey_size` is treated as one non-OP_RETURN ScriptPubKey size. New callers that
    /// need transaction-wide vectors should use [`crate::control_model::Bip110TransactionShape`].
    pub fn validate_transaction(
        &self,
        pushdatas: &[usize],
        op_return_size: Option<usize>,
        script_pubkey_size: usize,
        witness_elements: &[usize],
    ) -> Bip110ValidationResult {
        let op_return_sizes = op_return_size.into_iter().collect::<Vec<_>>();
        let non_op_return_sizes = [script_pubkey_size];

        self.validate_transaction_shape(
            pushdatas,
            &op_return_sizes,
            &non_op_return_sizes,
            witness_elements,
        )
    }

    /// Validates every size-bearing occurrence represented by the transaction shape.
    pub(crate) fn validate_transaction_shape(
        &self,
        pushdata_sizes: &[usize],
        op_return_script_pubkey_sizes: &[usize],
        non_op_return_script_pubkey_sizes: &[usize],
        witness_element_sizes: &[usize],
    ) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        let mut violations = Vec::new();

        Self::append_violations(
            &mut violations,
            pushdata_sizes.iter().copied(),
            self.limits.max_pushdata_bytes,
            Self::pushdata_violation,
        );
        Self::append_violations(
            &mut violations,
            op_return_script_pubkey_sizes.iter().copied(),
            self.limits.max_op_return_bytes,
            Self::op_return_violation,
        );
        Self::append_violations(
            &mut violations,
            non_op_return_script_pubkey_sizes.iter().copied(),
            self.limits.max_script_pubkey_bytes,
            Self::script_pubkey_violation,
        );
        Self::append_violations(
            &mut violations,
            witness_element_sizes.iter().copied(),
            self.limits.max_witness_element_bytes,
            Self::witness_element_violation,
        );

        if violations.is_empty() {
            Bip110ValidationResult::compliant()
        } else {
            Bip110ValidationResult::non_compliant(violations)
        }
    }

    fn validate_single_size(
        &self,
        size: usize,
        max: usize,
        make_violation: fn(usize, usize) -> Bip110Violation,
    ) -> Bip110ValidationResult {
        if !self.enabled {
            return Bip110ValidationResult::compliant();
        }

        match Self::violation_for_size(size, max, make_violation) {
            Some(violation) => Bip110ValidationResult::non_compliant(vec![violation]),
            None => Bip110ValidationResult::compliant(),
        }
    }

    fn append_violations(
        violations: &mut Vec<Bip110Violation>,
        sizes: impl IntoIterator<Item = usize>,
        max: usize,
        make_violation: fn(usize, usize) -> Bip110Violation,
    ) {
        for size in sizes {
            if let Some(violation) = Self::violation_for_size(size, max, make_violation) {
                violations.push(violation);
            }
        }
    }

    fn violation_for_size(
        size: usize,
        max: usize,
        make_violation: fn(usize, usize) -> Bip110Violation,
    ) -> Option<Bip110Violation> {
        (size > max).then(|| make_violation(size, max))
    }

    fn pushdata_violation(size: usize, max: usize) -> Bip110Violation {
        Bip110Violation::PushdataExceedsLimit { size, max }
    }

    fn op_return_violation(size: usize, max: usize) -> Bip110Violation {
        Bip110Violation::OpReturnExceedsLimit { size, max }
    }

    fn script_pubkey_violation(size: usize, max: usize) -> Bip110Violation {
        Bip110Violation::ScriptPubKeyExceedsLimit { size, max }
    }

    fn witness_element_violation(size: usize, max: usize) -> Bip110Violation {
        Bip110Violation::WitnessElementExceedsLimit { size, max }
    }
}
