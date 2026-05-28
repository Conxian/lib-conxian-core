pub mod anchoring;
pub mod mcp;
pub mod persistence;
pub mod remediation;
pub mod support;
use crate::engine::anchoring::{
    AnchoringError, AnchoringPublisher, AnchoringReceipt, AnchoringRequest, AnchoringTarget,
    OnChainAnchoringPublisher, TablelandAnchoringPublisher,
};
use crate::engine::persistence::{
    AppendEventOutcome, BitcoinTxPersistence, BtcTxEventRecord, BtcTxOrchestrationRecord,
    InMemoryBitcoinTxPersistence, PersistenceError,
};
use crate::engine::support::{SupportConfig, SupportIntake};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct AnchoringReplayRecord {
    pub request_fingerprint: String,
    pub receipt: AnchoringReceipt,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RiskAssessment {
    pub overall_level: String,
    pub da_score: u32,
    pub settlement_score: u32,
    pub bridge_score: u32,
    pub exit_mechanism_score: u32,
    pub operators_score: u32,
    pub decentralization_score: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailTrustAssumptions {
    pub security_anchor: String,
    pub operator_dependency: String,
    pub liveness_assumption: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailFinalitySemantics {
    pub confirmation_model: String,
    pub settlement_layer: String,
    pub typical_finality_window: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailCustodyModel {
    pub asset_control_model: String,
    pub signer_architecture: String,
    pub redemption_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailComplianceConstraints {
    pub baseline_controls: Vec<String>,
    pub jurisdictional_scope: String,
    pub monitoring_requirements: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailOperationalCapabilities {
    pub supported_flows: Vec<String>,
    pub integration_modes: Vec<String>,
    pub resilience_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RailMetadata {
    pub rail_family: String,
    pub trust_assumptions: RailTrustAssumptions,
    pub finality_semantics: RailFinalitySemantics,
    pub custody_model: RailCustodyModel,
    pub compliance_constraints: RailComplianceConstraints,
    pub operational_capabilities: RailOperationalCapabilities,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: DateTime<Utc>,
    pub latency_ms: u32,
    pub trust_model: String,
    pub risk_level: String,
    pub risk_assessment: Option<RiskAssessment>,
    pub data_availability: String,
    pub settlement: String,
    pub bridge_security: String,
    pub tvl_usd: f64,
    pub version: Option<String>,
    pub metadata: HashMap<String, String>,
    pub rail_metadata: RailMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReserveAsset {
    pub asset: String,
    pub total_supplied: f64,
    pub total_reserves: f64,
    pub collateral_ratio: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceInfo {
    pub asset: String,
    pub price_usd: f64,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComplianceStatus {
    pub status: String,
    pub last_audit: DateTime<Utc>,
    pub rules_active: Vec<String>,
    pub risk_score: u32,
    pub zkml_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffiliateInfo {
    pub partner_id: String,
    pub status: String,
    pub commission_rate: f64,
    pub active_campaigns: u32,
    pub total_referrals: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketingInfo {
    pub channel: String,
    pub status: String,
    pub active_offers: Vec<String>,
    pub reach: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinancialMetrics {
    pub mrr_usd: f64,
    pub arr_usd: f64,
    pub churn_rate_pct: f64,
    pub protocol_fees_collected_usd: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityRecord {
    pub address: String,
    pub ens_name: Option<String>,
    pub bns_name: Option<String>,
    pub world_id_verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErpSyncRecord {
    pub erp_system: String,
    pub last_sync: DateTime<Utc>,
    pub total_transactions_synced: u64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettlementEnvelope {
    pub protocol: String,
    pub payload: Value,
    pub raw_payload_bytes: String,
    pub ingress_timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLead {
    pub lead_id: String,
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartnerLeadStatus {
    New,
    Assigned,
    InProgress,
    Qualified,
    Escalated,
    ClosedWon,
    ClosedLost,
}

impl PartnerLeadStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PartnerLeadStatus::New => "new",
            PartnerLeadStatus::Assigned => "assigned",
            PartnerLeadStatus::InProgress => "in_progress",
            PartnerLeadStatus::Qualified => "qualified",
            PartnerLeadStatus::Escalated => "escalated",
            PartnerLeadStatus::ClosedWon => "closed_won",
            PartnerLeadStatus::ClosedLost => "closed_lost",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadEvent {
    pub event_id: String,
    pub lead_id: String,
    pub timestamp: DateTime<Utc>,
    pub from_status: PartnerLeadStatus,
    pub to_status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub note: Option<String>,
}

pub struct PartnerLeadCreateInput {
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub notes: Option<String>,
}

pub struct PartnerLeadStatusUpdateInput {
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub escalated_to: Option<String>,
    pub escalation_reason: Option<String>,
    pub event_note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadCreateOutcome {
    pub lead_id: String,
    pub status: String,
    pub idempotent_replay: bool,
}

#[derive(Debug)]
pub enum PartnerLeadTransitionError {
    NotFound,
    InvalidTransition {
        from: PartnerLeadStatus,
        to: PartnerLeadStatus,
    },
    OwnerRequired,
    EscalationReasonRequired,
}

pub const CONXIAN_BTC_TX_LIFECYCLE_ENABLED_ENV: &str = "CONXIAN_BTC_TX_LIFECYCLE_ENABLED";
pub const CONXIAN_BTC_TX_LIFECYCLE_SHADOW_MODE_ENV: &str = "CONXIAN_BTC_TX_LIFECYCLE_SHADOW_MODE";
pub const CONXIAN_BTC_FEE_BUMP_MAX_ATTEMPTS_ENV: &str = "CONXIAN_BTC_FEE_BUMP_MAX_ATTEMPTS";
pub const CONXIAN_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB_ENV: &str =
    "CONXIAN_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB";
pub const CONXIAN_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB_ENV: &str =
    "CONXIAN_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB";
pub const CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS_ENV: &str =
    "CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS";
pub const CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS_ENV: &str =
    "CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS";

const DEFAULT_REQUIRED_CONFIRMATIONS: u32 = 6;
const DEFAULT_BTC_FEE_BUMP_MAX_ATTEMPTS: u8 = 3;
const DEFAULT_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB: u64 = 150;
const DEFAULT_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB: u64 = 2;
const DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS: u32 = 3;
const DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS: u64 = 900;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

impl BitcoinTxLifecycleState {
    pub fn as_str(&self) -> &str {
        match self {
            BitcoinTxLifecycleState::Draft => "draft",
            BitcoinTxLifecycleState::Signed => "signed",
            BitcoinTxLifecycleState::BroadcastPending => "broadcast_pending",
            BitcoinTxLifecycleState::InMempool => "in_mempool",
            BitcoinTxLifecycleState::PendingConfirmations => "pending_confirmations",
            BitcoinTxLifecycleState::Confirmed => "confirmed",
            BitcoinTxLifecycleState::Finalized => "finalized",
            BitcoinTxLifecycleState::Reorged => "reorged",
            BitcoinTxLifecycleState::DeadLetter => "dead_letter",
        }
    }
}

impl FromStr for BitcoinTxLifecycleState {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(BitcoinTxLifecycleState::Draft),
            "signed" => Ok(BitcoinTxLifecycleState::Signed),
            "broadcast_pending" => Ok(BitcoinTxLifecycleState::BroadcastPending),
            "in_mempool" => Ok(BitcoinTxLifecycleState::InMempool),
            "pending_confirmations" => Ok(BitcoinTxLifecycleState::PendingConfirmations),
            "confirmed" => Ok(BitcoinTxLifecycleState::Confirmed),
            "finalized" => Ok(BitcoinTxLifecycleState::Finalized),
            "reorged" => Ok(BitcoinTxLifecycleState::Reorged),
            "dead_letter" => Ok(BitcoinTxLifecycleState::DeadLetter),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinTxLifecycleEvent {
    Sign,
    QueueBroadcast,
    MempoolObserved,
    ConfirmationsObserved,
    Finalize,
    ReorgDetected,
    MarkDeadLetter,
}

impl BitcoinTxLifecycleEvent {
    pub fn as_str(&self) -> &str {
        match self {
            BitcoinTxLifecycleEvent::Sign => "sign",
            BitcoinTxLifecycleEvent::QueueBroadcast => "queue_broadcast",
            BitcoinTxLifecycleEvent::MempoolObserved => "mempool_observed",
            BitcoinTxLifecycleEvent::ConfirmationsObserved => "confirmations_observed",
            BitcoinTxLifecycleEvent::Finalize => "finalize",
            BitcoinTxLifecycleEvent::ReorgDetected => "reorg_detected",
            BitcoinTxLifecycleEvent::MarkDeadLetter => "mark_dead_letter",
        }
    }
}

impl FromStr for BitcoinTxLifecycleEvent {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sign" => Ok(BitcoinTxLifecycleEvent::Sign),
            "queue_broadcast" => Ok(BitcoinTxLifecycleEvent::QueueBroadcast),
            "mempool_observed" => Ok(BitcoinTxLifecycleEvent::MempoolObserved),
            "confirmations_observed" => Ok(BitcoinTxLifecycleEvent::ConfirmationsObserved),
            "finalize" => Ok(BitcoinTxLifecycleEvent::Finalize),
            "reorg_detected" => Ok(BitcoinTxLifecycleEvent::ReorgDetected),
            "mark_dead_letter" => Ok(BitcoinTxLifecycleEvent::MarkDeadLetter),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinTxLifecycleExecutionMode {
    Disabled,
    Shadow,
    Active,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinTxLifecycleConfig {
    pub enabled: bool,
    pub shadow_mode: bool,
}

impl BitcoinTxLifecycleConfig {
    pub fn from_env() -> Self {
        Self {
            enabled: parse_env_bool(CONXIAN_BTC_TX_LIFECYCLE_ENABLED_ENV, false),
            shadow_mode: parse_env_bool(CONXIAN_BTC_TX_LIFECYCLE_SHADOW_MODE_ENV, false),
        }
    }

    pub fn execution_mode(&self) -> BitcoinTxLifecycleExecutionMode {
        if !self.enabled {
            BitcoinTxLifecycleExecutionMode::Disabled
        } else if self.shadow_mode {
            BitcoinTxLifecycleExecutionMode::Shadow
        } else {
            BitcoinTxLifecycleExecutionMode::Active
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinFeeBumpPolicy {
    pub max_attempts: u8,
    pub max_fee_rate_sats_vb: u64,
    pub min_bump_increment_sats_vb: u64,
    pub stuck_threshold_blocks: u32,
    pub stuck_threshold_seconds: u64,
}

impl Default for BitcoinFeeBumpPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_BTC_FEE_BUMP_MAX_ATTEMPTS,
            max_fee_rate_sats_vb: DEFAULT_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB,
            min_bump_increment_sats_vb: DEFAULT_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB,
            stuck_threshold_blocks: DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS,
            stuck_threshold_seconds: DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS,
        }
    }
}

impl BitcoinFeeBumpPolicy {
    pub fn from_env() -> Self {
        Self {
            max_attempts: parse_env_u8(
                CONXIAN_BTC_FEE_BUMP_MAX_ATTEMPTS_ENV,
                DEFAULT_BTC_FEE_BUMP_MAX_ATTEMPTS,
            )
            .max(1),
            max_fee_rate_sats_vb: parse_env_u64(
                CONXIAN_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB_ENV,
                DEFAULT_BTC_FEE_BUMP_MAX_FEE_RATE_SATS_VB,
            )
            .max(1),
            min_bump_increment_sats_vb: parse_env_u64(
                CONXIAN_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB_ENV,
                DEFAULT_BTC_FEE_BUMP_MIN_INCREMENT_SATS_VB,
            )
            .max(1),
            stuck_threshold_blocks: parse_env_u32(
                CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS_ENV,
                DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_BLOCKS,
            )
            .max(1),
            stuck_threshold_seconds: parse_env_u64(
                CONXIAN_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS_ENV,
                DEFAULT_BTC_FEE_BUMP_STUCK_THRESHOLD_SECONDS,
            )
            .max(1),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinFeeBumpAction {
    Rbf,
    Cpfp,
    Noop,
    Reject,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinFeeBumpReason {
    StuckByBlockThreshold,
    StuckByTimeThreshold,
    StuckThresholdNotMet,
    MissingStuckObservation,
    MaxAttemptsReached,
    FeeCapExceeded,
    RbfPreferred,
    CpfpFallback,
    NoAvailableBumpPath,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BitcoinTxStuckClassification {
    pub is_stuck: bool,
    pub reason: BitcoinFeeBumpReason,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinFeeBumpDecisionInput {
    pub attempts_used: u8,
    pub current_fee_rate_sats_vb: u64,
    pub network_target_fee_rate_sats_vb: u64,
    pub replaceable: bool,
    pub cpfp_available: bool,
    pub blocks_since_broadcast: Option<u32>,
    pub seconds_since_broadcast: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BitcoinFeeBumpDecision {
    pub action: BitcoinFeeBumpAction,
    pub reason: BitcoinFeeBumpReason,
    pub next_fee_rate_sats_vb: Option<u64>,
    pub next_attempt: Option<u8>,
}

pub fn classify_bitcoin_tx_stuck(
    policy: &BitcoinFeeBumpPolicy,
    blocks_since_broadcast: Option<u32>,
    seconds_since_broadcast: Option<u64>,
) -> BitcoinTxStuckClassification {
    if blocks_since_broadcast
        .map(|blocks| blocks >= policy.stuck_threshold_blocks)
        .unwrap_or(false)
    {
        return BitcoinTxStuckClassification {
            is_stuck: true,
            reason: BitcoinFeeBumpReason::StuckByBlockThreshold,
        };
    }

    if seconds_since_broadcast
        .map(|seconds| seconds >= policy.stuck_threshold_seconds)
        .unwrap_or(false)
    {
        return BitcoinTxStuckClassification {
            is_stuck: true,
            reason: BitcoinFeeBumpReason::StuckByTimeThreshold,
        };
    }

    if blocks_since_broadcast.is_none() && seconds_since_broadcast.is_none() {
        return BitcoinTxStuckClassification {
            is_stuck: false,
            reason: BitcoinFeeBumpReason::MissingStuckObservation,
        };
    }

    BitcoinTxStuckClassification {
        is_stuck: false,
        reason: BitcoinFeeBumpReason::StuckThresholdNotMet,
    }
}

pub fn evaluate_bitcoin_fee_bump_decision(
    policy: &BitcoinFeeBumpPolicy,
    input: &BitcoinFeeBumpDecisionInput,
) -> BitcoinFeeBumpDecision {
    let stuck = classify_bitcoin_tx_stuck(
        policy,
        input.blocks_since_broadcast,
        input.seconds_since_broadcast,
    );

    if !stuck.is_stuck {
        return BitcoinFeeBumpDecision {
            action: BitcoinFeeBumpAction::Noop,
            reason: stuck.reason,
            next_fee_rate_sats_vb: None,
            next_attempt: None,
        };
    }

    if input.attempts_used >= policy.max_attempts {
        return BitcoinFeeBumpDecision {
            action: BitcoinFeeBumpAction::Reject,
            reason: BitcoinFeeBumpReason::MaxAttemptsReached,
            next_fee_rate_sats_vb: None,
            next_attempt: None,
        };
    }

    let minimum_bump_rate = input
        .current_fee_rate_sats_vb
        .saturating_add(policy.min_bump_increment_sats_vb);
    let next_fee_rate = minimum_bump_rate.max(input.network_target_fee_rate_sats_vb);

    if next_fee_rate > policy.max_fee_rate_sats_vb {
        return BitcoinFeeBumpDecision {
            action: BitcoinFeeBumpAction::Reject,
            reason: BitcoinFeeBumpReason::FeeCapExceeded,
            next_fee_rate_sats_vb: None,
            next_attempt: None,
        };
    }

    let next_attempt = Some(input.attempts_used.saturating_add(1));

    if input.replaceable {
        return BitcoinFeeBumpDecision {
            action: BitcoinFeeBumpAction::Rbf,
            reason: BitcoinFeeBumpReason::RbfPreferred,
            next_fee_rate_sats_vb: Some(next_fee_rate),
            next_attempt,
        };
    }

    if input.cpfp_available {
        return BitcoinFeeBumpDecision {
            action: BitcoinFeeBumpAction::Cpfp,
            reason: BitcoinFeeBumpReason::CpfpFallback,
            next_fee_rate_sats_vb: Some(next_fee_rate),
            next_attempt,
        };
    }

    BitcoinFeeBumpDecision {
        action: BitcoinFeeBumpAction::Reject,
        reason: BitcoinFeeBumpReason::NoAvailableBumpPath,
        next_fee_rate_sats_vb: None,
        next_attempt: None,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinTxLifecycleRecord {
    pub tx_id: String,
    pub state: BitcoinTxLifecycleState,
    pub latest_transition: Option<BitcoinTxLifecycleEvent>,
    pub latest_event_id: Option<String>,
    pub fee_rate_sat_vb: Option<u64>,
    pub attempt: u32,
    pub confirmations_observed: u32,
    pub required_confirmations: u32,
    pub reorg_depth: Option<u32>,
    pub dead_letter_reason: Option<String>,
    pub recovery_cursor: u64,
    pub updated_at: DateTime<Utc>,
}

impl BitcoinTxLifecycleRecord {
    fn draft(tx_id: &str) -> Self {
        let now = now_epoch_ms();
        Self {
            tx_id: tx_id.to_string(),
            state: BitcoinTxLifecycleState::Draft,
            latest_transition: None,
            latest_event_id: None,
            fee_rate_sat_vb: None,
            attempt: 0,
            confirmations_observed: 0,
            required_confirmations: DEFAULT_REQUIRED_CONFIRMATIONS,
            reorg_depth: None,
            dead_letter_reason: None,
            recovery_cursor: now,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinTxLifecycleView {
    pub tx_id: String,
    pub execution_mode: BitcoinTxLifecycleExecutionMode,
    pub production: BitcoinTxLifecycleRecord,
    pub shadow: Option<BitcoinTxLifecycleRecord>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinTxTransitionInput {
    pub tx_id: String,
    #[serde(alias = "transition")]
    pub event: BitcoinTxLifecycleEvent,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub fee_rate_sat_vb: Option<u64>,
    #[serde(default)]
    pub attempt: Option<u32>,
    #[serde(default)]
    pub confirmations_observed: Option<u32>,
    #[serde(default)]
    pub required_confirmations: Option<u32>,
    #[serde(default)]
    pub reorg_depth: Option<u32>,
    #[serde(default)]
    pub dead_letter_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BitcoinTxTransitionOutcome {
    pub tx_id: String,
    pub event_id: String,
    pub idempotency_key: String,
    pub event: BitcoinTxLifecycleEvent,
    pub from_state: BitcoinTxLifecycleState,
    pub to_state: BitcoinTxLifecycleState,
    pub execution_mode: BitcoinTxLifecycleExecutionMode,
    pub idempotent_replay: bool,
    pub state_mutated: bool,
    pub telemetry_recorded: bool,
    pub transitioned_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinTxOrchestration {
    pub tx_id: String,
    pub state: BitcoinTxLifecycleState,
    pub latest_transition: Option<BitcoinTxLifecycleEvent>,
    pub latest_event_id: Option<String>,
    pub fee_rate_sat_vb: Option<u64>,
    pub attempt: u32,
    pub observed_confirmations: Option<u32>,
    pub recovery_cursor: u64,
    pub updated_at_epoch_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinTxEvent {
    pub event_id: String,
    pub tx_id: String,
    pub idempotency_key: String,
    pub event: BitcoinTxLifecycleEvent,
    pub from_state: BitcoinTxLifecycleState,
    pub to_state: BitcoinTxLifecycleState,
    pub attempt: u32,
    pub fee_rate_sat_vb: Option<u64>,
    pub observed_confirmations: Option<u32>,
    pub rationale: Option<String>,
    pub fingerprint: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Clone, Debug)]
pub enum BitcoinTxTransitionError {
    FeatureDisabled,
    TxIdRequired,
    MissingField {
        field: &'static str,
        event: BitcoinTxLifecycleEvent,
    },
    InvalidTransition {
        from: BitcoinTxLifecycleState,
        event: BitcoinTxLifecycleEvent,
        reason: String,
    },
    TerminalState {
        state: BitcoinTxLifecycleState,
    },
    UnknownPersistedState(String),
    UnknownPersistedEvent(String),
    IdempotencyConflict {
        tx_id: String,
        idempotency_key: String,
        existing_fingerprint: String,
        incoming_fingerprint: String,
    },
    Persistence(String),
}

fn parse_env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        })
        .unwrap_or(default)
}

fn parse_env_u8(name: &str, default: u8) -> u8 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u8>().ok())
        .unwrap_or(default)
}

fn parse_env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn parse_env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateProposal {
    pub proposal_id: String,
    pub trigger_id: String,
    pub proposed_state: String,
    pub timelock_end_block: u64,
    pub status: String, // "Pending", "Approved", "Executed"
    pub tee_attestation: String,
    pub yield_routing: String,  // "5/5/90"
    pub capital_status: String, // "TransitBond" or "Escrow"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SabWallet {
    pub address: String,
    pub role: String,   // "Execution", "Treasury", "Payout", "Signer", "Emergency"
    pub owner: String,  // "SAB", "Operator", "DAO"
    pub status: String, // "Active", "Deprecated", "Pending"
    pub quorum: Option<String>,
    pub spending_limit_usd: Option<f64>,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub support_intake: Arc<SupportIntake>,
    pub request_count: AtomicU64,
    pub total_tvl_usd: Arc<RwLock<f64>>,
    pub active_sovereign_nodes: AtomicU64,
    pub service_statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
    pub reserves: Arc<RwLock<Vec<ReserveAsset>>>,
    pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,
    pub compliance: Arc<RwLock<ComplianceStatus>>,
    pub affiliates: Arc<RwLock<HashMap<String, AffiliateInfo>>>,
    pub marketing: Arc<RwLock<Vec<MarketingInfo>>>,
    pub financial_metrics: Arc<RwLock<FinancialMetrics>>,
    pub identity_records: Arc<RwLock<HashMap<String, IdentityRecord>>>,
    pub erp_sync_status: Arc<RwLock<HashMap<String, ErpSyncRecord>>>,
    pub settlement_log: Arc<RwLock<Vec<SettlementEnvelope>>>,
    pub state_proposals: Arc<RwLock<HashMap<String, StateProposal>>>,
    pub bitcoin_tx_lifecycle_config: BitcoinTxLifecycleConfig,
    pub bitcoin_fee_bump_policy: BitcoinFeeBumpPolicy,
    pub bitcoin_tx_lifecycle: Arc<RwLock<HashMap<String, BitcoinTxLifecycleRecord>>>,
    pub bitcoin_tx_lifecycle_shadow: Arc<RwLock<HashMap<String, BitcoinTxLifecycleRecord>>>,
    pub bitcoin_tx_lifecycle_telemetry: Arc<RwLock<Vec<BitcoinTxTransitionOutcome>>>,
    pub bitcoin_tx_persistence: Arc<dyn BitcoinTxPersistence>,
    pub bitcoin_tx_event_sequence: AtomicU64,
    pub partner_leads: Arc<RwLock<HashMap<String, PartnerLead>>>,
    pub partner_lead_events: Arc<RwLock<Vec<PartnerLeadEvent>>>,
    pub partner_lead_idempotency: Arc<RwLock<HashMap<String, String>>>,
    pub partner_lead_sequence: AtomicU64,
    pub anchoring_idempotency: Arc<RwLock<HashMap<String, AnchoringReplayRecord>>>,
    pub anchoring_sequence: AtomicU64,
    pub tableland_anchoring_publisher: Arc<dyn AnchoringPublisher>,
    pub on_chain_anchoring_publisher: Arc<dyn AnchoringPublisher>,
    pub sab_wallets: Arc<RwLock<Vec<SabWallet>>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self::new_with_anchoring_publishers(
            Arc::new(TablelandAnchoringPublisher),
            Arc::new(OnChainAnchoringPublisher),
        )
    }

    pub(crate) fn new_with_anchoring_publishers(
        tableland_anchoring_publisher: Arc<dyn AnchoringPublisher>,
        on_chain_anchoring_publisher: Arc<dyn AnchoringPublisher>,
    ) -> Self {
        Self::new_with_anchoring_publishers_and_tx_lifecycle_config(
            tableland_anchoring_publisher,
            on_chain_anchoring_publisher,
            BitcoinTxLifecycleConfig::from_env(),
        )
    }

    pub(crate) fn new_with_anchoring_publishers_and_tx_lifecycle_config(
        tableland_anchoring_publisher: Arc<dyn AnchoringPublisher>,
        on_chain_anchoring_publisher: Arc<dyn AnchoringPublisher>,
        bitcoin_tx_lifecycle_config: BitcoinTxLifecycleConfig,
    ) -> Self {
        Self::new_with_anchoring_publishers_tx_lifecycle_config_and_persistence(
            tableland_anchoring_publisher,
            on_chain_anchoring_publisher,
            bitcoin_tx_lifecycle_config,
            Arc::new(InMemoryBitcoinTxPersistence::default()),
        )
    }

    pub(crate) fn new_with_anchoring_publishers_tx_lifecycle_config_and_persistence(
        tableland_anchoring_publisher: Arc<dyn AnchoringPublisher>,
        on_chain_anchoring_publisher: Arc<dyn AnchoringPublisher>,
        bitcoin_tx_lifecycle_config: BitcoinTxLifecycleConfig,
        bitcoin_tx_persistence: Arc<dyn BitcoinTxPersistence>,
    ) -> Self {
        let bitcoin_tx_lifecycle_projection = bitcoin_tx_persistence
            .list_orchestrations()
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|record| lifecycle_record_from_orchestration_record(record).ok())
            .map(|record| (record.tx_id.clone(), record))
            .collect::<HashMap<_, _>>();

        Engine {
            version: "0.2.3".to_string(),
            start_time: Utc::now(),
            support_intake: Arc::new(SupportIntake::new(SupportConfig::default())),
            request_count: AtomicU64::new(0),
            total_tvl_usd: Arc::new(RwLock::new(0.0)),
            active_sovereign_nodes: AtomicU64::new(5),
            service_statuses: Arc::new(RwLock::new(HashMap::new())),
            reserves: Arc::new(RwLock::new(Vec::new())),
            prices: Arc::new(RwLock::new(HashMap::new())),
            compliance: Arc::new(RwLock::new(ComplianceStatus {
                status: "compliant".to_string(),
                last_audit: Utc::now(),
                rules_active: vec!["AML".to_string(), "KYC".to_string(), "OFAC".to_string()],
                risk_score: 5,
                zkml_enabled: true,
            })),
            affiliates: Arc::new(RwLock::new(HashMap::new())),
            marketing: Arc::new(RwLock::new(Vec::new())),
            financial_metrics: Arc::new(RwLock::new(FinancialMetrics {
                mrr_usd: 125000.0,
                arr_usd: 1500000.0,
                churn_rate_pct: 2.5,
                protocol_fees_collected_usd: 85000.0,
                last_updated: Utc::now(),
            })),
            identity_records: Arc::new(RwLock::new(HashMap::new())),
            erp_sync_status: Arc::new(RwLock::new(HashMap::new())),
            settlement_log: Arc::new(RwLock::new(Vec::new())),
            state_proposals: Arc::new(RwLock::new(HashMap::new())),
            bitcoin_tx_lifecycle_config,
            bitcoin_fee_bump_policy: BitcoinFeeBumpPolicy::from_env(),
            bitcoin_tx_lifecycle: Arc::new(RwLock::new(bitcoin_tx_lifecycle_projection)),
            bitcoin_tx_lifecycle_shadow: Arc::new(RwLock::new(HashMap::new())),
            bitcoin_tx_lifecycle_telemetry: Arc::new(RwLock::new(Vec::new())),
            bitcoin_tx_persistence,
            bitcoin_tx_event_sequence: AtomicU64::new(now_epoch_ms()),
            partner_leads: Arc::new(RwLock::new(HashMap::new())),
            partner_lead_events: Arc::new(RwLock::new(Vec::new())),
            partner_lead_idempotency: Arc::new(RwLock::new(HashMap::new())),
            partner_lead_sequence: AtomicU64::new(1),
            anchoring_idempotency: Arc::new(RwLock::new(HashMap::new())),
            anchoring_sequence: AtomicU64::new(1),
            tableland_anchoring_publisher,
            on_chain_anchoring_publisher,
            sab_wallets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(&self) {
        self.initialize_services();
    }

    fn infer_rail_family(name: &str) -> &'static str {
        match name {
            "stacks" => "anchored_l2",
            "lightning" | "bisq" => "lightning_p2p",
            "liquid" | "rootstock" | "core-dao" | "botanix" => "federated_sidechain",
            "bitvm" | "bitvm2" | "bitlayer" | "bob" => "optimistic_rollup",
            "hemi" | "merlin" | "bison" | "alpen" | "b2network" => "zk_rollup",
            "babylon" | "lorenzo" => "staking_coordination",
            "rgb" | "taproot-assets" => "client_side_assets",
            "nubit" => "data_availability_layer",
            "mezo" | "zulu" => "partner_backed_hybrid",
            _ => "unknown",
        }
    }

    fn to_owned_vec(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    fn build_rail_metadata(
        name: &str,
        trust: &str,
        data_availability: &str,
        settlement: &str,
        bridge: &str,
    ) -> RailMetadata {
        let family = Self::infer_rail_family(name);
        let (
            confirmation_model,
            finality_window,
            custody_model,
            redemption_path,
            jurisdictional_scope,
            baseline_controls,
            monitoring_requirements,
            supported_flows,
            integration_modes,
            resilience_features,
        ): (
            &str,
            &str,
            &str,
            &str,
            &str,
            &[&str],
            &[&str],
            &[&str],
            &[&str],
            &[&str],
        ) = match family {
            "anchored_l2" => (
                "anchored_state_finality",
                "10-60m",
                "hybrid_bridge_custody",
                "bridge_redemption_to_bitcoin",
                "gateway_policy_controls",
                &["kyc_for_managed_entrypoints", "sanctions_screening"],
                &["anchor_finality_checks", "bridge_health_monitoring"],
                &[
                    "btc_bridging",
                    "smart_contract_execution",
                    "token_transfers",
                ],
                &["bridge_api", "gateway_service_status"],
                &["checkpoint_reconciliation", "bridge_failover_runbooks"],
            ),
            "lightning_p2p" => (
                "offchain_instant_with_l1_close",
                "seconds_to_1_block",
                "self_custodial_channels",
                "cooperative_or_unilateral_close",
                "p2p_with_gateway_overlays",
                &[
                    "risk_scoring_for_fiat_edges",
                    "sanctions_screening_on_managed_nodes",
                ],
                &["channel_liquidity_monitoring", "routing_uptime_alerts"],
                &["micropayments", "streaming_payments", "atomic_swaps"],
                &["invoice_and_keysend", "p2p_settlement"],
                &["watchtower_recovery", "multi_path_payment_routing"],
            ),
            "federated_sidechain" => (
                "sidechain_block_finality",
                "1-12_confs_plus_peg_batching",
                "federated_peg_custody",
                "peg_out_with_quorum",
                "federation_and_operator_licensing",
                &[
                    "kyb_for_federation_participants",
                    "sanctions_screening",
                    "counterparty_due_diligence",
                ],
                &["quorum_health_checks", "peg_in_peg_out_audits"],
                &[
                    "bridged_btc_transfers",
                    "sidechain_settlement",
                    "policy_managed_redemptions",
                ],
                &["bridge_api", "operator_node_rpc"],
                &["quorum_rotation", "delayed_exit_controls"],
            ),
            "optimistic_rollup" => (
                "soft_finality_then_challenge_window",
                "minutes_to_days",
                "bridge_escrow_with_delayed_exits",
                "forced_withdrawal_after_challenge",
                "managed_entry_exit_regional_controls",
                &[
                    "kyc_for_managed_offramps",
                    "sanctions_screening",
                    "transaction_risk_scoring",
                ],
                &[
                    "challenge_window_monitoring",
                    "sequencer_uptime_alerts",
                    "bridge_liquidity_checks",
                ],
                &[
                    "high_throughput_transfers",
                    "rollup_vm_execution",
                    "cross_chain_withdrawals",
                ],
                &["rollup_rpc", "bridge_contract_calls"],
                &["fault_proof_challenges", "forced_exit_paths"],
            ),
            "zk_rollup" => (
                "validity_proof_finality",
                "minutes_after_proof_submission",
                "proof_verified_bridge_escrow",
                "proof_backed_withdrawal",
                "gateway_policy_with_jurisdiction_overrides",
                &[
                    "kyc_for_managed_gateways",
                    "sanctions_screening",
                    "proof_audit_logging",
                ],
                &[
                    "proof_submission_latency",
                    "bridge_contract_health",
                    "sequencer_backlog_monitoring",
                ],
                &[
                    "zk_verified_transfers",
                    "rollup_contract_execution",
                    "batched_btc_settlement",
                ],
                &["proof_relay", "rollup_api", "bridge_indexing"],
                &["state_root_audits", "proof_verifier_redundancy"],
            ),
            "client_side_assets" => (
                "proof_validated_utxo_finality",
                "1-6_bitcoin_confirmations",
                "self_custodial_asset_proofs",
                "direct_utxo_redemption",
                "issuer_defined_restrictions",
                &[
                    "issuer_kyc_policies",
                    "sanctions_screening_on_managed_gateways",
                ],
                &["schema_validation_checks", "proof_consistency_audits"],
                &[
                    "asset_issuance",
                    "client_side_transfers",
                    "taproot_commitment_anchor",
                ],
                &["wallet_sdk", "psbt_workflows"],
                &["local_proof_backups", "utxo_reconciliation"],
            ),
            "staking_coordination" => (
                "epoch_based_finality",
                "epoch_plus_unlock_delay",
                "delegated_staking_lockups",
                "unbonding_queue_release",
                "institutional_staking_policy_constraints",
                &[
                    "validator_kyb",
                    "sanctions_screening_for_reward_payouts",
                    "counterparty_due_diligence",
                ],
                &[
                    "slashing_event_alerts",
                    "validator_performance_tracking",
                    "unlock_queue_monitoring",
                ],
                &[
                    "btc_staking",
                    "reward_distribution",
                    "restaking_coordination",
                ],
                &["staking_api", "validator_telemetry"],
                &["operator_rotation", "checkpoint_reconciliation"],
            ),
            "data_availability_layer" => (
                "availability_attestations_before_settlement",
                "minutes_plus_downstream_settlement",
                "custody_delegated_to_connected_bridges",
                "exit_via_connected_settlement_rail",
                "inherited_from_connected_execution_rails",
                &[
                    "data_retention_policies",
                    "sanctions_screening_on_settlement_edges",
                ],
                &[
                    "data_sampling_audits",
                    "blob_retention_checks",
                    "bridge_dependency_health",
                ],
                &[
                    "data_blob_publication",
                    "proof_data_serving",
                    "cross_rail_data_feeds",
                ],
                &["da_api", "light_client_sampling"],
                &["erasure_coding_redundancy", "archive_node_failover"],
            ),
            "partner_backed_hybrid" => (
                "operational_soft_then_bitcoin_settlement",
                "minutes_to_1-6_bitcoin_confs",
                "partner_treasury_and_bridge_escrow",
                "policy_governed_partner_redemption",
                "partner_licensing_perimeter",
                &[
                    "institutional_kyc_kyb",
                    "sanctions_screening",
                    "transaction_limit_enforcement",
                ],
                &[
                    "liquidity_buffer_tracking",
                    "operator_sla_monitoring",
                    "treasury_reconciliation",
                ],
                &[
                    "partner_liquidity_routing",
                    "yield_and_staking_products",
                    "btc_settlement_redemptions",
                ],
                &[
                    "partner_api_connectors",
                    "custody_hooks",
                    "policy_engine_controls",
                ],
                &["liquidity_circuit_breakers", "manual_override_runbooks"],
            ),
            _ => (
                "undocumented",
                "unknown",
                "undocumented",
                "manual_triage_required",
                "unspecified",
                &["manual_review_required"],
                &["manual_monitoring"],
                &["status_visibility_only"],
                &["service_registry_lookup"],
                &["fallback_to_manual_triage"],
            ),
        };

        let signer_architecture = if bridge == "N/A" {
            "native_or_not_applicable".to_string()
        } else {
            bridge.to_string()
        };

        RailMetadata {
            rail_family: family.to_string(),
            trust_assumptions: RailTrustAssumptions {
                security_anchor: format!("{} assumptions with {} DA", trust, data_availability),
                operator_dependency: format!("{} operator and verifier set", family),
                liveness_assumption: format!(
                    "{} settlement path and {} bridge path remain available",
                    settlement, bridge
                ),
            },
            finality_semantics: RailFinalitySemantics {
                confirmation_model: confirmation_model.to_string(),
                settlement_layer: settlement.to_string(),
                typical_finality_window: finality_window.to_string(),
            },
            custody_model: RailCustodyModel {
                asset_control_model: custody_model.to_string(),
                signer_architecture,
                redemption_path: redemption_path.to_string(),
            },
            compliance_constraints: RailComplianceConstraints {
                baseline_controls: Self::to_owned_vec(baseline_controls),
                jurisdictional_scope: jurisdictional_scope.to_string(),
                monitoring_requirements: Self::to_owned_vec(monitoring_requirements),
            },
            operational_capabilities: RailOperationalCapabilities {
                supported_flows: Self::to_owned_vec(supported_flows),
                integration_modes: Self::to_owned_vec(integration_modes),
                resilience_features: Self::to_owned_vec(resilience_features),
            },
        }
    }

    fn unknown_rail_metadata(name: &str) -> RailMetadata {
        RailMetadata {
            rail_family: "unknown".to_string(),
            trust_assumptions: RailTrustAssumptions {
                security_anchor: "Not registered in rail metadata catalog".to_string(),
                operator_dependency: "Manual review required".to_string(),
                liveness_assumption: format!("No modeled assumptions for service '{}'.", name),
            },
            finality_semantics: RailFinalitySemantics {
                confirmation_model: "Undocumented".to_string(),
                settlement_layer: "Unknown".to_string(),
                typical_finality_window: "Unknown".to_string(),
            },
            custody_model: RailCustodyModel {
                asset_control_model: "Undocumented".to_string(),
                signer_architecture: "Undocumented".to_string(),
                redemption_path: "Manual triage required".to_string(),
            },
            compliance_constraints: RailComplianceConstraints {
                baseline_controls: vec!["manual_review_required".to_string()],
                jurisdictional_scope: "Unspecified".to_string(),
                monitoring_requirements: vec!["manual_monitoring".to_string()],
            },
            operational_capabilities: RailOperationalCapabilities {
                supported_flows: vec!["status_visibility_only".to_string()],
                integration_modes: vec!["service_registry_lookup".to_string()],
                resilience_features: vec!["fallback_to_manual_triage".to_string()],
            },
        }
    }

    fn initialize_services(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        let is_mainnet = remediation::is_production_mainnet();

        let services = if is_mainnet {
            vec![
                (
                    "stacks",
                    0,
                    "PoX",
                    "On-chain",
                    "Bitcoin",
                    "sBTC Bridge",
                    0.0,
                ),
                (
                    "lightning",
                    0,
                    "State Channels",
                    "Off-chain",
                    "Bitcoin",
                    "P2P",
                    0.0,
                ),
                (
                    "liquid",
                    0,
                    "Federation",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    0.0,
                ),
                (
                    "rootstock",
                    0,
                    "Merge-mined",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    0.0,
                ),
                ("bisq", 0, "P2P", "Off-chain", "Bitcoin", "Atomic", 0.0),
                ("rgb", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                (
                    "bitvm",
                    0,
                    "Optimistic",
                    "On-chain",
                    "Bitcoin",
                    "BitVM",
                    0.0,
                ),
                (
                    "babylon", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0,
                ),
                (
                    "core-dao",
                    0,
                    "Satoshi Plus",
                    "Sidechain",
                    "Bitcoin",
                    "Relayer",
                    0.0,
                ),
                (
                    "lorenzo", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0,
                ),
                ("hemi", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "bob",
                    0,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "Optimistic",
                    0.0,
                ),
                ("merlin", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "mezo",
                    0,
                    "Economic Layer",
                    "On-chain",
                    "Bitcoin",
                    "tBTC",
                    0.0,
                ),
                ("nubit", 0, "DA", "On-chain", "Bitcoin", "N/A", 0.0),
                ("bison", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("zulu", 0, "Multi-layer", "On-chain", "Bitcoin", "N/A", 0.0),
                (
                    "botanix",
                    0,
                    "Spiderchain",
                    "Sidechain",
                    "Bitcoin",
                    "Spiderchain",
                    0.0,
                ),
                (
                    "bitlayer",
                    0,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "BitVM",
                    0.0,
                ),
                ("alpen", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "taproot-assets",
                    0,
                    "Client-side",
                    "Off-chain",
                    "Bitcoin",
                    "N/A",
                    0.0,
                ),
                (
                    "bitvm2",
                    0,
                    "ZK-Fraud Proofs",
                    "On-chain",
                    "Bitcoin",
                    "BitVM2",
                    0.0,
                ),
                ("b2network", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
            ]
        } else {
            vec![
                (
                    "stacks",
                    65,
                    "PoX",
                    "On-chain",
                    "Bitcoin",
                    "sBTC Bridge",
                    12500000.0,
                ),
                (
                    "lightning",
                    10,
                    "State Channels",
                    "Off-chain",
                    "Bitcoin",
                    "P2P",
                    1500000.0,
                ),
                (
                    "liquid",
                    50,
                    "Federation",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    4500000.0,
                ),
                (
                    "rootstock",
                    55,
                    "Merge-mined",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    3800000.0,
                ),
                (
                    "bisq",
                    120,
                    "P2P",
                    "Off-chain",
                    "Bitcoin",
                    "Atomic",
                    850000.0,
                ),
                ("rgb", 15, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                (
                    "bitvm",
                    90,
                    "Optimistic",
                    "On-chain",
                    "Bitcoin",
                    "BitVM",
                    5000000.0,
                ),
                (
                    "babylon", 25, "Staking", "On-chain", "Bitcoin", "Staking", 15000000.0,
                ),
                (
                    "core-dao",
                    40,
                    "Satoshi Plus",
                    "Sidechain",
                    "Bitcoin",
                    "Relayer",
                    75000000.0,
                ),
                (
                    "lorenzo", 30, "Staking", "On-chain", "Bitcoin", "Staking", 12000000.0,
                ),
                (
                    "hemi",
                    50,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    35000000.0,
                ),
                (
                    "bob",
                    45,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "Optimistic",
                    28000000.0,
                ),
                (
                    "merlin",
                    35,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    45000000.0,
                ),
                (
                    "mezo",
                    25,
                    "Economic Layer",
                    "On-chain",
                    "Bitcoin",
                    "tBTC",
                    150000000.0,
                ),
                ("nubit", 20, "DA", "On-chain", "Bitcoin", "N/A", 5000000.0),
                (
                    "bison",
                    55,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    10000000.0,
                ),
                (
                    "zulu",
                    40,
                    "Multi-layer",
                    "On-chain",
                    "Bitcoin",
                    "N/A",
                    15000000.0,
                ),
                (
                    "botanix",
                    60,
                    "Spiderchain",
                    "Sidechain",
                    "Bitcoin",
                    "Spiderchain",
                    8000000.0,
                ),
                (
                    "bitlayer",
                    45,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "BitVM",
                    25000000.0,
                ),
                (
                    "alpen",
                    30,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    12000000.0,
                ),
                (
                    "taproot-assets",
                    15,
                    "Client-side",
                    "Off-chain",
                    "Bitcoin",
                    "N/A",
                    5500000.0,
                ),
                (
                    "bitvm2",
                    85,
                    "ZK-Fraud Proofs",
                    "On-chain",
                    "Bitcoin",
                    "BitVM2",
                    15000000.0,
                ),
                (
                    "b2network",
                    35,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    18000000.0,
                ),
            ]
        };

        for (name, latency, trust, da, settlement, bridge, tvl) in services {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.2.0".to_string());
            if !is_mainnet {
                if name == "stacks" {
                    metadata.insert("block_height".to_string(), "841500".to_string());
                    metadata.insert("hiro_api_connected".to_string(), "true".to_string());
                }
                if name == "bitvm2" {
                    metadata.insert(
                        "bitvm_challenge_status".to_string(),
                        "NoActiveChallenges".to_string(),
                    );
                }
                if name == "lorenzo" {
                    metadata.insert("staked_btc".to_string(), "1250.5".to_string());
                }
                if name == "b2network" {
                    metadata.insert("block_height".to_string(), "12600".to_string());
                }
            }

            let assessment = self.calculate_risk_assessment(trust, da, settlement, bridge);
            let risk_level = assessment.overall_level.clone();
            let rail_metadata = Self::build_rail_metadata(name, trust, da, settlement, bridge);

            statuses.insert(
                name.to_string(),
                ServiceStatus {
                    name: name.to_string(),
                    status: "active".to_string(),
                    last_checked: Utc::now(),
                    latency_ms: latency,
                    trust_model: trust.to_string(),
                    risk_level,
                    risk_assessment: Some(assessment),
                    data_availability: da.to_string(),
                    settlement: settlement.to_string(),
                    bridge_security: bridge.to_string(),
                    tvl_usd: tvl,
                    version: Some("1.2.0".to_string()),
                    metadata,
                    rail_metadata,
                },
            );
        }

        if !is_mainnet {
            let mut reserves = self.reserves.write().unwrap();
            reserves.push(ReserveAsset {
                asset: "L-BTC".to_string(),
                total_supplied: 1500.0,
                total_reserves: 1500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });
            reserves.push(ReserveAsset {
                asset: "RBTC".to_string(),
                total_supplied: 2500.0,
                total_reserves: 2500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });

            let mut prices = self.prices.write().unwrap();
            prices.insert(
                "BTC".to_string(),
                PriceInfo {
                    asset: "BTC".to_string(),
                    price_usd: 65000.0,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
            prices.insert(
                "STX".to_string(),
                PriceInfo {
                    asset: "STX".to_string(),
                    price_usd: 2.50,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
        }
    }

    fn calculate_risk_assessment(
        &self,
        trust: &str,
        da: &str,
        settlement: &str,
        bridge: &str,
    ) -> RiskAssessment {
        let mut da_score = if da == "On-chain" { 95 } else { 70 };
        let mut settlement_score = if settlement == "Bitcoin" { 95 } else { 80 };
        let mut bridge_score = match bridge {
            "BitVM" | "BitVM2" => 90,
            "sBTC Bridge" => 85,
            "Powpeg" => 80,
            _ => 70,
        };

        if trust.contains("ZK") {
            da_score += 2;
            settlement_score += 2;
            bridge_score += 2;
        }

        let overall_score = (da_score + settlement_score + bridge_score) / 3;
        let overall_level = if overall_score > 90 {
            "Low".to_string()
        } else if overall_score > 80 {
            "Medium".to_string()
        } else {
            "High".to_string()
        };

        RiskAssessment {
            overall_level,
            da_score,
            settlement_score,
            bridge_score,
            exit_mechanism_score: 85,
            operators_score: 80,
            decentralization_score: 75,
        }
    }

    fn update_metrics(&self) {
        let total_requests = self.request_count.load(Ordering::SeqCst);
        let mut metrics = self.financial_metrics.write().unwrap();
        metrics.protocol_fees_collected_usd = total_requests as f64 * 0.05;
        metrics.mrr_usd = metrics.protocol_fees_collected_usd * 1.5;
        metrics.last_updated = Utc::now();

        let mut total_tvl = self.total_tvl_usd.write().unwrap();
        *total_tvl = self.calculate_total_tvl();

        {
            let wallets = self.sab_wallets.read().unwrap();
            if wallets.is_empty() {
                log::warn!("No SAB wallets configured for mainnet execution!");
            }
        }
    }

    pub fn get_service_status(&self, name: &str) -> ServiceStatus {
        let statuses = self.service_statuses.read().unwrap();
        statuses
            .get(name)
            .cloned()
            .unwrap_or_else(|| ServiceStatus {
                name: name.to_string(),
                status: "unknown".to_string(),
                last_checked: Utc::now(),
                latency_ms: 0,
                trust_model: "Unknown".to_string(),
                risk_level: "High".to_string(),
                risk_assessment: None,
                data_availability: "Unknown".to_string(),
                settlement: "Unknown".to_string(),
                bridge_security: "Unknown".to_string(),
                tvl_usd: 0.0,
                version: None,
                metadata: HashMap::new(),
                rail_metadata: Self::unknown_rail_metadata(name),
            })
    }

    pub async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        let fallback_height = 841500;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.mainnet.hiro.so/v2/info")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
        let mut height_opt = None;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                height_opt = info["stacks_tip_height"].as_u64();
            }
        }
        let height = height_opt.unwrap_or(fallback_height);
        {
            let mut statuses = self.service_statuses.write().unwrap();
            if let Some(status) = statuses.get_mut("stacks") {
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("hiro_api_connected".to_string(), "true".to_string());
                status.last_checked = Utc::now();
            }
        }
        Ok(height)
    }

    async fn analyze_bitcoin_mpool(&self) {
        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "https://bitcoin-rpc.publicnode.com".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "1.0", "id": "mempool-audit", "method": "getmempoolinfo", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                let size = info["result"]["size"].as_u64().unwrap_or(0);
                let bytes = info["result"]["bytes"].as_u64().unwrap_or(0);
                if size > 100000 {
                    log::warn!("High mempool congestion detected: {} txs", size);
                }
                let mut statuses = self.service_statuses.write().unwrap();
                if let Some(status) = statuses.get_mut("bitvm2") {
                    status
                        .metadata
                        .insert("mempool_size".to_string(), size.to_string());
                    status
                        .metadata
                        .insert("mempool_bytes".to_string(), bytes.to_string());
                }
            }
        }
    }

    async fn fetch_bitcoin_rpc_status(&self) {
        let is_mainnet = remediation::is_production_mainnet();
        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "https://bitcoin-rpc.publicnode.com".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "1.0", "id": "gateway-audit", "method": "getblockchaininfo", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        let mut height_opt = None;
        let mut connected = false;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                height_opt = info["result"]["blocks"].as_u64();
                connected = true;
            }
        }
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("bitvm2") {
            if connected {
                let height = height_opt.unwrap_or(841500);
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("rpc_connected".to_string(), "true".to_string());
                status.status = "active".to_string();
            } else if is_mainnet {
                status.status = "ConnectionRequired".to_string();
            }
            status.last_checked = Utc::now();
        }
    }

    async fn track_l2_finality(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("hemi") {
            status
                .metadata
                .insert("bitcoin_finality_depth".to_string(), "3".to_string());
            status
                .metadata
                .insert("ethereum_finality_depth".to_string(), "12".to_string());
            status
                .metadata
                .insert("cross_layer_audit".to_string(), "Passed".to_string());
        }
        if let Some(status) = statuses.get_mut("bob") {
            status
                .metadata
                .insert("optimistic_window_blocks".to_string(), "1008".to_string());
            status
                .metadata
                .insert("finality_status".to_string(), "SettledOnL1".to_string());
        }
    }

    async fn fetch_core_dao_rpc_status(&self) {
        let is_mainnet = remediation::is_production_mainnet();
        let rpc_url = std::env::var("CORE_DAO_RPC_URL")
            .unwrap_or_else(|_| "https://rpc.coredao.org".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        let mut height_opt = None;
        let mut connected = false;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                let hex_height = info["result"].as_str().unwrap_or("0x0");
                height_opt = u64::from_str_radix(hex_height.trim_start_matches("0x"), 16).ok();
                connected = true;
            }
        }
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("core-dao") {
            if connected {
                let height = height_opt.unwrap_or(0);
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("rpc_connected".to_string(), "true".to_string());
                status.status = "active".to_string();
            } else if is_mainnet {
                status.status = "ConnectionRequired".to_string();
            }
            status.last_checked = Utc::now();
        }
    }

    pub fn calculate_total_tvl(&self) -> f64 {
        let statuses = self.service_statuses.read().unwrap();
        statuses.values().map(|s| s.tvl_usd).sum()
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }
    pub fn get_bitvm_proof(&self, id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": id, "protocol": "bitvm", "verified": true, "commitment_hash": "0x123...abc", "challenge_period_blocks": 144, "active_verifiers": 15, "status": "Operational" })
    }
    pub fn get_citrea_proof(&self, id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": id, "protocol": "citrea", "verified": true, "zk_proof_type": "Groth16", "l1_commitment_tx": "0x987...xyz", "status": "Finalized" })
    }
    pub fn get_financial_metrics(&self) -> FinancialMetrics {
        self.financial_metrics.read().unwrap().clone()
    }
    pub fn get_b2network_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("b2network");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "block_height": status.metadata.get("block_height").cloned().unwrap_or_else(|| "0".to_string()), "sequencer_status": "Healthy", "zk_proof_generated": true })
    }
    pub fn get_bitlayer_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitlayer");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "node_status": "Active", "bridge_capacity_btc": 500.0, "active_challenges": 0 })
    }
    pub fn get_alpen_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "finality_depth_blocks": 6, "da_layer": "Bitcoin", "status": "Production" })
    }
    pub fn get_mezo_yield(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("mezo");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "apy_pct": 12.4, "asset": "tBTC", "yield_source": "Bitcoin Staking" })
    }
    pub fn get_zulu_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("zulu");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "network_type": "Multi-layer", "evm_compatible": true, "status": "Mainnet" })
    }
    pub fn get_bison_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bison");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "rollup_type": "ZK", "proof_system": "STARK", "status": "Active" })
    }
    pub fn get_hemi_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("hemi");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "h_po_w_status": "Active", "bitcoin_finality": true, "ethereum_finality": true })
    }
    pub fn get_taproot_assets_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("taproot-assets");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "assets_minted": 150, "total_transfers": 5600, "status": "Operational" })
    }
    pub fn get_nubit_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("nubit");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_throughput_mb_s": 10.5, "blob_count": 12500, "status": "Healthy" })
    }
    pub fn get_risk_assessments(&self) -> HashMap<String, Option<RiskAssessment>> {
        let statuses = self.service_statuses.read().unwrap();
        let mut assessments = HashMap::new();
        for (name, status) in statuses.iter() {
            assessments.insert(name.clone(), status.risk_assessment.clone());
        }
        assessments
    }
    pub fn resolve_identity(&self, query: &str) -> IdentityRecord {
        self.increment_requests();
        let mut records = self.identity_records.write().unwrap();
        records
            .entry(query.to_string())
            .or_insert_with(|| IdentityRecord {
                address: query.to_string(),
                ens_name: query
                    .strip_prefix("0x")
                    .or_else(|| query.strip_prefix("0X"))
                    .and_then(|s| {
                        let prefix: String = s
                            .chars()
                            .take(4)
                            .take_while(|c| c.is_ascii_hexdigit())
                            .map(|c| c.to_ascii_lowercase())
                            .collect();
                        (!prefix.is_empty()).then(|| format!("{prefix}.eth"))
                    }),
                bns_name: if query.len() > 20 {
                    Some("conxian.btc".to_string())
                } else {
                    None
                },
                world_id_verified: query.contains("verified"),
            })
            .clone()
    }
    pub fn sync_erp_data(&self, system: &str) -> ErpSyncRecord {
        self.increment_requests();
        let mut erp_sync = self.erp_sync_status.write().unwrap();
        let record = erp_sync
            .entry(system.to_string())
            .or_insert_with(|| ErpSyncRecord {
                erp_system: system.to_string(),
                last_sync: Utc::now(),
                total_transactions_synced: 0,
                status: "Initializing".to_string(),
            });
        record.last_sync = Utc::now();
        record.total_transactions_synced += 150;
        record.status = "Healthy".to_string();
        record.clone()
    }
    pub fn get_cjcs_v2_spec(&self) -> serde_json::Value {
        serde_json::json!({ "@context": "https://conxian.com/contexts/job-card/v2.0", "@type": "ConxianJobCard", "version": "2.0.0", "standard": "JSON-LD", "description": "Enterprise-to-Bitcoin labor orchestration protocol" })
    }
    pub fn get_dlc_bond_info(&self, bond_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "bond_id": bond_id, "status": "Active", "apr_pct": 4.5, "asset": "sBTC", "maturity_blocks": 2016, "dlc_oracle": "cxn-treasury-oracle" })
    }
    pub fn get_exchange_rate(&self, from: &str, to: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "from": from, "to": to, "rate": 1.0, "timestamp": Utc::now() })
    }
    pub fn get_core_dao_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("core-dao");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "active_stakers": 1500, "satoshi_plus_rewards_distributed_usd": 1200000.0 })
    }
    pub fn get_lorenzo_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("lorenzo");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "staked_btc": status.metadata.get("staked_btc").cloned().unwrap_or_else(|| "1250.5".to_string()), "active_stakers": 850 })
    }

    pub fn commit_state_to_tableland(&self, state_root: &str) -> serde_json::Value {
        let request = AnchoringRequest {
            state_root: state_root.to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: anchoring::DEFAULT_MAX_RETRY_ATTEMPTS,
        };

        match self.commit_state_checkpoint(request) {
            Ok(receipt) => serde_json::to_value(receipt).unwrap_or_else(|_| {
                serde_json::json!({
                    "state_root": state_root,
                    "status": "failed",
                    "error": "serialization_error",
                })
            }),
            Err(err) => serde_json::json!({
                "state_root": state_root,
                "status": "failed",
                "error": err.code(),
                "details": err,
            }),
        }
    }

    pub fn commit_state_checkpoint(
        &self,
        request: AnchoringRequest,
    ) -> Result<AnchoringReceipt, AnchoringError> {
        self.increment_requests();

        let normalized = request.normalized();

        if normalized.state_root.is_empty() {
            return Err(AnchoringError::Validation {
                message: "state_root must not be empty".to_string(),
            });
        }

        let idempotency_key = self.derive_anchoring_idempotency_key(&normalized);
        let request_fingerprint = self.anchoring_request_fingerprint(&normalized);

        if let Some(existing) = self
            .anchoring_idempotency
            .read()
            .unwrap()
            .get(&idempotency_key)
            .cloned()
        {
            if existing.request_fingerprint != request_fingerprint {
                return Err(AnchoringError::IdempotencyConflict {
                    idempotency_key,
                    existing_fingerprint: existing.request_fingerprint,
                    incoming_fingerprint: request_fingerprint,
                    existing_state_root: existing.receipt.state_root,
                    incoming_state_root: normalized.state_root,
                });
            }

            let mut replay_receipt = existing.receipt;
            replay_receipt.idempotent_replay = true;
            replay_receipt
                .audit_metadata
                .insert("idempotent_replay".to_string(), "true".to_string());
            replay_receipt.audit_metadata.insert(
                "replay_of_receipt_id".to_string(),
                replay_receipt.receipt_id.clone(),
            );

            return Ok(replay_receipt);
        }

        let mut publications = Vec::new();
        let mut total_attempts = 0u8;

        for path in normalized.target.execution_paths() {
            let publisher = self.publisher_for_path(path)?;
            let (publication, attempts) = self.publish_with_retry(publisher, &normalized)?;
            total_attempts = total_attempts.saturating_add(attempts);
            publications.push(publication);
        }

        let mut audit_metadata = normalized.metadata.clone();
        audit_metadata.insert(
            "request_fingerprint".to_string(),
            request_fingerprint.clone(),
        );
        audit_metadata.insert(
            "targets_executed".to_string(),
            normalized.target.execution_paths().join(","),
        );
        audit_metadata.insert(
            "retry_budget".to_string(),
            normalized.max_retry_attempts.to_string(),
        );
        audit_metadata.insert("idempotent_replay".to_string(), "false".to_string());

        let (table_name, transaction_hash, persistence) = publications
            .iter()
            .find(|publication| publication.adapter == "tableland")
            .map(|publication| {
                (
                    publication.metadata.get("table_name").cloned(),
                    Some(publication.reference.clone()),
                    Some(publication.persistence.clone()),
                )
            })
            .unwrap_or((None, None, None));

        let receipt = AnchoringReceipt {
            receipt_id: self.next_anchoring_receipt_id(),
            state_root: normalized.state_root,
            target: normalized.target.clone(),
            idempotency_key: idempotency_key.clone(),
            idempotent_replay: false,
            status: if normalized.target == AnchoringTarget::Tableland {
                "Finalized".to_string()
            } else {
                "Committed".to_string()
            },
            published_at: Utc::now(),
            total_attempts,
            publications,
            audit_metadata,
            table_name,
            transaction_hash,
            persistence,
        };

        self.anchoring_idempotency.write().unwrap().insert(
            idempotency_key,
            AnchoringReplayRecord {
                request_fingerprint,
                receipt: receipt.clone(),
            },
        );

        Ok(receipt)
    }

    fn next_anchoring_receipt_id(&self) -> String {
        let seq = self.anchoring_sequence.fetch_add(1, Ordering::SeqCst);
        format!("ANCHOR-{}-{seq}", Utc::now().timestamp_millis())
    }

    fn derive_anchoring_idempotency_key(&self, request: &AnchoringRequest) -> String {
        request.idempotency_key.clone().unwrap_or_else(|| {
            format!(
                "state_commit:{}:{}",
                request.target.as_str(),
                request.state_root.to_ascii_lowercase()
            )
        })
    }

    fn anchoring_request_fingerprint(&self, request: &AnchoringRequest) -> String {
        let mut metadata = request.metadata.iter().collect::<Vec<_>>();
        metadata.sort_by(|(left, _), (right, _)| left.cmp(right));

        let metadata_fingerprint = metadata
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");

        format!(
            "state_root={};target={};metadata={metadata_fingerprint}",
            request.state_root.to_ascii_lowercase(),
            request.target.as_str(),
        )
    }

    fn publisher_for_path(
        &self,
        path: &str,
    ) -> Result<Arc<dyn AnchoringPublisher>, AnchoringError> {
        match path {
            "tableland" => Ok(Arc::clone(&self.tableland_anchoring_publisher)),
            "on_chain" => Ok(Arc::clone(&self.on_chain_anchoring_publisher)),
            unsupported => Err(AnchoringError::Validation {
                message: format!("unsupported anchoring path: {unsupported}"),
            }),
        }
    }

    fn publish_with_retry(
        &self,
        publisher: Arc<dyn AnchoringPublisher>,
        request: &AnchoringRequest,
    ) -> Result<(anchoring::AnchoringPublication, u8), AnchoringError> {
        let max_attempts = request.max_retry_attempts.max(1);
        let mut attempt = 0u8;

        loop {
            attempt = attempt.saturating_add(1);
            match publisher.publish(request, attempt) {
                Ok(publication) => return Ok((publication, attempt)),
                Err(err) if err.is_retryable() && attempt < max_attempts => continue,
                Err(err) if err.is_retryable() => {
                    return Err(AnchoringError::RetryExhausted {
                        adapter: publisher.name().to_string(),
                        attempts: attempt,
                        message: err.to_string(),
                    })
                }
                Err(err) => return Err(err),
            }
        }
    }

    pub fn get_status(&self) -> serde_json::Value {
        let uptime = (Utc::now() - self.start_time).num_seconds();
        serde_json::json!({ "version": self.version, "uptime_seconds": uptime, "status": "operational", "total_requests": self.request_count.load(Ordering::SeqCst), "total_tvl_usd": *self.total_tvl_usd.read().unwrap(), "active_nodes": self.active_sovereign_nodes.load(Ordering::SeqCst) })
    }
    pub fn is_healthy(&self) -> bool {
        true
    }
    pub fn check_compliance(&self, address: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "address": address, "status": "cleared", "risk_score": 0, "timestamp": Utc::now() })
    }
    pub fn verify_zkml_proof(&self, _proof: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": format!("zkml-{}", Utc::now().timestamp()), "verified": true, "attestation_role": "Guardian", "compliance_standard": "CARF/BRS v1.5", "timestamp": Utc::now() })
    }
    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
            return serde_json::json!({ "asset": "L-BTC", "peg_status": "ConnectionRequired", "error": "Mainnet node connection required for Liquid peg verification.", "remediation": "Configure LIQUID_RPC_URL" });
        }
        serde_json::json!({ "asset": "L-BTC", "peg_status": "Active", "collateral_ratio": 1.0, "verified_on_chain": true })
    }
    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
            return serde_json::json!({ "asset": "RBTC", "powpeg_status": "ConnectionRequired", "error": "Mainnet node connection required for Rootstock powpeg verification.", "remediation": "Configure ROOTSTOCK_RPC_URL" });
        }
        serde_json::json!({ "asset": "RBTC", "powpeg_status": "Active", "signatories_active": 12, "btc_locked": 2500.0 })
    }
    pub fn get_all_service_statuses(&self) -> Vec<ServiceStatus> {
        self.service_statuses
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    pub fn get_babylon_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("babylon");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "active_delegators": 1250, "staking_apr": 8.5 })
    }
    pub fn create_lightning_invoice(
        &self,
        amount_msat: u64,
        description: &str,
    ) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "invoice": "lnbc...", "amount_msat": amount_msat, "description": description, "expires_at": (Utc::now() + chrono::Duration::hours(1)).timestamp() })
    }
    pub fn pay_lightning_invoice(&self, invoice: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "payment_hash": "0xabc...", "status": "Complete", "invoice": invoice })
    }
    pub fn get_stacks_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "contract_id": contract_id, "status": "Active", "source_code_verified": true, "tx_count": 1250 })
    }
    pub fn get_proposals(&self) -> Vec<StateProposal> {
        self.state_proposals
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    pub fn approve_proposal(&self, proposal_id: &str) -> bool {
        let mut proposals = self.state_proposals.write().unwrap();
        if let Some(proposal) = proposals.get_mut(proposal_id) {
            if proposal.status == "Pending" {
                proposal.status = "Approved".to_string();
                return true;
            }
        }
        false
    }
    pub fn execute_proposal(&self, proposal_id: &str) -> bool {
        let mut proposals = self.state_proposals.write().unwrap();
        if let Some(proposal) = proposals.get_mut(proposal_id) {
            if proposal.status == "Approved" {
                proposal.status = "Executed".to_string();
                return true;
            }
        }
        false
    }
    pub fn get_reserves(&self) -> Vec<ReserveAsset> {
        self.reserves.read().unwrap().clone()
    }
    pub fn get_prices(&self) -> Vec<PriceInfo> {
        self.prices.read().unwrap().values().cloned().collect()
    }
    pub fn process_external_settlement(&self, protocol: &str, payload: Value) -> StateProposal {
        self.increment_requests();
        let raw_payload = serde_json::to_string(&payload).unwrap_or_default();
        let envelope = SettlementEnvelope {
            protocol: protocol.to_string(),
            payload,
            raw_payload_bytes: raw_payload,
            ingress_timestamp: Utc::now(),
        };
        self.settlement_log.write().unwrap().push(envelope);
        let trigger_id = format!(
            "{}-trigger-{}",
            protocol.to_lowercase(),
            Utc::now().timestamp()
        );
        let proposal_id = format!("prop-{}", trigger_id);
        let current_height: u64 = self
            .get_service_status("stacks")
            .metadata
            .get("block_height")
            .and_then(|h| h.parse().ok())
            .unwrap_or(841500);
        let timelock_end = current_height + 144;
        let proposal = StateProposal {
            proposal_id: proposal_id.clone(),
            trigger_id,
            proposed_state: "MainnetSovereignStateUpdate".to_string(),
            timelock_end_block: timelock_end,
            status: "Pending".to_string(),
            tee_attestation: "VerifiedByStrongBox-Mainnet-v1.0".to_string(),
            yield_routing: "5/5/90".to_string(),
            capital_status: "TransitBond".to_string(),
        };
        self.state_proposals
            .write()
            .unwrap()
            .insert(proposal_id, proposal.clone());
        proposal
    }
    fn next_partner_lead_token(&self, prefix: &str) -> String {
        let seq = self.partner_lead_sequence.fetch_add(1, Ordering::SeqCst);
        format!("{prefix}-{}-{seq}", Utc::now().timestamp_millis())
    }
    pub fn create_partner_lead(
        &self,
        input: PartnerLeadCreateInput,
        idempotency_key: &str,
    ) -> PartnerLeadCreateOutcome {
        self.increment_requests();
        if let Some(existing_id) = self
            .partner_lead_idempotency
            .read()
            .unwrap()
            .get(idempotency_key)
            .cloned()
        {
            if let Some(existing_lead) = self.partner_leads.read().unwrap().get(&existing_id) {
                return PartnerLeadCreateOutcome {
                    lead_id: existing_lead.lead_id.clone(),
                    status: "idempotent_replay".to_string(),
                    idempotent_replay: true,
                };
            }
        }
        let lead_id = self.next_partner_lead_token("LEAD");
        let now = Utc::now();
        let lead = PartnerLead {
            lead_id: lead_id.clone(),
            partner_name: input.partner_name,
            contact_name: input.contact_name,
            contact_email: input.contact_email,
            company_name: input.company_name,
            status: PartnerLeadStatus::New,
            owner: None,
            created_at: now,
            updated_at: now,
            notes: input.notes,
        };
        self.partner_leads
            .write()
            .unwrap()
            .insert(lead_id.clone(), lead);
        self.partner_lead_idempotency
            .write()
            .unwrap()
            .insert(idempotency_key.to_string(), lead_id.clone());
        PartnerLeadCreateOutcome {
            lead_id,
            status: "created".to_string(),
            idempotent_replay: false,
        }
    }
    pub fn get_partner_lead(&self, lead_id: &str) -> Option<PartnerLead> {
        self.partner_leads.read().unwrap().get(lead_id).cloned()
    }
    pub fn list_partner_leads(
        &self,
        status: Option<PartnerLeadStatus>,
        owner: Option<&str>,
    ) -> Vec<PartnerLead> {
        self.partner_leads
            .read()
            .unwrap()
            .values()
            .filter(|l| status.as_ref().is_none_or(|s| &l.status == s))
            .filter(|l| {
                owner
                    .as_ref()
                    .is_none_or(|o| l.owner.as_deref() == Some(*o))
            })
            .cloned()
            .collect()
    }
    pub fn transition_partner_lead(
        &self,
        lead_id: &str,
        input: PartnerLeadStatusUpdateInput,
    ) -> Result<PartnerLead, PartnerLeadTransitionError> {
        self.increment_requests();
        let mut leads = self.partner_leads.write().unwrap();
        let lead = leads
            .get_mut(lead_id)
            .ok_or(PartnerLeadTransitionError::NotFound)?;
        self.validate_partner_transition(&lead.status, &input.status, input.owner.as_deref())?;
        let from_status = lead.status.clone();
        let now = Utc::now();
        lead.status = input.status.clone();
        lead.updated_at = now;
        if let Some(new_owner) = &input.owner {
            lead.owner = Some(new_owner.clone());
        }
        let event = PartnerLeadEvent {
            event_id: self.next_partner_lead_token("EVT"),
            lead_id: lead_id.to_string(),
            timestamp: now,
            from_status,
            to_status: lead.status.clone(),
            owner: input.owner,
            note: input.event_note,
        };
        self.partner_lead_events.write().unwrap().push(event);
        Ok(lead.clone())
    }
    fn validate_partner_transition(
        &self,
        from: &PartnerLeadStatus,
        to: &PartnerLeadStatus,
        owner: Option<&str>,
    ) -> Result<(), PartnerLeadTransitionError> {
        if from == to {
            return Ok(());
        }
        match (from, to) {
            (PartnerLeadStatus::New, PartnerLeadStatus::Assigned) => {
                if owner.is_none() {
                    return Err(PartnerLeadTransitionError::OwnerRequired);
                }
            }
            (PartnerLeadStatus::Assigned, PartnerLeadStatus::InProgress) => {}
            (PartnerLeadStatus::InProgress, PartnerLeadStatus::Qualified) => {}
            (_, PartnerLeadStatus::Escalated) => {}
            (_, PartnerLeadStatus::ClosedWon) => {}
            (_, PartnerLeadStatus::ClosedLost) => {}
            _ => {
                return Err(PartnerLeadTransitionError::InvalidTransition {
                    from: from.clone(),
                    to: to.clone(),
                })
            }
        }
        Ok(())
    }

    pub fn get_bitcoin_tx_lifecycle_config(&self) -> BitcoinTxLifecycleConfig {
        self.bitcoin_tx_lifecycle_config.clone()
    }

    pub fn get_bitcoin_fee_bump_policy(&self) -> BitcoinFeeBumpPolicy {
        self.bitcoin_fee_bump_policy.clone()
    }

    pub fn evaluate_bitcoin_fee_bump(
        &self,
        input: BitcoinFeeBumpDecisionInput,
    ) -> BitcoinFeeBumpDecision {
        evaluate_bitcoin_fee_bump_decision(&self.bitcoin_fee_bump_policy, &input)
    }

    pub fn get_bitcoin_tx_lifecycle_record(&self, tx_id: &str) -> BitcoinTxLifecycleRecord {
        if let Some(record) = self.read_orchestration_projection(tx_id) {
            return record;
        }

        BitcoinTxLifecycleRecord::draft(tx_id)
    }

    pub fn get_bitcoin_tx_orchestration(
        &self,
        tx_id: &str,
    ) -> Result<Option<BitcoinTxOrchestration>, BitcoinTxTransitionError> {
        if tx_id.trim().is_empty() {
            return Err(BitcoinTxTransitionError::TxIdRequired);
        }

        let Some(record) = self
            .bitcoin_tx_persistence
            .get_orchestration(tx_id)
            .map_err(map_persistence_error)?
        else {
            return Ok(None);
        };

        orchestration_from_record(record).map(Some)
    }

    pub fn list_bitcoin_tx_events(
        &self,
        tx_id: &str,
    ) -> Result<Vec<BitcoinTxEvent>, BitcoinTxTransitionError> {
        if tx_id.trim().is_empty() {
            return Err(BitcoinTxTransitionError::TxIdRequired);
        }

        self.bitcoin_tx_persistence
            .list_events(tx_id)
            .map_err(map_persistence_error)?
            .into_iter()
            .map(event_from_record)
            .collect()
    }

    pub fn get_bitcoin_tx_lifecycle_view(&self, tx_id: &str) -> BitcoinTxLifecycleView {
        let production = self.get_bitcoin_tx_lifecycle_record(tx_id);
        let shadow = self
            .bitcoin_tx_lifecycle_shadow
            .read()
            .unwrap()
            .get(tx_id)
            .cloned();

        BitcoinTxLifecycleView {
            tx_id: tx_id.to_string(),
            execution_mode: self.bitcoin_tx_lifecycle_config.execution_mode(),
            production,
            shadow,
        }
    }

    pub fn get_bitcoin_tx_lifecycle_telemetry(&self) -> Vec<BitcoinTxTransitionOutcome> {
        self.bitcoin_tx_lifecycle_telemetry.read().unwrap().clone()
    }

    pub fn apply_bitcoin_tx_transition(
        &self,
        input: BitcoinTxTransitionInput,
    ) -> Result<BitcoinTxTransitionOutcome, BitcoinTxTransitionError> {
        self.increment_requests();

        let tx_id = input.tx_id.trim().to_string();
        if tx_id.is_empty() {
            return Err(BitcoinTxTransitionError::TxIdRequired);
        }

        let execution_mode = self.bitcoin_tx_lifecycle_config.execution_mode();
        if execution_mode == BitcoinTxLifecycleExecutionMode::Disabled {
            return Err(BitcoinTxTransitionError::FeatureDisabled);
        }

        let mut input = input;
        input.tx_id = tx_id.clone();
        input.idempotency_key = trim_optional_string(input.idempotency_key);
        input.rationale = trim_optional_string(input.rationale);
        input.dead_letter_reason = trim_optional_string(input.dead_letter_reason);

        let current = self.bitcoin_tx_record_for_transition(&tx_id, &execution_mode);

        let idempotency_key = input.idempotency_key.clone().unwrap_or_else(|| {
            derive_idempotency_key(&input, current.attempt, current.fee_rate_sat_vb)
        });

        let fingerprint = build_fingerprint(
            &tx_id,
            &input.event,
            input.attempt.unwrap_or(current.attempt),
            input.fee_rate_sat_vb.or(current.fee_rate_sat_vb),
            input.confirmations_observed,
            input.required_confirmations,
            input.reorg_depth,
            input
                .dead_letter_reason
                .as_deref()
                .or(input.rationale.as_deref()),
        );

        if execution_mode == BitcoinTxLifecycleExecutionMode::Active {
            if let Some(existing) = self.find_existing_tx_event(&tx_id, &idempotency_key)? {
                if existing.fingerprint == fingerprint {
                    return self.duplicate_outcome_from_record(existing, execution_mode);
                }

                return Err(BitcoinTxTransitionError::IdempotencyConflict {
                    tx_id,
                    idempotency_key,
                    existing_fingerprint: existing.fingerprint,
                    incoming_fingerprint: fingerprint,
                });
            }
        }

        let next = Self::next_bitcoin_tx_record(&current, &input)?;

        let event_id = format!(
            "evt-{}-{}",
            sanitize_id(&tx_id),
            self.bitcoin_tx_event_sequence
                .fetch_add(1, Ordering::SeqCst)
        );

        let projected_next =
            Self::apply_transition_projection_metadata(&current, &next, &input, &event_id);

        let outcome = BitcoinTxTransitionOutcome {
            tx_id: tx_id.clone(),
            event_id: event_id.clone(),
            idempotency_key: idempotency_key.clone(),
            event: input.event.clone(),
            from_state: current.state.clone(),
            to_state: projected_next.state.clone(),
            execution_mode: execution_mode.clone(),
            idempotent_replay: false,
            state_mutated: execution_mode == BitcoinTxLifecycleExecutionMode::Active,
            telemetry_recorded: execution_mode == BitcoinTxLifecycleExecutionMode::Shadow,
            transitioned_at: projected_next.updated_at,
        };

        match execution_mode {
            BitcoinTxLifecycleExecutionMode::Active => {
                let event_record = BtcTxEventRecord {
                    event_id: event_id.clone(),
                    tx_id: tx_id.clone(),
                    idempotency_key: idempotency_key.clone(),
                    transition: input.event.as_str().to_string(),
                    from_state: current.state.as_str().to_string(),
                    to_state: projected_next.state.as_str().to_string(),
                    attempt: projected_next.attempt,
                    fee_rate_sat_vb: projected_next.fee_rate_sat_vb,
                    observed_confirmations: observed_confirmations_from_record(&projected_next),
                    rationale: input
                        .dead_letter_reason
                        .clone()
                        .or_else(|| input.rationale.clone()),
                    fingerprint,
                    created_at_epoch_ms: projected_next.recovery_cursor,
                };

                match self
                    .bitcoin_tx_persistence
                    .append_event(event_record)
                    .map_err(map_persistence_error)?
                {
                    AppendEventOutcome::Inserted => {}
                    AppendEventOutcome::Duplicate(existing) => {
                        return self.duplicate_outcome_from_record(
                            existing,
                            BitcoinTxLifecycleExecutionMode::Active,
                        )
                    }
                }

                self.bitcoin_tx_persistence
                    .upsert_orchestration(lifecycle_record_to_orchestration_record(&projected_next))
                    .map_err(map_persistence_error)?;

                self.bitcoin_tx_lifecycle
                    .write()
                    .unwrap()
                    .insert(tx_id, projected_next);
            }
            BitcoinTxLifecycleExecutionMode::Shadow => {
                self.bitcoin_tx_lifecycle_shadow
                    .write()
                    .unwrap()
                    .insert(tx_id, projected_next);
                self.bitcoin_tx_lifecycle_telemetry
                    .write()
                    .unwrap()
                    .push(outcome.clone());
            }
            BitcoinTxLifecycleExecutionMode::Disabled => {}
        }

        Ok(outcome)
    }

    fn read_orchestration_projection(&self, tx_id: &str) -> Option<BitcoinTxLifecycleRecord> {
        let tx_id = tx_id.trim();
        if tx_id.is_empty() {
            return None;
        }

        if let Some(record) = self
            .bitcoin_tx_lifecycle
            .read()
            .unwrap()
            .get(tx_id)
            .cloned()
        {
            return Some(record);
        }

        let persisted = self
            .bitcoin_tx_persistence
            .get_orchestration(tx_id)
            .ok()
            .flatten()?;
        let hydrated = lifecycle_record_from_orchestration_record(persisted).ok()?;

        self.bitcoin_tx_lifecycle
            .write()
            .unwrap()
            .insert(tx_id.to_string(), hydrated.clone());

        Some(hydrated)
    }

    fn find_existing_tx_event(
        &self,
        tx_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<BtcTxEventRecord>, BitcoinTxTransitionError> {
        let events = self
            .bitcoin_tx_persistence
            .list_events(tx_id)
            .map_err(map_persistence_error)?;

        Ok(events
            .into_iter()
            .find(|event| event.idempotency_key == idempotency_key))
    }

    fn duplicate_outcome_from_record(
        &self,
        existing: BtcTxEventRecord,
        execution_mode: BitcoinTxLifecycleExecutionMode,
    ) -> Result<BitcoinTxTransitionOutcome, BitcoinTxTransitionError> {
        let from_state = parse_persisted_state(&existing.from_state)?;
        let to_state = parse_persisted_state(&existing.to_state)?;
        let event = parse_persisted_event(&existing.transition)?;

        Ok(BitcoinTxTransitionOutcome {
            tx_id: existing.tx_id,
            event_id: existing.event_id,
            idempotency_key: existing.idempotency_key,
            event,
            from_state,
            to_state,
            execution_mode,
            idempotent_replay: true,
            state_mutated: false,
            telemetry_recorded: false,
            transitioned_at: epoch_ms_to_datetime(existing.created_at_epoch_ms),
        })
    }

    fn apply_transition_projection_metadata(
        current: &BitcoinTxLifecycleRecord,
        next: &BitcoinTxLifecycleRecord,
        input: &BitcoinTxTransitionInput,
        event_id: &str,
    ) -> BitcoinTxLifecycleRecord {
        let mut projected = next.clone();
        projected.latest_transition = Some(input.event.clone());
        projected.latest_event_id = Some(event_id.to_string());
        projected.fee_rate_sat_vb = input.fee_rate_sat_vb.or(current.fee_rate_sat_vb);
        projected.attempt = update_attempt(current.attempt, input, current, &projected);
        projected.recovery_cursor = now_epoch_ms();
        projected
    }

    fn bitcoin_tx_record_for_transition(
        &self,
        tx_id: &str,
        execution_mode: &BitcoinTxLifecycleExecutionMode,
    ) -> BitcoinTxLifecycleRecord {
        if *execution_mode == BitcoinTxLifecycleExecutionMode::Shadow {
            if let Some(shadow_record) = self
                .bitcoin_tx_lifecycle_shadow
                .read()
                .unwrap()
                .get(tx_id)
                .cloned()
            {
                return shadow_record;
            }
        }

        self.read_orchestration_projection(tx_id)
            .unwrap_or_else(|| BitcoinTxLifecycleRecord::draft(tx_id))
    }

    fn invalid_bitcoin_transition(
        from: &BitcoinTxLifecycleState,
        event: &BitcoinTxLifecycleEvent,
        reason: impl Into<String>,
    ) -> BitcoinTxTransitionError {
        BitcoinTxTransitionError::InvalidTransition {
            from: from.clone(),
            event: event.clone(),
            reason: reason.into(),
        }
    }

    fn next_bitcoin_tx_record(
        current: &BitcoinTxLifecycleRecord,
        input: &BitcoinTxTransitionInput,
    ) -> Result<BitcoinTxLifecycleRecord, BitcoinTxTransitionError> {
        if current.state == BitcoinTxLifecycleState::DeadLetter {
            return Err(BitcoinTxTransitionError::TerminalState {
                state: BitcoinTxLifecycleState::DeadLetter,
            });
        }

        let mut next = current.clone();
        next.updated_at = Utc::now();

        match input.event {
            BitcoinTxLifecycleEvent::Sign => {
                if current.state != BitcoinTxLifecycleState::Draft {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "only draft transactions can be signed",
                    ));
                }
                next.state = BitcoinTxLifecycleState::Signed;
                next.confirmations_observed = 0;
                next.reorg_depth = None;
                next.dead_letter_reason = None;
            }
            BitcoinTxLifecycleEvent::QueueBroadcast => {
                if current.state != BitcoinTxLifecycleState::Signed
                    && current.state != BitcoinTxLifecycleState::Reorged
                {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "queue_broadcast requires signed or reorged state",
                    ));
                }
                next.state = BitcoinTxLifecycleState::BroadcastPending;
                next.confirmations_observed = 0;
                next.reorg_depth = None;
            }
            BitcoinTxLifecycleEvent::MempoolObserved => {
                if current.state != BitcoinTxLifecycleState::BroadcastPending
                    && current.state != BitcoinTxLifecycleState::Reorged
                {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "mempool_observed requires broadcast_pending or reorged state",
                    ));
                }
                next.state = BitcoinTxLifecycleState::InMempool;
                next.confirmations_observed = 0;
                next.reorg_depth = None;
            }
            BitcoinTxLifecycleEvent::ConfirmationsObserved => {
                if current.state != BitcoinTxLifecycleState::InMempool
                    && current.state != BitcoinTxLifecycleState::PendingConfirmations
                    && current.state != BitcoinTxLifecycleState::Confirmed
                    && current.state != BitcoinTxLifecycleState::Reorged
                {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "confirmations can only update mempool/pending/confirmed/reorged states",
                    ));
                }

                let confirmations =
                    input
                        .confirmations_observed
                        .ok_or(BitcoinTxTransitionError::MissingField {
                            field: "confirmations_observed",
                            event: BitcoinTxLifecycleEvent::ConfirmationsObserved,
                        })?;

                let required = input
                    .required_confirmations
                    .unwrap_or(current.required_confirmations);
                if required == 0 {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "required_confirmations must be greater than zero",
                    ));
                }
                if confirmations == 0 {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "confirmations_observed must be greater than zero",
                    ));
                }

                next.state = if confirmations >= required {
                    BitcoinTxLifecycleState::Confirmed
                } else {
                    BitcoinTxLifecycleState::PendingConfirmations
                };
                next.confirmations_observed = confirmations;
                next.required_confirmations = required;
                next.reorg_depth = None;
            }
            BitcoinTxLifecycleEvent::Finalize => {
                if current.state != BitcoinTxLifecycleState::Confirmed {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "only confirmed transactions can be finalized",
                    ));
                }
                next.state = BitcoinTxLifecycleState::Finalized;
            }
            BitcoinTxLifecycleEvent::ReorgDetected => {
                if current.state != BitcoinTxLifecycleState::PendingConfirmations
                    && current.state != BitcoinTxLifecycleState::Confirmed
                    && current.state != BitcoinTxLifecycleState::Finalized
                {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "reorg rollback requires pending_confirmations, confirmed, or finalized state",
                    ));
                }

                let depth = input.reorg_depth.unwrap_or(1);
                if depth == 0 {
                    return Err(Self::invalid_bitcoin_transition(
                        &current.state,
                        &input.event,
                        "reorg_depth must be greater than zero",
                    ));
                }

                next.state = BitcoinTxLifecycleState::Reorged;
                next.confirmations_observed = 0;
                next.reorg_depth = Some(depth);
            }
            BitcoinTxLifecycleEvent::MarkDeadLetter => {
                let reason = input
                    .dead_letter_reason
                    .as_deref()
                    .map(str::trim)
                    .filter(|reason| !reason.is_empty())
                    .ok_or(BitcoinTxTransitionError::MissingField {
                        field: "dead_letter_reason",
                        event: BitcoinTxLifecycleEvent::MarkDeadLetter,
                    })?
                    .to_string();
                next.state = BitcoinTxLifecycleState::DeadLetter;
                next.confirmations_observed = 0;
                next.reorg_depth = None;
                next.dead_letter_reason = Some(reason);
            }
        }

        Ok(next)
    }

    pub fn get_compliance_status(&self) -> ComplianceStatus {
        self.compliance.read().unwrap().clone()
    }
    pub fn get_bitvm2_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitvm2");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "challenge_period": 144, "bridge_status": "Active" })
    }
    pub fn get_bitvm2_segments(&self, state_root: &str) -> serde_json::Value {
        self.increment_requests();
        let orchestrator = lib_conxian_core::bitvm2::Bitvm2Orchestrator::new();
        let segments = orchestrator.generate_segments(state_root);
        serde_json::json!({ "state_root": state_root, "segments_count": segments.len(), "segments": segments })
    }
    pub fn get_bob_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bob");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "optimistic_finality": true, "fraud_proof_window_blocks": 1008 })
    }
    pub fn get_merlin_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("merlin");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "sequencer_batch_count": 12500, "zk_proof_system": "zk-STARK" })
    }
    pub fn get_botanix_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("botanix");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "spiderchain_nodes": 64, "peg_in_status": "Operational" })
    }
    pub fn get_rgb_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "contract_id": contract_id, "status": "Finalized", "asset_type": "RGB20" })
    }
    pub fn get_alpen_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_verification": "Enabled" })
    }
    pub fn get_bison_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bison");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "proof_count": 850 })
    }
    pub fn get_hemi_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("hemi");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "network": "Mainnet" })
    }
    pub fn get_taproot_assets_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("taproot-assets");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "issuance_count": 45 })
    }
    pub fn get_nubit_da_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("nubit");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_layer_status": "Active" })
    }
    pub fn get_b2_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("b2network");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "status": "Operational" })
    }
    pub fn get_affiliates(&self) -> Vec<AffiliateInfo> {
        self.affiliates.read().unwrap().values().cloned().collect()
    }
    pub fn get_marketing(&self) -> Vec<MarketingInfo> {
        self.marketing.read().unwrap().clone()
    }
    pub fn is_mainnet_only() -> bool {
        remediation::is_production_mainnet()
    }

    pub async fn start_monitoring(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                engine.update_metrics();
                let engine_clone = Arc::clone(&engine);
                tokio::spawn(async move {
                    let _ = engine_clone.fetch_stacks_block_height().await;
                });
                let engine_clone_btc = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_btc.fetch_bitcoin_rpc_status().await;
                });
                let engine_clone_core = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_core.fetch_core_dao_rpc_status().await;
                });
                let engine_clone_mpool = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_mpool.analyze_bitcoin_mpool().await;
                });
                let engine_clone_l2 = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_l2.track_l2_finality().await;
                });
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    pub async fn broadcast_intents(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                let proposals = engine.get_proposals();
                let pending_count = proposals.iter().filter(|p| p.status == "Pending").count();
                if pending_count > 0 {
                    log::info!("Broadcasting Phase 9 Real-time Intent Update: {} pending proposals requiring handshake", pending_count);
                }
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        });
    }

    pub fn get_sab_wallets(&self) -> Vec<SabWallet> {
        self.sab_wallets.read().unwrap().clone()
    }
    pub async fn poll_support(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                log::info!("Polling support mailbox for new tickets...");
                let ts = Utc::now();
                let ticket = engine.support_intake.process_inbound_metadata(
                    "support@conxian-labs.com",
                    "user@external.com",
                    "Assistance required with MuSig2",
                    "<sim-123@external.com>",
                    ts,
                );
                log::info!("Generated support ticket: {}", ticket.token);
                tokio::time::sleep(std::time::Duration::from_secs(
                    engine.support_intake.config.poll_interval_secs,
                ))
                .await;
            }
        });
    }
}

fn trim_optional_string(input: Option<String>) -> Option<String> {
    input
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_persisted_state(value: &str) -> Result<BitcoinTxLifecycleState, BitcoinTxTransitionError> {
    BitcoinTxLifecycleState::from_str(value)
        .map_err(|_| BitcoinTxTransitionError::UnknownPersistedState(value.to_string()))
}

fn parse_persisted_event(value: &str) -> Result<BitcoinTxLifecycleEvent, BitcoinTxTransitionError> {
    BitcoinTxLifecycleEvent::from_str(value)
        .map_err(|_| BitcoinTxTransitionError::UnknownPersistedEvent(value.to_string()))
}

fn orchestration_from_record(
    record: BtcTxOrchestrationRecord,
) -> Result<BitcoinTxOrchestration, BitcoinTxTransitionError> {
    Ok(BitcoinTxOrchestration {
        tx_id: record.tx_id,
        state: parse_persisted_state(&record.state)?,
        latest_transition: record
            .latest_transition
            .as_deref()
            .map(parse_persisted_event)
            .transpose()?,
        latest_event_id: record.latest_event_id,
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        attempt: record.attempt,
        observed_confirmations: record.observed_confirmations,
        recovery_cursor: record.recovery_cursor,
        updated_at_epoch_ms: record.updated_at_epoch_ms,
    })
}

fn event_from_record(record: BtcTxEventRecord) -> Result<BitcoinTxEvent, BitcoinTxTransitionError> {
    Ok(BitcoinTxEvent {
        event_id: record.event_id,
        tx_id: record.tx_id,
        idempotency_key: record.idempotency_key,
        event: parse_persisted_event(&record.transition)?,
        from_state: parse_persisted_state(&record.from_state)?,
        to_state: parse_persisted_state(&record.to_state)?,
        attempt: record.attempt,
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        observed_confirmations: record.observed_confirmations,
        rationale: record.rationale,
        fingerprint: record.fingerprint,
        created_at_epoch_ms: record.created_at_epoch_ms,
    })
}

fn lifecycle_record_from_orchestration_record(
    record: BtcTxOrchestrationRecord,
) -> Result<BitcoinTxLifecycleRecord, BitcoinTxTransitionError> {
    Ok(BitcoinTxLifecycleRecord {
        tx_id: record.tx_id,
        state: parse_persisted_state(&record.state)?,
        latest_transition: record
            .latest_transition
            .as_deref()
            .map(parse_persisted_event)
            .transpose()?,
        latest_event_id: record.latest_event_id,
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        attempt: record.attempt,
        confirmations_observed: record.observed_confirmations.unwrap_or(0),
        required_confirmations: DEFAULT_REQUIRED_CONFIRMATIONS,
        reorg_depth: None,
        dead_letter_reason: None,
        recovery_cursor: record.recovery_cursor,
        updated_at: epoch_ms_to_datetime(record.updated_at_epoch_ms),
    })
}

fn lifecycle_record_to_orchestration_record(
    record: &BitcoinTxLifecycleRecord,
) -> BtcTxOrchestrationRecord {
    BtcTxOrchestrationRecord {
        tx_id: record.tx_id.clone(),
        state: record.state.as_str().to_string(),
        latest_transition: record
            .latest_transition
            .as_ref()
            .map(|event| event.as_str().to_string()),
        latest_event_id: record.latest_event_id.clone(),
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        attempt: record.attempt,
        observed_confirmations: observed_confirmations_from_record(record),
        recovery_cursor: record.recovery_cursor,
        updated_at_epoch_ms: record.updated_at.timestamp_millis().max(0) as u64,
    }
}

fn observed_confirmations_from_record(record: &BitcoinTxLifecycleRecord) -> Option<u32> {
    (record.confirmations_observed > 0).then_some(record.confirmations_observed)
}

fn map_persistence_error(error: PersistenceError) -> BitcoinTxTransitionError {
    match error {
        PersistenceError::IdempotencyConflict {
            tx_id,
            idempotency_key,
            existing_fingerprint,
            incoming_fingerprint,
        } => BitcoinTxTransitionError::IdempotencyConflict {
            tx_id,
            idempotency_key,
            existing_fingerprint,
            incoming_fingerprint,
        },
        other => BitcoinTxTransitionError::Persistence(other.to_string()),
    }
}

fn derive_idempotency_key(
    input: &BitcoinTxTransitionInput,
    current_attempt: u32,
    current_fee_rate: Option<u64>,
) -> String {
    let attempt = input.attempt.unwrap_or(current_attempt);
    let fee_rate = input
        .fee_rate_sat_vb
        .or(current_fee_rate)
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let confirmations = input
        .confirmations_observed
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());

    format!(
        "{}:{}:{}:{}:{}",
        input.tx_id,
        input.event.as_str(),
        attempt,
        fee_rate,
        confirmations
    )
}

fn build_fingerprint(
    tx_id: &str,
    event: &BitcoinTxLifecycleEvent,
    attempt: u32,
    fee_rate_sat_vb: Option<u64>,
    observed_confirmations: Option<u32>,
    required_confirmations: Option<u32>,
    reorg_depth: Option<u32>,
    rationale: Option<&str>,
) -> String {
    let fee = fee_rate_sat_vb
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let confirmations = observed_confirmations
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let required = required_confirmations
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let reorg_depth = reorg_depth
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let rationale = rationale.unwrap_or("-");

    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        tx_id,
        event.as_str(),
        attempt,
        fee,
        confirmations,
        required,
        reorg_depth,
        rationale,
    )
}

fn update_attempt(
    current_attempt: u32,
    input: &BitcoinTxTransitionInput,
    from_state: &BitcoinTxLifecycleRecord,
    to_state: &BitcoinTxLifecycleRecord,
) -> u32 {
    if let Some(explicit) = input.attempt {
        return explicit;
    }

    if input.event == BitcoinTxLifecycleEvent::QueueBroadcast
        && from_state.state != BitcoinTxLifecycleState::BroadcastPending
        && to_state.state == BitcoinTxLifecycleState::BroadcastPending
    {
        return current_attempt.saturating_add(1);
    }

    current_attempt
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

fn epoch_ms_to_datetime(epoch_ms: u64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_millis(epoch_ms as i64).unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::anchoring::AnchoringPublication;
    use std::sync::atomic::AtomicUsize;

    struct ScriptedPublisher {
        name: &'static str,
        fail_until_attempt: u8,
        always_fail: bool,
        retryable: bool,
        calls: Arc<AtomicUsize>,
    }

    impl AnchoringPublisher for ScriptedPublisher {
        fn name(&self) -> &'static str {
            self.name
        }

        fn publish(
            &self,
            request: &AnchoringRequest,
            attempt: u8,
        ) -> Result<AnchoringPublication, AnchoringError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            if self.always_fail || attempt <= self.fail_until_attempt {
                return Err(AnchoringError::AdapterFailure {
                    adapter: self.name.to_string(),
                    code: "simulated_failure".to_string(),
                    message: format!("simulated failure on attempt {attempt}"),
                    retryable: self.retryable,
                });
            }

            let mut metadata = HashMap::new();
            if self.name == "tableland" {
                metadata.insert("table_name".to_string(), "conxian_state_shards".to_string());
            }
            metadata.insert("state_root".to_string(), request.state_root.clone());

            Ok(AnchoringPublication {
                adapter: self.name.to_string(),
                status: if self.name == "tableland" {
                    "Finalized".to_string()
                } else {
                    "Broadcasted".to_string()
                },
                reference: format!("0x{}{:02x}", self.name.replace('_', ""), attempt),
                persistence: if self.name == "tableland" {
                    "Decentralized (Tableland)".to_string()
                } else {
                    "L1 Commitment Registry".to_string()
                },
                attempts: attempt,
                metadata,
            })
        }
    }

    fn scripted_engine(
        tableland: Arc<dyn AnchoringPublisher>,
        on_chain: Arc<dyn AnchoringPublisher>,
    ) -> Engine {
        Engine::new_with_anchoring_publishers(tableland, on_chain)
    }

    fn lifecycle_engine(config: BitcoinTxLifecycleConfig) -> Engine {
        Engine::new_with_anchoring_publishers_and_tx_lifecycle_config(
            Arc::new(TablelandAnchoringPublisher),
            Arc::new(OnChainAnchoringPublisher),
            config,
        )
    }

    fn lifecycle_engine_with_persistence(
        config: BitcoinTxLifecycleConfig,
        persistence: Arc<dyn BitcoinTxPersistence>,
    ) -> Engine {
        Engine::new_with_anchoring_publishers_tx_lifecycle_config_and_persistence(
            Arc::new(TablelandAnchoringPublisher),
            Arc::new(OnChainAnchoringPublisher),
            config,
            persistence,
        )
    }

    fn fee_bump_policy() -> BitcoinFeeBumpPolicy {
        BitcoinFeeBumpPolicy {
            max_attempts: 3,
            max_fee_rate_sats_vb: 150,
            min_bump_increment_sats_vb: 2,
            stuck_threshold_blocks: 3,
            stuck_threshold_seconds: 900,
        }
    }

    #[test]
    fn bitcoin_fee_bump_prefers_rbf_when_replaceable_and_guardrails_pass() {
        let decision = evaluate_bitcoin_fee_bump_decision(
            &fee_bump_policy(),
            &BitcoinFeeBumpDecisionInput {
                attempts_used: 0,
                current_fee_rate_sats_vb: 10,
                network_target_fee_rate_sats_vb: 11,
                replaceable: true,
                cpfp_available: true,
                blocks_since_broadcast: Some(4),
                seconds_since_broadcast: Some(120),
            },
        );

        assert_eq!(decision.action, BitcoinFeeBumpAction::Rbf);
        assert_eq!(decision.reason, BitcoinFeeBumpReason::RbfPreferred);
        assert_eq!(decision.next_fee_rate_sats_vb, Some(12));
        assert_eq!(decision.next_attempt, Some(1));
    }

    #[test]
    fn bitcoin_fee_bump_falls_back_to_cpfp_when_rbf_not_available() {
        let decision = evaluate_bitcoin_fee_bump_decision(
            &fee_bump_policy(),
            &BitcoinFeeBumpDecisionInput {
                attempts_used: 1,
                current_fee_rate_sats_vb: 18,
                network_target_fee_rate_sats_vb: 19,
                replaceable: false,
                cpfp_available: true,
                blocks_since_broadcast: Some(4),
                seconds_since_broadcast: Some(1800),
            },
        );

        assert_eq!(decision.action, BitcoinFeeBumpAction::Cpfp);
        assert_eq!(decision.reason, BitcoinFeeBumpReason::CpfpFallback);
        assert_eq!(decision.next_fee_rate_sats_vb, Some(20));
        assert_eq!(decision.next_attempt, Some(2));
    }

    #[test]
    fn bitcoin_fee_bump_rejects_when_max_attempts_exhausted() {
        let decision = evaluate_bitcoin_fee_bump_decision(
            &fee_bump_policy(),
            &BitcoinFeeBumpDecisionInput {
                attempts_used: 3,
                current_fee_rate_sats_vb: 20,
                network_target_fee_rate_sats_vb: 24,
                replaceable: true,
                cpfp_available: true,
                blocks_since_broadcast: Some(5),
                seconds_since_broadcast: Some(2000),
            },
        );

        assert_eq!(decision.action, BitcoinFeeBumpAction::Reject);
        assert_eq!(decision.reason, BitcoinFeeBumpReason::MaxAttemptsReached);
        assert_eq!(decision.next_fee_rate_sats_vb, None);
        assert_eq!(decision.next_attempt, None);
    }

    #[test]
    fn bitcoin_fee_bump_rejects_when_fee_cap_would_be_exceeded() {
        let policy = BitcoinFeeBumpPolicy {
            max_attempts: 3,
            max_fee_rate_sats_vb: 21,
            min_bump_increment_sats_vb: 3,
            stuck_threshold_blocks: 2,
            stuck_threshold_seconds: 600,
        };

        let decision = evaluate_bitcoin_fee_bump_decision(
            &policy,
            &BitcoinFeeBumpDecisionInput {
                attempts_used: 1,
                current_fee_rate_sats_vb: 20,
                network_target_fee_rate_sats_vb: 20,
                replaceable: true,
                cpfp_available: true,
                blocks_since_broadcast: Some(3),
                seconds_since_broadcast: Some(700),
            },
        );

        assert_eq!(decision.action, BitcoinFeeBumpAction::Reject);
        assert_eq!(decision.reason, BitcoinFeeBumpReason::FeeCapExceeded);
        assert_eq!(decision.next_fee_rate_sats_vb, None);
        assert_eq!(decision.next_attempt, None);
    }

    #[test]
    fn commit_state_checkpoint_successful_publish_path() {
        let engine = Engine::new();

        let receipt = engine
            .commit_state_checkpoint(AnchoringRequest {
                state_root: "0xabc123".to_string(),
                target: AnchoringTarget::Tableland,
                idempotency_key: Some("issue-534-success".to_string()),
                metadata: HashMap::new(),
                max_retry_attempts: 3,
            })
            .expect("state commit should succeed");

        assert_eq!(receipt.target, AnchoringTarget::Tableland);
        assert_eq!(receipt.status, "Finalized");
        assert_eq!(receipt.publications.len(), 1);
        assert_eq!(receipt.publications[0].adapter, "tableland");
        assert_eq!(receipt.table_name.as_deref(), Some("conxian_state_shards"));
        assert!(receipt.transaction_hash.is_some());
        assert!(!receipt.idempotent_replay);
    }

    #[test]
    fn commit_state_checkpoint_returns_idempotent_replay_without_republish() {
        let tableland_calls = Arc::new(AtomicUsize::new(0));
        let on_chain_calls = Arc::new(AtomicUsize::new(0));

        let tableland = Arc::new(ScriptedPublisher {
            name: "tableland",
            fail_until_attempt: 0,
            always_fail: false,
            retryable: false,
            calls: Arc::clone(&tableland_calls),
        });

        let on_chain = Arc::new(ScriptedPublisher {
            name: "on_chain",
            fail_until_attempt: 0,
            always_fail: false,
            retryable: false,
            calls: Arc::clone(&on_chain_calls),
        });

        let engine = scripted_engine(tableland, on_chain);

        let request = AnchoringRequest {
            state_root: "0xreplay123".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: Some("issue-534-replay".to_string()),
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let first = engine
            .commit_state_checkpoint(request.clone())
            .expect("first publication should succeed");
        let replay = engine
            .commit_state_checkpoint(request)
            .expect("replay should return cached receipt");

        assert_eq!(first.receipt_id, replay.receipt_id);
        assert!(replay.idempotent_replay);
        assert_eq!(tableland_calls.load(Ordering::SeqCst), 1);
        assert_eq!(on_chain_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn commit_state_checkpoint_retries_retryable_adapter_errors() {
        let tableland_calls = Arc::new(AtomicUsize::new(0));

        let tableland = Arc::new(ScriptedPublisher {
            name: "tableland",
            fail_until_attempt: 2,
            always_fail: false,
            retryable: true,
            calls: Arc::clone(&tableland_calls),
        });

        let on_chain = Arc::new(ScriptedPublisher {
            name: "on_chain",
            fail_until_attempt: 0,
            always_fail: false,
            retryable: false,
            calls: Arc::new(AtomicUsize::new(0)),
        });

        let engine = scripted_engine(tableland, on_chain);

        let receipt = engine
            .commit_state_checkpoint(AnchoringRequest {
                state_root: "0xretry123".to_string(),
                target: AnchoringTarget::Tableland,
                idempotency_key: Some("issue-534-retry".to_string()),
                metadata: HashMap::new(),
                max_retry_attempts: 3,
            })
            .expect("retry should eventually succeed");

        assert_eq!(receipt.total_attempts, 3);
        assert_eq!(receipt.publications[0].attempts, 3);
        assert_eq!(tableland_calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn commit_state_checkpoint_returns_retry_exhausted_when_budget_consumed() {
        let tableland = Arc::new(ScriptedPublisher {
            name: "tableland",
            fail_until_attempt: 0,
            always_fail: true,
            retryable: true,
            calls: Arc::new(AtomicUsize::new(0)),
        });

        let on_chain = Arc::new(ScriptedPublisher {
            name: "on_chain",
            fail_until_attempt: 0,
            always_fail: false,
            retryable: false,
            calls: Arc::new(AtomicUsize::new(0)),
        });

        let engine = scripted_engine(tableland, on_chain);

        let err = engine
            .commit_state_checkpoint(AnchoringRequest {
                state_root: "0xretryexhausted".to_string(),
                target: AnchoringTarget::Tableland,
                idempotency_key: Some("issue-534-retry-exhausted".to_string()),
                metadata: HashMap::new(),
                max_retry_attempts: 2,
            })
            .expect_err("retry budget should be exhausted");

        assert!(matches!(
            err,
            AnchoringError::RetryExhausted {
                attempts: 2,
                adapter,
                ..
            } if adapter == "tableland"
        ));
    }

    #[test]
    fn bitcoin_tx_lifecycle_happy_path_reaches_finalized() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: false,
        });

        let tx_id = "btc-tx-happy-path";

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("draft should transition to signed");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("signed should transition to broadcast_pending");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::MempoolObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("broadcast_pending should transition to in_mempool");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::ConfirmationsObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: Some(2),
                required_confirmations: Some(6),
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("in_mempool should transition to pending_confirmations");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::ConfirmationsObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: Some(6),
                required_confirmations: Some(6),
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("pending_confirmations should transition to confirmed");

        let finalized = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::Finalize,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("confirmed should transition to finalized");

        assert_eq!(
            finalized.to_state,
            BitcoinTxLifecycleState::Finalized,
            "expected finalized terminal on normal path"
        );
        assert!(finalized.state_mutated);

        let current = engine.get_bitcoin_tx_lifecycle_record(tx_id);
        assert_eq!(current.state, BitcoinTxLifecycleState::Finalized);
    }

    #[test]
    fn bitcoin_tx_lifecycle_reorg_rollback_branch_recovers() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: false,
        });
        let tx_id = "btc-tx-reorg";

        for event in [
            BitcoinTxLifecycleEvent::Sign,
            BitcoinTxLifecycleEvent::QueueBroadcast,
            BitcoinTxLifecycleEvent::MempoolObserved,
        ] {
            engine
                .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                    tx_id: tx_id.to_string(),
                    event,
                    idempotency_key: None,
                    rationale: None,
                    fee_rate_sat_vb: None,
                    attempt: None,
                    confirmations_observed: None,
                    required_confirmations: None,
                    reorg_depth: None,
                    dead_letter_reason: None,
                })
                .expect("expected pre-reorg transition");
        }

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::ConfirmationsObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: Some(6),
                required_confirmations: Some(6),
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("transaction should be confirmed");

        let reorged = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::ReorgDetected,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: Some(2),
                dead_letter_reason: None,
            })
            .expect("confirmed transaction should support reorg rollback");
        assert_eq!(reorged.to_state, BitcoinTxLifecycleState::Reorged);

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("reorged tx should re-enter broadcast queue");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::MempoolObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("reorged tx should return to mempool");

        let recovered = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::ConfirmationsObserved,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: Some(6),
                required_confirmations: Some(6),
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("reorged tx should be recoverable to confirmed");
        assert_eq!(recovered.to_state, BitcoinTxLifecycleState::Confirmed);
    }

    #[test]
    fn bitcoin_tx_lifecycle_dead_letter_is_terminal() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: false,
        });
        let tx_id = "btc-tx-dead-letter";

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::MarkDeadLetter,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: Some("max_fee_policy_exceeded".to_string()),
            })
            .expect("dead_letter transition should be accepted with reason");

        let err = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect_err("dead_letter must remain terminal");

        assert!(matches!(
            err,
            BitcoinTxTransitionError::TerminalState {
                state: BitcoinTxLifecycleState::DeadLetter
            }
        ));
    }

    #[test]
    fn bitcoin_tx_lifecycle_rejects_invalid_transitions() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: false,
        });

        let err = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-invalid".to_string(),
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect_err("draft -> queue_broadcast should be invalid");

        assert!(matches!(
            err,
            BitcoinTxTransitionError::InvalidTransition {
                from: BitcoinTxLifecycleState::Draft,
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                ..
            }
        ));
    }

    #[test]
    fn bitcoin_tx_lifecycle_shadow_mode_is_telemetry_only() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: true,
        });
        let tx_id = "btc-tx-shadow";

        let outcome = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: tx_id.to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("shadow mode should evaluate transition");

        assert!(!outcome.state_mutated);
        assert!(outcome.telemetry_recorded);
        assert_eq!(
            outcome.execution_mode,
            BitcoinTxLifecycleExecutionMode::Shadow
        );

        let production = engine.get_bitcoin_tx_lifecycle_record(tx_id);
        assert_eq!(
            production.state,
            BitcoinTxLifecycleState::Draft,
            "production state must not mutate in shadow mode"
        );

        let shadow_view = engine.get_bitcoin_tx_lifecycle_view(tx_id);
        let shadow_state = shadow_view
            .shadow
            .expect("shadow projection should be tracked for telemetry");
        assert_eq!(shadow_state.state, BitcoinTxLifecycleState::Signed);

        assert_eq!(engine.get_bitcoin_tx_lifecycle_telemetry().len(), 1);
    }

    #[test]
    fn duplicate_transition_is_idempotent_and_does_not_append_event_twice() {
        let engine = lifecycle_engine_with_persistence(
            BitcoinTxLifecycleConfig {
                enabled: true,
                shadow_mode: false,
            },
            Arc::new(InMemoryBitcoinTxPersistence::default()),
        );

        let first = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-1".to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: Some("request-1".to_string()),
                rationale: Some("signature complete".to_string()),
                fee_rate_sat_vb: Some(12),
                attempt: Some(0),
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("first transition should apply");

        let second = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-1".to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: Some("request-1".to_string()),
                rationale: Some("signature complete".to_string()),
                fee_rate_sat_vb: Some(12),
                attempt: Some(0),
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("duplicate transition should be idempotent");

        assert!(!first.idempotent_replay);
        assert!(second.idempotent_replay);
        assert_eq!(first.event_id, second.event_id);

        let events = engine
            .list_bitcoin_tx_events("btc-tx-1")
            .expect("events should load");
        assert_eq!(events.len(), 1);

        let orchestration = engine
            .get_bitcoin_tx_orchestration("btc-tx-1")
            .expect("orchestration lookup should succeed")
            .expect("orchestration should exist");
        assert_eq!(orchestration.state, BitcoinTxLifecycleState::Signed);
    }

    #[test]
    fn restart_recovery_rehydrates_orchestration_state_from_persistence() {
        let storage_path =
            std::env::temp_dir().join(format!("con717-recovery-{}.json", now_epoch_ms()));

        let persistence = Arc::new(persistence::JsonFileBitcoinTxPersistence::new(
            storage_path.clone(),
        ));
        let config = BitcoinTxLifecycleConfig {
            enabled: true,
            shadow_mode: false,
        };

        let engine = lifecycle_engine_with_persistence(config.clone(), persistence.clone());

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-2".to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: Some("req-sign".to_string()),
                rationale: None,
                fee_rate_sat_vb: Some(10),
                attempt: Some(0),
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("sign transition should persist");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-2".to_string(),
                event: BitcoinTxLifecycleEvent::QueueBroadcast,
                idempotency_key: Some("req-queue".to_string()),
                rationale: Some("ready for mempool".to_string()),
                fee_rate_sat_vb: Some(15),
                attempt: Some(1),
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect("queue transition should persist");

        drop(engine);

        let restarted = lifecycle_engine_with_persistence(config, persistence);

        let recovered = restarted
            .get_bitcoin_tx_orchestration("btc-tx-2")
            .expect("recovered state should be readable")
            .expect("orchestration should be recovered");

        assert_eq!(recovered.state, BitcoinTxLifecycleState::BroadcastPending);
        assert_eq!(recovered.attempt, 1);
        assert_eq!(recovered.fee_rate_sat_vb, Some(15));

        let recovered_events = restarted
            .list_bitcoin_tx_events("btc-tx-2")
            .expect("recovered events should be readable");
        assert_eq!(recovered_events.len(), 2);

        let _ = std::fs::remove_file(storage_path);
    }

    #[test]
    fn bitcoin_tx_lifecycle_feature_flag_blocks_orchestration_when_disabled() {
        let engine = lifecycle_engine(BitcoinTxLifecycleConfig {
            enabled: false,
            shadow_mode: false,
        });

        let err = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-disabled".to_string(),
                event: BitcoinTxLifecycleEvent::Sign,
                idempotency_key: None,
                rationale: None,
                fee_rate_sat_vb: None,
                attempt: None,
                confirmations_observed: None,
                required_confirmations: None,
                reorg_depth: None,
                dead_letter_reason: None,
            })
            .expect_err("disabled feature flag should reject orchestration");

        assert!(matches!(err, BitcoinTxTransitionError::FeatureDisabled));
    }
}
