//! Platform-neutral contracts for deterministic protocol verification.
//!
//! [`ProtocolVerifier`] defines the boundary between protocol-bearing core types
//! and runtime verifiers.  The trait deliberately does not acquire proofs,
//! query nodes, persist observations, or perform chain-specific orchestration.
//! Implementations belong in downstream adapters, Nexus, or Gateway and must
//! validate capability and request invariants before returning a result.
//!
//! # Examples
//!
//! Construct a chain identifier without coupling the contract to a particular
//! RPC or light-client implementation:
//!
//! ```
//! use lib_conxian_core::verifier::{ChainId, VerifierCapability, VerifierCapabilities};
//! use lib_conxian_core::control_model::ChainFamily;
//!
//! let chain = ChainId::new(ChainFamily::BitcoinUtxo, "mainnet");
//! let capabilities = VerifierCapabilities {
//!     verifier_id: "deterministic-test-verifier".to_string(),
//!     version: "1".to_string(),
//!     supported_chains: vec![chain.clone()],
//!     supported_families: vec![ChainFamily::BitcoinUtxo],
//!     capabilities: vec![VerifierCapability::LatestVerifiedBlock],
//!     proof_formats: Vec::new(),
//!     verification_classes: Vec::new(),
//!     finality_classes: Vec::new(),
//!     trust_tiers: Vec::new(),
//! };
//!
//! assert!(capabilities.supports_chain(&chain));
//! assert!(capabilities.supports(VerifierCapability::LatestVerifiedBlock));
//! ```
//!
//! Validate proof input before passing it to a runtime verifier:
//!
//! ```
//! use lib_conxian_core::verifier::{
//!     ChainId, ChainStateReference, ProofData, ProofFormat, ProofVerificationRequest,
//! };
//! use lib_conxian_core::control_model::ChainFamily;
//!
//! let request = ProofVerificationRequest::new(
//!     ChainId::new(ChainFamily::Evm, "ethereum-mainnet"),
//!     ChainStateReference::new("0xblock", 100, Some("0xroot".to_string())),
//!     ProofData::new(ProofFormat::Merkle, vec![1, 2, 3]),
//! );
//!
//! assert!(request.validate().is_ok());
//! ```
//!
//! Model finality as an explicit status rather than assuming that a successful
//! proof is automatically final:
//!
//! ```
//! use lib_conxian_core::verifier::{
//!     validate_finality_transition, TransactionFinalityStatus,
//! };
//!
//! let pending = TransactionFinalityStatus::Pending;
//! let confirmed = TransactionFinalityStatus::Confirmed { confirmations: 2 };
//! assert!(validate_finality_transition(&pending, &confirmed).is_ok());
//! assert!(!confirmed.is_final());
//! ```

use crate::control_model::{
    validate_trust_tier_policy, Chain, ChainFamily, FinalityClass, ProofEnvelope, TrustTier,
    VerificationClass, VerificationStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies a chain instance while preserving the existing [`Chain`] and
/// [`ChainFamily`] taxonomies.
///
/// `chain` is optional so downstream integrations can represent rails that are
/// not yet enumerated by the core `Chain` model (for example a private network
/// or a newly introduced chain) without changing that public enum. `network`
/// remains the stable routing identity used by the verifier contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainId {
    /// Existing core family classification for the chain.
    pub family: ChainFamily,
    /// Optional known chain variant from the existing control model.
    pub chain: Option<Chain>,
    /// Network or deployment identifier, such as `mainnet` or
    /// `ethereum-mainnet`.
    pub network: String,
}

impl ChainId {
    /// Creates a chain identifier for a family and deployment name.
    pub fn new(family: ChainFamily, network: impl Into<String>) -> Self {
        Self {
            family,
            chain: None,
            network: network.into(),
        }
    }

    /// Creates a chain identifier tied to an existing [`Chain`] variant.
    pub fn from_chain(chain: Chain, family: ChainFamily, network: impl Into<String>) -> Self {
        Self {
            family,
            chain: Some(chain),
            network: network.into(),
        }
    }

    /// Validates the structural identity required by every verifier request.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if self.network.trim().is_empty() {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "chain network must not be empty".to_string(),
            });
        }
        Ok(())
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(chain) = &self.chain {
            write!(f, "{:?}:{}", chain, self.network)
        } else {
            write!(f, "{:?}:{}", self.family, self.network)
        }
    }
}

/// Format of the proof bytes supplied to a verifier.
///
/// This is a representation taxonomy, not a verification policy taxonomy;
/// policy continues to use [`VerificationClass`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProofFormat {
    HeaderChain,
    Merkle,
    StateRoot,
    TransactionInclusion,
    ZkProof,
    Custom(String),
}

impl ProofFormat {
    /// Validates a proof format descriptor.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if let Self::Custom(name) = self {
            if name.trim().is_empty() {
                return Err(ProtocolVerifierError::MalformedProof {
                    reason: "custom proof format must not be empty".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl fmt::Display for ProofFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HeaderChain => write!(f, "header_chain"),
            Self::Merkle => write!(f, "merkle"),
            Self::StateRoot => write!(f, "state_root"),
            Self::TransactionInclusion => write!(f, "transaction_inclusion"),
            Self::ZkProof => write!(f, "zk_proof"),
            Self::Custom(name) => write!(f, "custom({name})"),
        }
    }
}

/// Proof bytes and their format, without prescribing a chain-specific proof
/// encoding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofData {
    pub format: ProofFormat,
    pub bytes: Vec<u8>,
    pub evidence_hash: Option<String>,
}

impl ProofData {
    /// Creates proof data with no separately supplied evidence hash.
    pub fn new(format: ProofFormat, bytes: Vec<u8>) -> Self {
        Self {
            format,
            bytes,
            evidence_hash: None,
        }
    }

    /// Associates a caller-computed evidence hash with the proof bytes.
    pub fn with_evidence_hash(mut self, evidence_hash: impl Into<String>) -> Self {
        self.evidence_hash = Some(evidence_hash.into());
        self
    }

    /// Validates proof bytes and their format without attempting cryptographic
    /// verification.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.format.validate()?;
        if self.bytes.is_empty() {
            return Err(ProtocolVerifierError::InsufficientProofData {
                required: "at least one proof byte".to_string(),
            });
        }
        if self
            .evidence_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(ProtocolVerifierError::MalformedProof {
                reason: "evidence hash must not be empty when supplied".to_string(),
            });
        }
        Ok(())
    }
}

/// A block and optional state-root reference targeted by a proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainStateReference {
    pub block_hash: String,
    pub block_height: u64,
    pub state_root: Option<String>,
}

impl ChainStateReference {
    pub fn new(
        block_hash: impl Into<String>,
        block_height: u64,
        state_root: Option<String>,
    ) -> Self {
        Self {
            block_hash: block_hash.into(),
            block_height,
            state_root,
        }
    }

    /// Validates the structural state reference.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if self.block_hash.trim().is_empty() {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "state block hash must not be empty".to_string(),
            });
        }
        if self
            .state_root
            .as_ref()
            .is_some_and(|root| root.trim().is_empty())
        {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "state root must not be empty when supplied".to_string(),
            });
        }
        Ok(())
    }
}

/// Input to [`ProtocolVerifier::verify_chain_state`].
///
/// An optional [`ProofEnvelope`] carries existing bridge metadata when a proof
/// comes from a cross-domain flow. The envelope is validated when present; it
/// is not required for a native chain-state proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationRequest {
    pub chain: ChainId,
    pub state: ChainStateReference,
    pub proof: ProofData,
    pub envelope: Option<ProofEnvelope>,
}

impl ProofVerificationRequest {
    pub fn new(chain: ChainId, state: ChainStateReference, proof: ProofData) -> Self {
        Self {
            chain,
            state,
            proof,
            envelope: None,
        }
    }

    /// Adds existing bridge metadata to the proof request.
    pub fn with_envelope(mut self, envelope: ProofEnvelope) -> Self {
        self.envelope = Some(envelope);
        self
    }

    /// Validates the request against the current clock.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.validate_at(Utc::now())
    }

    /// Validates the request at a caller-supplied time for deterministic tests.
    pub fn validate_at(&self, now: DateTime<Utc>) -> Result<(), ProtocolVerifierError> {
        self.chain.validate()?;
        self.state.validate()?;
        self.proof.validate()?;
        if let Some(envelope) = &self.envelope {
            validate_proof_envelope_at(envelope, now)?;
        }
        Ok(())
    }
}

/// Alias matching the chain-state terminology used by downstream integrations.
pub type ChainStateVerificationRequest = ProofVerificationRequest;

/// Validates bridge metadata using the current clock.
pub fn validate_proof_envelope(envelope: &ProofEnvelope) -> Result<(), ProtocolVerifierError> {
    validate_proof_envelope_at(envelope, Utc::now())
}

/// Validates bridge metadata at a caller-supplied time.
pub fn validate_proof_envelope_at(
    envelope: &ProofEnvelope,
    now: DateTime<Utc>,
) -> Result<(), ProtocolVerifierError> {
    for (name, value) in [
        ("system version", envelope.system_version.as_str()),
        ("source chain id", envelope.source_chain_id.as_str()),
        (
            "destination chain id",
            envelope.destination_chain_id.as_str(),
        ),
        ("proof reference", envelope.proof_ref.as_str()),
        ("evidence hash", envelope.evidence_hash.as_str()),
        ("verifier set reference", envelope.verifier_set_ref.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(ProtocolVerifierError::MalformedProof {
                reason: format!("proof envelope {name} must not be empty"),
            });
        }
    }

    if envelope
        .evidence_uri
        .as_ref()
        .is_some_and(|uri| uri.trim().is_empty())
    {
        return Err(ProtocolVerifierError::MalformedProof {
            reason: "proof envelope evidence URI must not be empty when supplied".to_string(),
        });
    }

    if envelope.expires_at <= envelope.observed_at {
        return Err(ProtocolVerifierError::MalformedProof {
            reason: "proof envelope expiry must be after observation time".to_string(),
        });
    }

    if envelope.expires_at <= now {
        return Err(ProtocolVerifierError::ExpiredEvidence {
            reference: envelope.proof_ref.clone(),
            expires_at: envelope.expires_at,
        });
    }

    validate_trust_tier_policy(
        envelope.trust_tier.clone(),
        envelope.verification_class.clone(),
    )
    .map_err(|reason| ProtocolVerifierError::PolicyBlocked {
        trust_tier: envelope.trust_tier.clone(),
        reason,
    })?;

    if envelope.verification_status != VerificationStatus::Verified {
        return Err(ProtocolVerifierError::PolicyBlocked {
            trust_tier: envelope.trust_tier.clone(),
            reason: format!(
                "proof envelope status {:?} is not accepted as verified evidence",
                envelope.verification_status
            ),
        });
    }

    Ok(())
}

/// Capability advertised by a verifier implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifierCapability {
    StateProofVerification,
    LatestVerifiedBlock,
    TransactionFinality,
}

impl fmt::Display for VerifierCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StateProofVerification => write!(f, "state_proof_verification"),
            Self::LatestVerifiedBlock => write!(f, "latest_verified_block"),
            Self::TransactionFinality => write!(f, "transaction_finality"),
        }
    }
}

/// Machine-readable declaration of the chains, proof formats, and policies a
/// verifier can actually support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifierCapabilities {
    pub verifier_id: String,
    pub version: String,
    pub supported_chains: Vec<ChainId>,
    pub supported_families: Vec<ChainFamily>,
    pub capabilities: Vec<VerifierCapability>,
    pub proof_formats: Vec<ProofFormat>,
    pub verification_classes: Vec<VerificationClass>,
    pub finality_classes: Vec<FinalityClass>,
    pub trust_tiers: Vec<TrustTier>,
}

/// Alias for callers that use the protocol-advertisement terminology.
pub type CapabilityAdvertisement = VerifierCapabilities;

impl VerifierCapabilities {
    /// Returns whether this verifier advertises a capability.
    pub fn supports(&self, capability: VerifierCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Returns whether this verifier advertises the exact chain or its family.
    pub fn supports_chain(&self, chain: &ChainId) -> bool {
        self.supported_chains
            .iter()
            .any(|candidate| candidate == chain)
            || self.supported_families.contains(&chain.family)
    }

    /// Returns whether this verifier advertises a proof format.
    pub fn supports_proof_format(&self, format: &ProofFormat) -> bool {
        self.proof_formats.contains(format)
    }

    /// Validates the internal consistency of the advertisement.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if self.verifier_id.trim().is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verifier id must not be empty".to_string(),
            });
        }
        if self.version.trim().is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verifier version must not be empty".to_string(),
            });
        }
        if self.supported_chains.is_empty() && self.supported_families.is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verifier must advertise at least one chain or family".to_string(),
            });
        }
        if self.capabilities.is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verifier must advertise at least one capability".to_string(),
            });
        }
        if self.trust_tiers.is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verifier must advertise at least one trust tier".to_string(),
            });
        }

        for chain in &self.supported_chains {
            chain.validate()?;
            if !self.supported_families.contains(&chain.family) {
                return Err(ProtocolVerifierError::InvariantViolation {
                    reason: format!(
                        "supported chain {} is missing its family advertisement",
                        chain
                    ),
                });
            }
        }

        if self.supports(VerifierCapability::StateProofVerification)
            && self.proof_formats.is_empty()
        {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "state proof verification requires at least one proof format".to_string(),
            });
        }

        if self.supports(VerifierCapability::TransactionFinality)
            && self.finality_classes.is_empty()
        {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "transaction finality requires at least one finality class".to_string(),
            });
        }

        if self.trust_tiers.contains(&TrustTier::Strict)
            && !self
                .verification_classes
                .contains(&VerificationClass::LightClient)
        {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "strict trust support requires light-client verification".to_string(),
            });
        }

        Ok(())
    }

    /// Fails closed when a chain or capability was not advertised.
    pub fn require_capability(
        &self,
        chain: &ChainId,
        capability: VerifierCapability,
    ) -> Result<(), ProtocolVerifierError> {
        self.validate()?;
        chain.validate()?;
        if !self.supports_chain(chain) {
            return Err(ProtocolVerifierError::UnsupportedChain {
                chain: chain.clone(),
            });
        }
        if !self.supports(capability.clone()) {
            return Err(ProtocolVerifierError::UnsupportedCapability {
                chain: chain.clone(),
                capability,
            });
        }
        Ok(())
    }

    /// Fails closed when a proof representation was not advertised.
    pub fn require_proof_format(
        &self,
        chain: &ChainId,
        format: &ProofFormat,
    ) -> Result<(), ProtocolVerifierError> {
        self.require_capability(chain, VerifierCapability::StateProofVerification)?;
        if !self.supports_proof_format(format) {
            return Err(ProtocolVerifierError::UnsupportedProofFormat {
                chain: chain.clone(),
                format: format.clone(),
            });
        }
        Ok(())
    }
}

/// Provenance for a verified result or block reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationProvenance {
    pub verifier_id: String,
    pub evidence_ref: Option<String>,
    pub verified_at: DateTime<Utc>,
}

impl VerificationProvenance {
    /// Validates provenance metadata.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if self.verifier_id.trim().is_empty() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verification provenance verifier id must not be empty".to_string(),
            });
        }
        if self
            .evidence_ref
            .as_ref()
            .is_some_and(|reference| reference.trim().is_empty())
        {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "verification evidence reference must not be empty when supplied"
                    .to_string(),
            });
        }
        Ok(())
    }
}

/// Platform-neutral block header data returned by a verifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub hash: String,
    pub parent_hash: Option<String>,
    pub height: u64,
    pub timestamp: DateTime<Utc>,
    pub state_root: Option<String>,
}

impl BlockHeader {
    /// Validates the structural block header fields.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        if self.hash.trim().is_empty() {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "block header hash must not be empty".to_string(),
            });
        }
        if self
            .parent_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "block header parent hash must not be empty when supplied".to_string(),
            });
        }
        if self
            .state_root
            .as_ref()
            .is_some_and(|root| root.trim().is_empty())
        {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "block header state root must not be empty when supplied".to_string(),
            });
        }
        Ok(())
    }
}

/// The latest block a verifier has validated, together with finality and
/// trust metadata. This is a reference, not a promise that the core crate can
/// acquire a newer block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LatestVerifiedBlock {
    pub chain: ChainId,
    pub header: BlockHeader,
    pub finality_class: FinalityClass,
    pub confirmations: u32,
    pub verification_class: VerificationClass,
    pub trust_tier: TrustTier,
    pub verification_status: VerificationStatus,
    pub provenance: VerificationProvenance,
}

/// Alias for callers that refer to a verified block as a reference.
pub type VerifiedBlockReference = LatestVerifiedBlock;

/// Backwards-friendly descriptive alias for block-reference consumers.
pub type BlockReference = LatestVerifiedBlock;

impl LatestVerifiedBlock {
    /// Returns whether this reference is safe to treat as verified evidence.
    pub fn is_verified(&self) -> bool {
        self.verification_status == VerificationStatus::Verified
            && self.trust_tier.is_production_allowed()
    }

    /// Validates the block reference and its trust mapping.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.chain.validate()?;
        self.header.validate()?;
        self.provenance.validate()?;
        validate_trust_tier_policy(self.trust_tier.clone(), self.verification_class.clone())
            .map_err(|reason| ProtocolVerifierError::PolicyBlocked {
                trust_tier: self.trust_tier.clone(),
                reason,
            })?;
        if self.verification_status == VerificationStatus::Blocked {
            return Err(ProtocolVerifierError::PolicyBlocked {
                trust_tier: self.trust_tier.clone(),
                reason: "latest verified block is blocked".to_string(),
            });
        }
        Ok(())
    }
}

/// Result returned after a state proof has been accepted by a verifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofVerificationResult {
    pub chain: ChainId,
    pub state: ChainStateReference,
    pub proof_format: ProofFormat,
    pub verified_block: LatestVerifiedBlock,
}

impl ProofVerificationResult {
    pub fn is_verified(&self) -> bool {
        self.verified_block.is_verified()
    }

    /// Validates the result's state/block and chain invariants.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.chain.validate()?;
        self.state.validate()?;
        self.proof_format.validate()?;
        self.verified_block.validate()?;

        if self.chain != self.verified_block.chain {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "proof result chain does not match verified block chain".to_string(),
            });
        }
        if self.state.block_hash != self.verified_block.header.hash
            || self.state.block_height != self.verified_block.header.height
        {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "proof state reference does not match verified block".to_string(),
            });
        }
        if let (Some(state_root), Some(header_root)) = (
            &self.state.state_root,
            &self.verified_block.header.state_root,
        ) {
            if state_root != header_root {
                return Err(ProtocolVerifierError::InvariantViolation {
                    reason: "proof state root does not match verified block state root".to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Request for a transaction-finality decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionFinalityRequest {
    pub chain: ChainId,
    pub transaction_id: String,
    pub min_confirmations: u32,
    pub require_finality: bool,
}

impl TransactionFinalityRequest {
    pub fn new(
        chain: ChainId,
        transaction_id: impl Into<String>,
        min_confirmations: u32,
        require_finality: bool,
    ) -> Self {
        Self {
            chain,
            transaction_id: transaction_id.into(),
            min_confirmations,
            require_finality,
        }
    }

    /// Validates the request before evidence acquisition or finality policy.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.chain.validate()?;
        if self.transaction_id.trim().is_empty() {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "transaction id must not be empty".to_string(),
            });
        }
        Ok(())
    }
}

/// Explicit transaction lifecycle status returned by a verifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionFinalityStatus {
    Pending,
    Confirmed { confirmations: u32 },
    Finalized { confirmations: u32 },
    Reorged,
    Rejected,
}

impl TransactionFinalityStatus {
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Finalized { .. })
    }

    pub fn confirmations(&self) -> u32 {
        match self {
            Self::Pending | Self::Reorged | Self::Rejected => 0,
            Self::Confirmed { confirmations } | Self::Finalized { confirmations } => *confirmations,
        }
    }
}

impl fmt::Display for TransactionFinalityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Confirmed { confirmations } => write!(f, "confirmed({confirmations})"),
            Self::Finalized { confirmations } => write!(f, "finalized({confirmations})"),
            Self::Reorged => write!(f, "reorged"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

/// Result of checking a transaction's finality at a verified block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionFinalityResult {
    pub chain: ChainId,
    pub transaction_id: String,
    pub status: TransactionFinalityStatus,
    pub finality_class: FinalityClass,
    pub required_confirmations: u32,
    pub observed_confirmations: u32,
    pub latest_block: Option<LatestVerifiedBlock>,
    pub verification_class: VerificationClass,
    pub trust_tier: TrustTier,
    pub verification_status: VerificationStatus,
    pub provenance: VerificationProvenance,
}

impl TransactionFinalityResult {
    pub fn is_final(&self) -> bool {
        self.status.is_final()
            && self.verification_status == VerificationStatus::Verified
            && self.trust_tier.is_production_allowed()
    }

    /// Converts a non-final result into the typed failure used by callers that
    /// require finality.
    pub fn require_final(&self) -> Result<(), ProtocolVerifierError> {
        if self.is_final() {
            Ok(())
        } else {
            Err(ProtocolVerifierError::NonFinalState {
                transaction_id: self.transaction_id.clone(),
                status: self.status.clone(),
                confirmations: self.observed_confirmations,
                required_confirmations: self.required_confirmations,
            })
        }
    }

    /// Validates status, confirmation, block, and trust invariants.
    pub fn validate(&self) -> Result<(), ProtocolVerifierError> {
        self.chain.validate()?;
        if self.transaction_id.trim().is_empty() {
            return Err(ProtocolVerifierError::InvalidRequest {
                reason: "transaction id must not be empty".to_string(),
            });
        }
        self.provenance.validate()?;
        validate_trust_tier_policy(self.trust_tier.clone(), self.verification_class.clone())
            .map_err(|reason| ProtocolVerifierError::PolicyBlocked {
                trust_tier: self.trust_tier.clone(),
                reason,
            })?;
        if self.verification_status == VerificationStatus::Blocked {
            return Err(ProtocolVerifierError::PolicyBlocked {
                trust_tier: self.trust_tier.clone(),
                reason: "transaction finality result is blocked".to_string(),
            });
        }

        if self.observed_confirmations != self.status.confirmations() {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "observed confirmations do not match transaction status".to_string(),
            });
        }
        if matches!(
            self.status,
            TransactionFinalityStatus::Confirmed { confirmations: 0 }
                | TransactionFinalityStatus::Finalized { confirmations: 0 }
        ) {
            return Err(ProtocolVerifierError::InvariantViolation {
                reason: "confirmed or finalized transaction must have at least one confirmation"
                    .to_string(),
            });
        }
        if let TransactionFinalityStatus::Finalized { confirmations } = self.status {
            if confirmations < self.required_confirmations {
                return Err(ProtocolVerifierError::InvariantViolation {
                    reason: "finalized transaction has fewer confirmations than required"
                        .to_string(),
                });
            }
        }

        if let Some(block) = &self.latest_block {
            block.validate()?;
            if block.chain != self.chain {
                return Err(ProtocolVerifierError::InvariantViolation {
                    reason: "transaction finality block does not match transaction chain"
                        .to_string(),
                });
            }
        }
        Ok(())
    }
}

/// Validates a finality result against the request that produced it.
///
/// This helper lets downstream implementations preserve a non-final status
/// when the caller only requested observation, while returning a typed
/// [`ProtocolVerifierError::NonFinalState`] when finality was required.
pub fn validate_finality_result(
    request: &TransactionFinalityRequest,
    result: &TransactionFinalityResult,
) -> Result<(), ProtocolVerifierError> {
    request.validate()?;
    result.validate()?;

    if request.chain != result.chain {
        return Err(ProtocolVerifierError::InvariantViolation {
            reason: "finality result chain does not match request chain".to_string(),
        });
    }
    if request.transaction_id != result.transaction_id {
        return Err(ProtocolVerifierError::InvariantViolation {
            reason: "finality result transaction does not match request".to_string(),
        });
    }
    if request.require_finality
        && (result.observed_confirmations < request.min_confirmations || !result.is_final())
    {
        return Err(ProtocolVerifierError::NonFinalState {
            transaction_id: result.transaction_id.clone(),
            status: result.status.clone(),
            confirmations: result.observed_confirmations,
            required_confirmations: request.min_confirmations,
        });
    }
    Ok(())
}

/// Checks that a finality status transition is monotonic and cannot silently
/// move a terminal status back into an earlier state.
pub fn validate_finality_transition(
    previous: &TransactionFinalityStatus,
    next: &TransactionFinalityStatus,
) -> Result<(), ProtocolVerifierError> {
    let valid = match (previous, next) {
        (TransactionFinalityStatus::Pending, _) => true,
        (
            TransactionFinalityStatus::Confirmed {
                confirmations: previous_confirmations,
            },
            TransactionFinalityStatus::Confirmed {
                confirmations: next_confirmations,
            },
        ) => next_confirmations >= previous_confirmations,
        (
            TransactionFinalityStatus::Confirmed { .. },
            TransactionFinalityStatus::Finalized { .. },
        )
        | (TransactionFinalityStatus::Confirmed { .. }, TransactionFinalityStatus::Reorged)
        | (TransactionFinalityStatus::Confirmed { .. }, TransactionFinalityStatus::Rejected) => {
            true
        }
        (
            TransactionFinalityStatus::Finalized {
                confirmations: previous_confirmations,
            },
            TransactionFinalityStatus::Finalized {
                confirmations: next_confirmations,
            },
        ) => next_confirmations >= previous_confirmations,
        (TransactionFinalityStatus::Reorged, TransactionFinalityStatus::Reorged)
        | (TransactionFinalityStatus::Rejected, TransactionFinalityStatus::Rejected) => true,
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(ProtocolVerifierError::InvariantViolation {
            reason: format!("invalid finality transition: {previous} -> {next}"),
        })
    }
}

/// Typed failures returned by protocol verifier implementations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtocolVerifierError {
    UnsupportedVerifier {
        verifier_id: String,
        reason: String,
    },
    UnsupportedChain {
        chain: ChainId,
    },
    UnsupportedCapability {
        chain: ChainId,
        capability: VerifierCapability,
    },
    UnsupportedProofFormat {
        chain: ChainId,
        format: ProofFormat,
    },
    MalformedProof {
        reason: String,
    },
    InsufficientProofData {
        required: String,
    },
    InvalidProof {
        reason: String,
    },
    UnavailableEvidence {
        reference: String,
    },
    ExpiredEvidence {
        reference: String,
        expires_at: DateTime<Utc>,
    },
    StaleReference {
        chain: ChainId,
        expected_height: u64,
        actual_height: u64,
    },
    NonFinalState {
        transaction_id: String,
        status: TransactionFinalityStatus,
        confirmations: u32,
        required_confirmations: u32,
    },
    PolicyBlocked {
        trust_tier: TrustTier,
        reason: String,
    },
    InvalidRequest {
        reason: String,
    },
    InvariantViolation {
        reason: String,
    },
}

impl fmt::Display for ProtocolVerifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVerifier { verifier_id, reason } => {
                write!(f, "unsupported verifier {verifier_id}: {reason}")
            }
            Self::UnsupportedChain { chain } => write!(f, "unsupported chain: {chain}"),
            Self::UnsupportedCapability { chain, capability } => {
                write!(f, "unsupported capability {capability} for chain {chain}")
            }
            Self::UnsupportedProofFormat { chain, format } => {
                write!(f, "unsupported proof format {format} for chain {chain}")
            }
            Self::MalformedProof { reason } => write!(f, "malformed proof: {reason}"),
            Self::InsufficientProofData { required } => {
                write!(f, "insufficient proof data; required {required}")
            }
            Self::InvalidProof { reason } => write!(f, "invalid proof: {reason}"),
            Self::UnavailableEvidence { reference } => {
                write!(f, "verification evidence unavailable: {reference}")
            }
            Self::ExpiredEvidence {
                reference,
                expires_at,
            } => write!(f, "verification evidence {reference} expired at {expires_at}"),
            Self::StaleReference {
                chain,
                expected_height,
                actual_height,
            } => write!(
                f,
                "stale block reference for {chain}: expected height {expected_height}, got {actual_height}"
            ),
            Self::NonFinalState {
                transaction_id,
                status,
                confirmations,
                required_confirmations,
            } => write!(
                f,
                "transaction {transaction_id} is {status} with {confirmations}/{required_confirmations} confirmations"
            ),
            Self::PolicyBlocked { trust_tier, reason } => {
                write!(f, "policy blocked for trust tier {trust_tier:?}: {reason}")
            }
            Self::InvalidRequest { reason } => write!(f, "invalid verifier request: {reason}"),
            Self::InvariantViolation { reason } => write!(f, "verifier invariant violation: {reason}"),
        }
    }
}

impl std::error::Error for ProtocolVerifierError {}

/// Platform-neutral verification contract.
///
/// Implementations must remain fail-closed: unsupported chains, capabilities,
/// malformed proofs, unavailable evidence, stale references, and policy
/// violations must be returned as [`ProtocolVerifierError`] values rather than
/// represented as `Ok(true)`. Network I/O, proof acquisition, persistence, and
/// live orchestration are intentionally outside this trait.
pub trait ProtocolVerifier: Send + Sync {
    /// Advertises the implementation's supported chains and operations.
    fn capabilities(&self) -> &VerifierCapabilities;

    /// Verifies a chain-state proof and returns its verified block provenance.
    fn verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError>;

    /// Returns the latest block/reference already verified by the implementation.
    fn get_latest_verified_block(
        &self,
        chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError>;

    /// Evaluates transaction finality without assuming that confirmation alone
    /// implies deterministic finality.
    fn verify_transaction_finality(
        &self,
        request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError>;

    /// Shared capability check for downstream implementations.
    fn ensure_capability(
        &self,
        chain: &ChainId,
        capability: VerifierCapability,
    ) -> Result<(), ProtocolVerifierError> {
        self.capabilities().require_capability(chain, capability)
    }

    /// Shared proof-format check for downstream implementations.
    fn ensure_proof_format(
        &self,
        chain: &ChainId,
        format: &ProofFormat,
    ) -> Result<(), ProtocolVerifierError> {
        self.capabilities().require_proof_format(chain, format)
    }

    /// Shared finality-result check for downstream implementations.
    fn validate_finality_result(
        &self,
        request: &TransactionFinalityRequest,
        result: &TransactionFinalityResult,
    ) -> Result<(), ProtocolVerifierError> {
        validate_finality_result(request, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn timestamp(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0)
            .single()
            .expect("valid timestamp")
    }

    fn chain() -> ChainId {
        ChainId::new(ChainFamily::BitcoinUtxo, "mainnet")
    }

    fn block(chain: ChainId, status: VerificationStatus) -> LatestVerifiedBlock {
        LatestVerifiedBlock {
            chain,
            header: BlockHeader {
                hash: "block-hash".to_string(),
                parent_hash: Some("parent-hash".to_string()),
                height: 42,
                timestamp: timestamp(1_000),
                state_root: Some("state-root".to_string()),
            },
            finality_class: FinalityClass::Probabilistic,
            confirmations: 6,
            verification_class: VerificationClass::LightClient,
            trust_tier: TrustTier::Strict,
            verification_status: status,
            provenance: VerificationProvenance {
                verifier_id: "mock".to_string(),
                evidence_ref: Some("proof-1".to_string()),
                verified_at: timestamp(1_010),
            },
        }
    }

    #[test]
    fn proof_request_rejects_empty_proof_data() {
        let request = ProofVerificationRequest::new(
            chain(),
            ChainStateReference::new("block-hash", 42, None),
            ProofData::new(ProofFormat::Merkle, Vec::new()),
        );

        assert!(matches!(
            request.validate_at(timestamp(2_000)),
            Err(ProtocolVerifierError::InsufficientProofData { .. })
        ));
    }

    #[test]
    fn proof_request_rejects_expired_envelope() {
        let envelope = ProofEnvelope {
            system: crate::control_model::BridgeSystem::Ibc,
            system_version: "v1".to_string(),
            trust_tier: TrustTier::Strict,
            verification_class: VerificationClass::LightClient,
            source_chain_id: "bitcoin-mainnet".to_string(),
            destination_chain_id: "stacks-mainnet".to_string(),
            finality_class: FinalityClass::Probabilistic,
            min_confirmations: 6,
            observed_at: timestamp(1_000),
            expires_at: timestamp(1_100),
            proof_ref: "proof-1".to_string(),
            evidence_hash: "hash-1".to_string(),
            evidence_uri: None,
            verifier_set_ref: "set-1".to_string(),
            security_params: serde_json::json!({}),
            verification_status: VerificationStatus::Verified,
            verification_reason: None,
        };
        let request = ProofVerificationRequest::new(
            chain(),
            ChainStateReference::new("block-hash", 42, None),
            ProofData::new(ProofFormat::HeaderChain, vec![1]),
        )
        .with_envelope(envelope);

        assert!(matches!(
            request.validate_at(timestamp(2_000)),
            Err(ProtocolVerifierError::ExpiredEvidence { .. })
        ));
    }

    #[test]
    fn capabilities_fail_closed_for_unsupported_chain_and_format() {
        let bitcoin = chain();
        let capabilities = VerifierCapabilities {
            verifier_id: "mock".to_string(),
            version: "1".to_string(),
            supported_chains: vec![bitcoin.clone()],
            supported_families: vec![ChainFamily::BitcoinUtxo],
            capabilities: vec![VerifierCapability::StateProofVerification],
            proof_formats: vec![ProofFormat::HeaderChain],
            verification_classes: vec![VerificationClass::LightClient],
            finality_classes: vec![],
            trust_tiers: vec![TrustTier::Strict],
        };
        let unsupported_chain = ChainId::new(ChainFamily::Evm, "ethereum-mainnet");

        assert!(matches!(
            capabilities.require_capability(
                &unsupported_chain,
                VerifierCapability::StateProofVerification
            ),
            Err(ProtocolVerifierError::UnsupportedChain { .. })
        ));
        assert!(matches!(
            capabilities.require_proof_format(&bitcoin, &ProofFormat::Merkle),
            Err(ProtocolVerifierError::UnsupportedProofFormat { .. })
        ));
    }

    #[test]
    fn latest_block_and_proof_result_enforce_reference_invariants() {
        let verified_block = block(chain(), VerificationStatus::Verified);
        assert!(verified_block.validate().is_ok());
        assert!(verified_block.is_verified());

        let result = ProofVerificationResult {
            chain: chain(),
            state: ChainStateReference::new("block-hash", 42, Some("state-root".to_string())),
            proof_format: ProofFormat::HeaderChain,
            verified_block,
        };
        assert!(result.validate().is_ok());

        let mut invalid = result;
        invalid.state.block_hash = "different-block".to_string();
        assert!(matches!(
            invalid.validate(),
            Err(ProtocolVerifierError::InvariantViolation { .. })
        ));
    }

    #[test]
    fn finality_transitions_are_monotonic_and_non_final_is_typed() {
        let pending = TransactionFinalityStatus::Pending;
        let confirmed = TransactionFinalityStatus::Confirmed { confirmations: 2 };
        let finalized = TransactionFinalityStatus::Finalized { confirmations: 6 };

        assert!(validate_finality_transition(&pending, &confirmed).is_ok());
        assert!(validate_finality_transition(&confirmed, &finalized).is_ok());
        assert!(validate_finality_transition(
            &confirmed,
            &TransactionFinalityStatus::Confirmed { confirmations: 1 }
        )
        .is_err());
        assert!(validate_finality_transition(&finalized, &pending).is_err());

        let result = TransactionFinalityResult {
            chain: chain(),
            transaction_id: "tx-1".to_string(),
            status: confirmed,
            finality_class: FinalityClass::Probabilistic,
            required_confirmations: 6,
            observed_confirmations: 2,
            latest_block: None,
            verification_class: VerificationClass::LightClient,
            trust_tier: TrustTier::Strict,
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: "mock".to_string(),
                evidence_ref: None,
                verified_at: timestamp(1_010),
            },
        };
        assert!(result.validate().is_ok());
        assert!(matches!(
            result.require_final(),
            Err(ProtocolVerifierError::NonFinalState { .. })
        ));
    }

    #[test]
    fn trust_mapping_rejects_strict_external_quorum_and_observer_only() {
        assert!(
            validate_trust_tier_policy(TrustTier::Strict, VerificationClass::ExternalQuorum)
                .is_err()
        );
        assert!(validate_trust_tier_policy(
            TrustTier::ObserverOnly,
            VerificationClass::NativeObservation
        )
        .is_err());

        let mut capabilities = VerifierCapabilities {
            verifier_id: "mock".to_string(),
            version: "1".to_string(),
            supported_chains: vec![chain()],
            supported_families: vec![ChainFamily::BitcoinUtxo],
            capabilities: vec![VerifierCapability::LatestVerifiedBlock],
            proof_formats: vec![],
            verification_classes: vec![VerificationClass::ExternalQuorum],
            finality_classes: vec![],
            trust_tiers: vec![TrustTier::Strict],
        };
        assert!(matches!(
            capabilities.validate(),
            Err(ProtocolVerifierError::InvariantViolation { .. })
        ));

        capabilities.verification_classes = vec![VerificationClass::LightClient];
        assert!(capabilities.validate().is_ok());
    }
}
