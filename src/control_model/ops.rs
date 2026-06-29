use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub release_track: ReleaseTrack,
    pub status: ReleaseStatus,
    pub changelog_uri: String,
    pub commit_hash: String,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReleaseTrack {
    Stable,
    Beta,
    Alpha,
    Nightly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReleaseStatus {
    Proposed,
    InReview,
    Approved,
    Rejected,
    Released,
    Deprecated,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEvent {
    pub event_id: String,
    pub category: AuditCategory,
    pub severity: AuditSeverity,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditCategory {
    Security,
    Governance,
    Operations,
    Financial,
    Technical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyChangeRequest {
    pub request_id: String,
    pub policy_id: String,
    pub requester: String,
    pub change_description: String,
    pub status: PolicyApprovalStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentRegistryEntry {
    pub name: String,
    pub env_type: EnvironmentType,
    pub base_url: String,
    pub status: String,
    pub last_sync: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvironmentType {
    Production,
    Staging,
    Development,
    Ephemeral,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigRegistryEntry {
    pub key: String,
    pub value_hash: String,
    pub is_secret: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub updated_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinancialMetrics {
    pub mrr_usd: f64,
    pub arr_usd: f64,
    pub churn_rate_pct: f64,
    pub protocol_fees_collected_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

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

pub fn validate_release_transition(from: &ReleaseStatus, to: &ReleaseStatus) -> Result<(), String> {
    let valid = matches!(
        (from, to),
        (ReleaseStatus::Proposed, ReleaseStatus::InReview)
            | (ReleaseStatus::Proposed, ReleaseStatus::Revoked)
            | (ReleaseStatus::InReview, ReleaseStatus::Approved)
            | (ReleaseStatus::InReview, ReleaseStatus::Rejected)
            | (ReleaseStatus::InReview, ReleaseStatus::Revoked)
            | (ReleaseStatus::Approved, ReleaseStatus::Released)
            | (ReleaseStatus::Approved, ReleaseStatus::Revoked)
            | (ReleaseStatus::Released, ReleaseStatus::Deprecated)
            | (ReleaseStatus::Released, ReleaseStatus::Revoked)
    );

    if valid {
        Ok(())
    } else {
        Err(format!(
            "Invalid release transition: {:?} -> {:?}",
            from, to
        ))
    }
}

pub fn validate_policy_approval_transition(
    from: &PolicyApprovalStatus,
    to: &PolicyApprovalStatus,
) -> Result<(), String> {
    let valid = matches!(
        (from, to),
        (
            PolicyApprovalStatus::Pending,
            PolicyApprovalStatus::Approved
        ) | (
            PolicyApprovalStatus::Pending,
            PolicyApprovalStatus::Rejected
        ) | (
            PolicyApprovalStatus::Approved,
            PolicyApprovalStatus::Implemented
        )
    );

    if valid {
        Ok(())
    } else {
        Err(format!(
            "Invalid policy approval transition: {:?} -> {:?}",
            from, to
        ))
    }
}
