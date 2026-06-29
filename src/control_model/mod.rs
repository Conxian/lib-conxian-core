pub mod trust;
pub mod lifecycle;
pub mod ops;

pub use trust::*;
pub use lifecycle::*;
pub use ops::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WalletAuthority {
    pub authority_id: String,
    pub wallet_id: String,
    pub class: WalletAuthorityClass,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectedActionInvariantSet {
    pub timelock: TimelockInvariant,
    pub quorum: QuorumInvariant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedEnvelopeDescriptor {
    pub event_id: String,
    pub sequence: u64,
    pub publisher: String,
    pub payload_hash: String,
    pub commitments: Vec<String>,
}

impl SignedEnvelopeDescriptor {
    pub fn idempotency_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.publisher.trim(),
            self.event_id.trim(),
            self.sequence
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionTrustClaims {
    pub session_id: String,
    pub attestation_reference: String,
    pub confirmation_thumbprints: Vec<String>,
    pub lifecycle_status: SessionLifecycleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentAuthorizationRequest {
    pub intent_id: String,
    pub authority: WalletAuthority,
    pub action_state: ProtectedActionLifecycleState,
    pub trigger_state: TriggerLifecycleState,
    pub invariants: ProtectedActionInvariantSet,
    pub envelope: SignedEnvelopeDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntentAuthorizationDecision {
    pub authorized: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionIssuanceRequest {
    pub session_id: String,
    pub subject: String,
    pub requested_by: WalletAuthority,
    pub attestation_reference: String,
    pub confirmation_thumbprints: Vec<String>,
}

pub trait ControlModelAdapter {
    fn authorize_intent(
        &self,
        request: &IntentAuthorizationRequest,
    ) -> Result<IntentAuthorizationDecision, String>;

    fn issue_session(&self, request: &SessionIssuanceRequest)
        -> Result<SessionTrustClaims, String>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdentityRecord {
    pub address: String,
    pub ens_name: Option<String>,
    pub bns_name: Option<String>,
    pub world_id_verified: bool,
}

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
        return Err("Quorum invariant violation: eligible_approvers must be greater than zero".to_string());
    }
    if invariant.approvals_required == 0 {
        return Err("Quorum invariant violation: approvals_required must be greater than zero".to_string());
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
        return Err("Signed envelope descriptor validation failed: payload_hash is empty".to_string());
    }
    if descriptor
        .commitments
        .iter()
        .any(|commitment| commitment.trim().is_empty())
    {
        return Err("Signed envelope descriptor validation failed: commitments contain empty value".to_string());
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
        return Err("Session trust validation failed: confirmation_thumbprints cannot be empty".to_string());
    }
    if claims
        .confirmation_thumbprints
        .iter()
        .any(|thumbprint| thumbprint.trim().is_empty())
    {
        return Err("Session trust validation failed: confirmation_thumbprints contain empty value".to_string());
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
        ).is_ok());

        assert!(validate_protected_action_transition(
            &ProtectedActionLifecycleState::Executed,
            &ProtectedActionLifecycleState::ReadyForExecution,
        ).is_err());
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
        assert!(is_duplicate_idempotency_key(&descriptor.idempotency_key(), &seen));
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
    }

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
            BridgeSystem::Bitvm2,
        ];
        assert_eq!(systems.len(), 5);
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

    #[test]
    fn test_release_status_transitions() {
        assert!(validate_release_transition(&ReleaseStatus::Proposed, &ReleaseStatus::InReview).is_ok());
        assert!(validate_release_transition(&ReleaseStatus::Released, &ReleaseStatus::Deprecated).is_ok());
        assert!(validate_release_transition(&ReleaseStatus::Released, &ReleaseStatus::Approved).is_err());
    }

    #[test]
    fn test_policy_approval_transitions() {
        assert!(validate_policy_approval_transition(&PolicyApprovalStatus::Pending, &PolicyApprovalStatus::Approved).is_ok());
        assert!(validate_policy_approval_transition(&PolicyApprovalStatus::Approved, &PolicyApprovalStatus::Implemented).is_ok());
        assert!(validate_policy_approval_transition(&PolicyApprovalStatus::Implemented, &PolicyApprovalStatus::Pending).is_err());
    }

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::High);
        assert!(AuditSeverity::High > AuditSeverity::Medium);
        assert!(AuditSeverity::Medium > AuditSeverity::Low);
    }

    #[test]
    fn test_trust_tier_production_allowance() {
        assert!(TrustTier::Strict.is_production_allowed());
        assert!(TrustTier::Managed.is_production_allowed());
        assert!(TrustTier::Expedient.is_production_allowed());
        assert!(!TrustTier::ObserverOnly.is_production_allowed());
    }

    #[test]
    fn test_validate_trust_tier_policy() {
        assert!(validate_trust_tier_policy(TrustTier::Strict, VerificationClass::LightClient).is_ok());
        assert!(validate_trust_tier_policy(TrustTier::Strict, VerificationClass::ExternalQuorum).is_err());
        assert!(validate_trust_tier_policy(TrustTier::Managed, VerificationClass::ExternalQuorum).is_ok());
        assert!(validate_trust_tier_policy(TrustTier::ObserverOnly, VerificationClass::NativeObservation).is_err());
    }
}
