use serde::{Deserialize, Serialize};

/// Canonical authority classes used by wallet and protected-action controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WalletAuthorityClass {
    WalletOwner,
    Delegate,
    Guardian,
    ServiceOperator,
    Automation,
    Auditor,
}

/// Metadata describing the actor class and wallet domain for an authorization decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletAuthority {
    pub authority_id: String,
    pub wallet_id: String,
    pub class: WalletAuthorityClass,
}

/// Lifecycle states for actions requiring explicit policy and/or human controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProtectedActionLifecycleState {
    Draft,
    PendingAuthorization,
    Timelocked,
    ReadyForExecution,
    Executed,
    Rejected,
    Cancelled,
    Expired,
    Failed,
}

/// Lifecycle states for normalized external triggers before and after intake checks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerLifecycleState {
    Received,
    Validated,
    Rejected,
    MaterializedAsPendingAction,
}

/// Lifecycle states for actions waiting for controls (quorum/timelock) before execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PendingActionLifecycleState {
    AwaitingQuorum,
    QuorumSatisfied,
    Timelocked,
    Ready,
    Executing,
    Executed,
    Cancelled,
    Expired,
    Failed,
}

/// Timelock invariants used to guarantee a minimum waiting period before execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelockInvariant {
    pub created_at_block: u64,
    pub timelock_blocks: u32,
    pub not_before_block: u64,
}

impl TimelockInvariant {
    pub fn new(created_at_block: u64, timelock_blocks: u32) -> Self {
        Self {
            created_at_block,
            timelock_blocks,
            not_before_block: created_at_block.saturating_add(timelock_blocks as u64),
        }
    }
}

/// Quorum invariants used for threshold-style policy controls.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuorumInvariant {
    pub approvals_required: u16,
    pub eligible_approvers: u16,
}

impl QuorumInvariant {
    pub fn new(approvals_required: u16, eligible_approvers: u16) -> Self {
        Self {
            approvals_required,
            eligible_approvers,
        }
    }
}

/// Combined control invariants for protected-action gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectedActionInvariantSet {
    pub timelock: TimelockInvariant,
    pub quorum: QuorumInvariant,
}

/// Signed envelope metadata for protected actions and idempotent processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedEnvelopeDescriptor {
    pub event_id: String,
    pub sequence: u64,
    pub publisher: String,
    pub payload_hash: String,
    pub commitments: Vec<String>,
}

impl SignedEnvelopeDescriptor {
    /// A deterministic key suitable for idempotency and replay-resistant processing.
    pub fn idempotency_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.publisher.trim(),
            self.event_id.trim(),
            self.sequence
        )
    }
}

/// Session lifecycle status for trust/security claims exchanged across adapters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionLifecycleStatus {
    PendingAttestation,
    Active,
    Suspended,
    Revoked,
    Expired,
}

/// Shared session trust/security claims surfaced by external runtime adapters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionTrustClaims {
    pub session_id: String,
    pub attestation_reference: String,
    pub confirmation_thumbprints: Vec<String>,
    pub lifecycle_status: SessionLifecycleStatus,
}

/// Adapter-facing request model for intent authorization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentAuthorizationRequest {
    pub intent_id: String,
    pub authority: WalletAuthority,
    pub action_state: ProtectedActionLifecycleState,
    pub trigger_state: TriggerLifecycleState,
    pub invariants: ProtectedActionInvariantSet,
    pub envelope: SignedEnvelopeDescriptor,
}

/// Adapter-facing authorization decision model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentAuthorizationDecision {
    pub authorized: bool,
    pub reason: String,
}

/// Adapter-facing request model for trust/session issuance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionIssuanceRequest {
    pub session_id: String,
    pub subject: String,
    pub requested_by: WalletAuthority,
    pub attestation_reference: String,
    pub confirmation_thumbprints: Vec<String>,
}

/// Integration boundary for standalone gateway runtime implementations.
///
/// This trait is intentionally interface-only. Runtime/provider behavior stays outside core.
pub trait ControlModelAdapter {
    fn authorize_intent(
        &self,
        request: &IntentAuthorizationRequest,
    ) -> Result<IntentAuthorizationDecision, String>;

    fn issue_session(&self, request: &SessionIssuanceRequest)
        -> Result<SessionTrustClaims, String>;
}

pub fn validate_timelock_invariant(invariant: &TimelockInvariant) -> Result<(), String> {
    let expected_not_before = invariant
        .created_at_block
        .saturating_add(invariant.timelock_blocks as u64);

    if invariant.not_before_block != expected_not_before {
        return Err(format!(
            "Timelock invariant violation: not_before_block {} does not match expected {}",
            invariant.not_before_block, expected_not_before
        ));
    }

    Ok(())
}

pub fn is_timelock_satisfied(invariant: &TimelockInvariant, current_block: u64) -> bool {
    current_block >= invariant.not_before_block
}

pub fn validate_quorum_invariant(invariant: &QuorumInvariant) -> Result<(), String> {
    if invariant.eligible_approvers == 0 {
        return Err(
            "Quorum invariant violation: eligible_approvers must be greater than zero".to_string(),
        );
    }

    if invariant.approvals_required == 0 {
        return Err(
            "Quorum invariant violation: approvals_required must be greater than zero".to_string(),
        );
    }

    if invariant.approvals_required > invariant.eligible_approvers {
        return Err(format!(
            "Quorum invariant violation: approvals_required {} exceeds eligible_approvers {}",
            invariant.approvals_required, invariant.eligible_approvers
        ));
    }

    Ok(())
}

pub fn has_reached_quorum(invariant: &QuorumInvariant, approvals_observed: u16) -> bool {
    approvals_observed >= invariant.approvals_required
}

pub fn validate_protected_action_invariants(
    invariants: &ProtectedActionInvariantSet,
) -> Result<(), String> {
    validate_timelock_invariant(&invariants.timelock)?;
    validate_quorum_invariant(&invariants.quorum)?;
    Ok(())
}

pub fn is_valid_protected_action_transition(
    from: &ProtectedActionLifecycleState,
    to: &ProtectedActionLifecycleState,
) -> bool {
    matches!(
        (from, to),
        (
            ProtectedActionLifecycleState::Draft,
            ProtectedActionLifecycleState::PendingAuthorization
        ) | (
            ProtectedActionLifecycleState::Draft,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Timelocked
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Rejected
        ) | (
            ProtectedActionLifecycleState::PendingAuthorization,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::ReadyForExecution
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::Expired
        ) | (
            ProtectedActionLifecycleState::Timelocked,
            ProtectedActionLifecycleState::Cancelled
        ) | (
            ProtectedActionLifecycleState::ReadyForExecution,
            ProtectedActionLifecycleState::Executed
        ) | (
            ProtectedActionLifecycleState::ReadyForExecution,
            ProtectedActionLifecycleState::Failed
        )
    )
}

pub fn validate_protected_action_transition(
    from: &ProtectedActionLifecycleState,
    to: &ProtectedActionLifecycleState,
) -> Result<(), String> {
    if is_valid_protected_action_transition(from, to) {
        return Ok(());
    }

    Err(format!(
        "Invalid protected action transition: {:?} -> {:?}",
        from, to
    ))
}

pub fn validate_signed_envelope_descriptor(
    descriptor: &SignedEnvelopeDescriptor,
) -> Result<(), String> {
    if descriptor.event_id.trim().is_empty() {
        return Err("Signed envelope descriptor validation failed: event_id is empty".to_string());
    }

    if descriptor.publisher.trim().is_empty() {
        return Err("Signed envelope descriptor validation failed: publisher is empty".to_string());
    }

    if descriptor.payload_hash.trim().is_empty() {
        return Err(
            "Signed envelope descriptor validation failed: payload_hash is empty".to_string(),
        );
    }

    if descriptor
        .commitments
        .iter()
        .any(|commitment| commitment.trim().is_empty())
    {
        return Err(
            "Signed envelope descriptor validation failed: commitments contain empty value"
                .to_string(),
        );
    }

    Ok(())
}

pub fn validate_monotonic_envelope_sequence(
    current_sequence: u64,
    last_seen_sequence: Option<u64>,
) -> Result<(), String> {
    if let Some(previous) = last_seen_sequence {
        if current_sequence <= previous {
            return Err(format!(
                "Replay risk detected: sequence {} is not greater than previously seen {}",
                current_sequence, previous
            ));
        }
    }

    Ok(())
}

pub fn is_duplicate_idempotency_key(idempotency_key: &str, seen_keys: &[String]) -> bool {
    seen_keys
        .iter()
        .any(|candidate| candidate.trim() == idempotency_key.trim())
}

pub fn validate_session_trust_claims(claims: &SessionTrustClaims) -> Result<(), String> {
    if claims.session_id.trim().is_empty() {
        return Err("Session trust validation failed: session_id is empty".to_string());
    }

    if claims.attestation_reference.trim().is_empty() {
        return Err("Session trust validation failed: attestation_reference is empty".to_string());
    }

    if claims.confirmation_thumbprints.is_empty() {
        return Err(
            "Session trust validation failed: confirmation_thumbprints cannot be empty".to_string(),
        );
    }

    if claims
        .confirmation_thumbprints
        .iter()
        .any(|thumbprint| thumbprint.trim().is_empty())
    {
        return Err(
            "Session trust validation failed: confirmation_thumbprints contain empty value"
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timelock_invariant_is_valid_and_enforced() {
        let timelock = TimelockInvariant::new(840_000, 144);

        assert!(validate_timelock_invariant(&timelock).is_ok());
        assert!(!is_timelock_satisfied(&timelock, 840_143));
        assert!(is_timelock_satisfied(&timelock, 840_144));
    }

    #[test]
    fn test_timelock_invariant_rejects_incorrect_not_before_block() {
        let invalid = TimelockInvariant {
            created_at_block: 100,
            timelock_blocks: 5,
            not_before_block: 104,
        };

        assert!(validate_timelock_invariant(&invalid).is_err());
    }

    #[test]
    fn test_quorum_invariant_checks() {
        let valid = QuorumInvariant::new(2, 3);
        assert!(validate_quorum_invariant(&valid).is_ok());
        assert!(!has_reached_quorum(&valid, 1));
        assert!(has_reached_quorum(&valid, 2));

        let invalid = QuorumInvariant::new(3, 2);
        assert!(validate_quorum_invariant(&invalid).is_err());
    }

    #[test]
    fn test_protected_action_lifecycle_transition_rules() {
        assert!(validate_protected_action_transition(
            &ProtectedActionLifecycleState::Draft,
            &ProtectedActionLifecycleState::PendingAuthorization,
        )
        .is_ok());

        assert!(validate_protected_action_transition(
            &ProtectedActionLifecycleState::Executed,
            &ProtectedActionLifecycleState::ReadyForExecution,
        )
        .is_err());
    }

    #[test]
    fn test_signed_envelope_descriptor_and_replay_helpers() {
        let descriptor = SignedEnvelopeDescriptor {
            event_id: "evt-123".to_string(),
            sequence: 42,
            publisher: "gateway-a".to_string(),
            payload_hash: "sha256:abc".to_string(),
            commitments: vec!["commitment-a".to_string()],
        };

        assert!(validate_signed_envelope_descriptor(&descriptor).is_ok());
        assert_eq!(descriptor.idempotency_key(), "gateway-a:evt-123:42");
        assert!(validate_monotonic_envelope_sequence(43, Some(42)).is_ok());
        assert!(validate_monotonic_envelope_sequence(42, Some(42)).is_err());

        let seen = vec!["gateway-a:evt-123:42".to_string()];
        assert!(is_duplicate_idempotency_key(
            &descriptor.idempotency_key(),
            &seen,
        ));
    }

    #[test]
    fn test_session_trust_claims_validation() {
        let claims = SessionTrustClaims {
            session_id: "session-1".to_string(),
            attestation_reference: "attestation:strongbox:v1".to_string(),
            confirmation_thumbprints: vec!["thumbprint-1".to_string()],
            lifecycle_status: SessionLifecycleStatus::Active,
        };

        assert!(validate_session_trust_claims(&claims).is_ok());

        let empty_attestation = SessionTrustClaims {
            attestation_reference: "   ".to_string(),
            ..claims.clone()
        };

        assert!(validate_session_trust_claims(&empty_attestation).is_err());
    }
}

/// Multi-factor risk assessment for a protocol rail.
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

/// Comprehensive status for a Conxian-supported protocol rail.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub latency_ms: u32,
    pub trust_model: String,
    pub risk_level: String,
    pub risk_assessment: Option<RiskAssessment>,
    pub data_availability: String,
    pub settlement: String,
    pub bridge_security: String,
    pub tvl_usd: f64,
    pub version: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
    pub rail_metadata: RailMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdentityRecord {
    pub address: String,
    pub ens_name: Option<String>,
    pub bns_name: Option<String>,
    pub world_id_verified: bool,
}

/// Bitcoin transaction orchestration state record.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BtcTxOrchestrationRecord {
    pub tx_id: String,
    pub state: String,
    pub latest_transition: Option<String>,
    pub latest_event_id: Option<String>,
    pub fee_rate_sat_vb: Option<u64>,
    pub attempt: u32,
    pub observed_confirmations: Option<u32>,
    pub recovery_cursor: u64,
    pub updated_at_epoch_ms: u64,
}

/// Bitcoin transaction lifecycle event record.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BtcTxEventRecord {
    pub event_id: String,
    pub tx_id: String,
    pub idempotency_key: String,
    pub transition: String,
    pub from_state: String,
    pub to_state: String,
    pub attempt: u32,
    pub fee_rate_sat_vb: Option<u64>,
    pub observed_confirmations: Option<u32>,
    pub rationale: Option<String>,
    pub fingerprint: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleState {
    Draft,
    Signed,
    BroadcastPending,
    InMempool,
    PendingConfirmations,
    Confirmed,
    Finalized,
    Reorged,
    DeadLetter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleEvent {
    Sign,
    QueueBroadcast,
    MempoolObserved,
    ConfirmationsObserved,
    Finalize,
    ReorgDetected,
    MarkDeadLetter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleExecutionMode {
    Disabled,
    Shadow,
    Active,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxLifecycleRolloutMode {
    Shadow,
    Limited,
    Full,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BitcoinFeeBumpAction {
    None,
    Rbf,
    Cpfp,
    Escalate,
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
}

pub enum BitcoinFeeBumpReason {
    PolicyAged,
    PolicyStuck,
    ManualIntervention,
    NetworkCongestion,
}

/// Tier 1, 2, and 3 chain families for universal support (ADR-006).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    /// Bitcoin/UTXO: Native, Stacks, Liquid, Babylon, BOB, Mezo.
    BitcoinUtxo,
    /// EVM: Ethereum, Base, Arbitrum, Optimism, Polygon, Botanix.
    Evm,
    /// Cosmos/IBC: Cosmos Hub, Osmosis, Celestia.
    CosmosIbc,
    /// Solana/SVM: Solana, Eclipse.
    SolanaSvm,
    /// Move: Sui, Aptos.
    Move,
    /// Substrate: Polkadot, Kusama.
    Substrate,
}

/// Approved bridge and messaging trust tiers (CON-791).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    /// T1: Sovereign Verified (e.g., IBC light-client paths).
    Strict,
    /// T2: Hybrid Verified (proof + independent secondary verifiers).
    Managed,
    /// T3: Attester Network (external quorum with caps/kill-switches).
    Expedient,
    /// T4: Observer/Weak (not allowed in production).
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

/// Classification of the verification mechanism used by a bridge or messaging system.
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

/// Finality guarantees for cross-chain messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalityClass {
    Economic,
    Probabilistic,
    Deterministic,
}

/// Status of a verification decision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Degraded,
    Blocked,
}

/// Approved bridge and messaging systems (CON-791).
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
}

/// Canonical proof envelope for cross-chain operations (CON-791/CON-799).
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

#[cfg(test)]
mod trust_policy_tests {
    use super::*;

    #[test]
    fn test_trust_tier_production_allowance() {
        assert!(TrustTier::Strict.is_production_allowed());
        assert!(TrustTier::Managed.is_production_allowed());
        assert!(TrustTier::Expedient.is_production_allowed());
        assert!(!TrustTier::ObserverOnly.is_production_allowed());
    }

    #[test]
    fn test_validate_trust_tier_policy() {
        assert!(
            validate_trust_tier_policy(TrustTier::Strict, VerificationClass::LightClient).is_ok()
        );
        assert!(
            validate_trust_tier_policy(TrustTier::Strict, VerificationClass::ExternalQuorum)
                .is_err()
        );
        assert!(
            validate_trust_tier_policy(TrustTier::Managed, VerificationClass::ExternalQuorum)
                .is_ok()
        );
        assert!(validate_trust_tier_policy(
            TrustTier::ObserverOnly,
            VerificationClass::NativeObservation
        )
        .is_err());
    }
}

/// Basic status of an ecosystem service.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BasicServiceStatus {
    pub service_name: String,
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceResponse {
    pub service: String,
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Common trait for all Conxian ecosystem services.
pub trait ConxianService {
    fn name(&self) -> &str;
    fn status(&self) -> BasicServiceStatus;
    fn handle_request(&self, payload: &str) -> ServiceResponse;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReserveAsset {
    pub asset: String,
    pub total_supplied: f64,
    pub total_reserves: f64,
    pub collateral_ratio: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PriceInfo {
    // f64 does not implement Eq
    pub asset: String,
    pub price_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ComplianceStatus {
    pub status: String,
    pub last_audit: chrono::DateTime<chrono::Utc>,
    pub rules_active: Vec<String>,
    pub risk_score: u32,
    pub zkml_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinancialMetrics {
    pub mrr_usd: f64,
    pub arr_usd: f64,
    pub churn_rate_pct: f64,
    pub protocol_fees_collected_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod universal_chain_tests {
    use super::*;

    #[test]
    fn test_chain_family_variants() {
        let families = [
            ChainFamily::BitcoinUtxo,
            ChainFamily::Evm,
            ChainFamily::CosmosIbc,
            ChainFamily::SolanaSvm,
            ChainFamily::Move,
            ChainFamily::Substrate,
        ];
        assert_eq!(families.len(), 6);
    }

    #[test]
    fn test_bridge_system_expansion() {
        let systems = [
            BridgeSystem::ChainlinkCcip,
            BridgeSystem::NearChainSignatures,
            BridgeSystem::CircleCctp,
            BridgeSystem::NexusZkVM,
        ];
        assert_eq!(systems.len(), 4);
    }

    #[test]
    fn test_chain_enum_variants() {
        let chains = [
            Chain::Babylon,
            Chain::Bob,
            Chain::Mezo,
            Chain::Citrea,
            Chain::Botanix,
            Chain::Ethereum,
            Chain::Base,
        ];
        assert!(chains.len() >= 7);
    }
}
