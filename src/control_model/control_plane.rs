use serde::{Deserialize, Serialize};

// --- Control Plane Modules (CON-773) ---

/// Release governance models for tracking and approving protocol releases.
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

/// Audit models for visibility into internal operations and protocol events.
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

/// Policy approval models for managing change control.
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

/// Environment and Config Registry models for private operational use.
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

#[cfg(test)]
mod control_plane_tests {
    use super::*;

    #[test]
    fn test_release_status_transitions() {
        assert!(
            validate_release_transition(&ReleaseStatus::Proposed, &ReleaseStatus::InReview).is_ok()
        );
        assert!(
            validate_release_transition(&ReleaseStatus::Released, &ReleaseStatus::Deprecated)
                .is_ok()
        );
        assert!(
            validate_release_transition(&ReleaseStatus::Released, &ReleaseStatus::Approved)
                .is_err()
        );
    }

    #[test]
    fn test_policy_approval_transitions() {
        assert!(validate_policy_approval_transition(
            &PolicyApprovalStatus::Pending,
            &PolicyApprovalStatus::Approved
        )
        .is_ok());
        assert!(validate_policy_approval_transition(
            &PolicyApprovalStatus::Approved,
            &PolicyApprovalStatus::Implemented
        )
        .is_ok());
        assert!(validate_policy_approval_transition(
            &PolicyApprovalStatus::Implemented,
            &PolicyApprovalStatus::Pending
        )
        .is_err());
    }

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::High);
        assert!(AuditSeverity::High > AuditSeverity::Medium);
        assert!(AuditSeverity::Medium > AuditSeverity::Low);
    }
}
