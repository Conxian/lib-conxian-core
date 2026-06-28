use serde::{Deserialize, Serialize};

use crate::control_model::ChainFamily;

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
    /// Returns true if this trust tier is allowed in production.
    ///
    /// ```
    /// use lib_conxian_core::control_model::TrustTier;
    ///
    /// assert!(TrustTier::Strict.is_production_allowed());
    /// assert!(TrustTier::Managed.is_production_allowed());
    /// assert!(TrustTier::Expedient.is_production_allowed());
    /// assert!(!TrustTier::ObserverOnly.is_production_allowed());
    /// ```
    pub fn is_production_allowed(&self) -> bool {
        matches!(
            self,
            TrustTier::Strict | TrustTier::Managed | TrustTier::Expedient
        )
    }

    /// Returns true if this tier mandates light-client verification.
    ///
    /// ```
    /// use lib_conxian_core::control_model::TrustTier;
    ///
    /// assert!(TrustTier::Strict.requires_light_client());
    /// assert!(!TrustTier::Managed.requires_light_client());
    /// ```
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
    Bitvm2,
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
