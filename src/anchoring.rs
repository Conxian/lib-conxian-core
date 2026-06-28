use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub const DEFAULT_MAX_RETRY_ATTEMPTS: u8 = 3;

fn default_max_retry_attempts() -> u8 {
    DEFAULT_MAX_RETRY_ATTEMPTS
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnchoringTarget {
    #[default]
    Tableland,
    OnChain,
    Both,
}

impl AnchoringTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnchoringTarget::Tableland => "tableland",
            AnchoringTarget::OnChain => "on_chain",
            AnchoringTarget::Both => "both",
        }
    }

    pub fn execution_paths(&self) -> &'static [&'static str] {
        match self {
            AnchoringTarget::Tableland => &["tableland"],
            AnchoringTarget::OnChain => &["on_chain"],
            AnchoringTarget::Both => &["tableland", "on_chain"],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchoringRequest {
    pub state_root: String,
    #[serde(default)]
    pub target: AnchoringTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default = "default_max_retry_attempts")]
    pub max_retry_attempts: u8,
}

impl AnchoringRequest {
    pub fn normalized(&self) -> Self {
        Self {
            state_root: self.state_root.trim().to_string(),
            target: self.target.clone(),
            idempotency_key: self
                .idempotency_key
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            metadata: self.metadata.clone(),
            max_retry_attempts: self.max_retry_attempts.max(1),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchoringPublication {
    pub adapter: String,
    pub status: String,
    pub reference: String,
    pub persistence: String,
    pub attempts: u8,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnchoringReceipt {
    pub receipt_id: String,
    pub state_root: String,
    pub target: AnchoringTarget,
    pub idempotency_key: String,
    pub idempotent_replay: bool,
    pub status: String,
    pub published_at: DateTime<Utc>,
    pub total_attempts: u8,
    pub publications: Vec<AnchoringPublication>,
    #[serde(default)]
    pub audit_metadata: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnchoringError {
    Validation {
        message: String,
    },
    IdempotencyConflict {
        idempotency_key: String,
        existing_fingerprint: String,
        incoming_fingerprint: String,
        existing_state_root: String,
        incoming_state_root: String,
    },
    AdapterFailure {
        adapter: String,
        code: String,
        message: String,
        retryable: bool,
    },
    RetryExhausted {
        adapter: String,
        attempts: u8,
        message: String,
    },
}

impl AnchoringError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AnchoringError::AdapterFailure {
                retryable: true,
                ..
            }
        )
    }

    pub fn code(&self) -> &'static str {
        match self {
            AnchoringError::Validation { .. } => "validation_error",
            AnchoringError::IdempotencyConflict { .. } => "idempotency_conflict",
            AnchoringError::AdapterFailure { .. } => "adapter_failure",
            AnchoringError::RetryExhausted { .. } => "retry_exhausted",
        }
    }
}

impl fmt::Display for AnchoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnchoringError::Validation { message } => write!(f, "validation error: {message}"),
            AnchoringError::IdempotencyConflict {
                idempotency_key, ..
            } => {
                write!(f, "idempotency conflict for key {idempotency_key}")
            }
            AnchoringError::AdapterFailure {
                adapter,
                code,
                message,
                ..
            } => write!(f, "adapter failure ({adapter}/{code}): {message}"),
            AnchoringError::RetryExhausted {
                adapter,
                attempts,
                message,
            } => {
                write!(
                    f,
                    "retry exhausted for {adapter} after {attempts} attempts: {message}"
                )
            }
        }
    }
}

impl std::error::Error for AnchoringError {}

pub trait AnchoringPublisher: Send + Sync {
    fn name(&self) -> &'static str;
    fn publish(
        &self,
        request: &AnchoringRequest,
        attempt: u8,
    ) -> Result<AnchoringPublication, AnchoringError>;
}

pub struct TablelandAnchoringPublisher;

impl AnchoringPublisher for TablelandAnchoringPublisher {
    fn name(&self) -> &'static str {
        "tableland"
    }

    fn publish(
        &self,
        request: &AnchoringRequest,
        attempt: u8,
    ) -> Result<AnchoringPublication, AnchoringError> {
        if request.state_root.trim().is_empty() {
            return Err(AnchoringError::Validation {
                message: "state_root is required".to_string(),
            });
        }

        let compact = compact_state_root(&request.state_root);
        let table_name = "conxian_state_shards";
        let tx = format!("0xtbl{compact}{attempt:02x}");

        let mut metadata = request.metadata.clone();
        metadata.insert("table_name".to_string(), table_name.to_string());
        metadata.insert(
            "commitment_type".to_string(),
            "checkpoint_state_root".to_string(),
        );

        Ok(AnchoringPublication {
            adapter: self.name().to_string(),
            status: "Finalized".to_string(),
            reference: tx,
            persistence: "Decentralized (Tableland)".to_string(),
            attempts: attempt,
            metadata,
        })
    }
}

pub struct OnChainAnchoringPublisher;

impl AnchoringPublisher for OnChainAnchoringPublisher {
    fn name(&self) -> &'static str {
        "on_chain"
    }

    fn publish(
        &self,
        request: &AnchoringRequest,
        attempt: u8,
    ) -> Result<AnchoringPublication, AnchoringError> {
        if request.state_root.trim().is_empty() {
            return Err(AnchoringError::Validation {
                message: "state_root is required".to_string(),
            });
        }

        let compact = compact_state_root(&request.state_root);
        let mut metadata = request.metadata.clone();
        metadata.insert(
            "chain_network".to_string(),
            metadata
                .get("chain_network")
                .cloned()
                .unwrap_or_else(|| "bitcoin-mainnet".to_string()),
        );
        metadata.insert(
            "commitment_contract".to_string(),
            metadata
                .get("commitment_contract")
                .cloned()
                .unwrap_or_else(|| "checkpoint-registry-v1".to_string()),
        );

        Ok(AnchoringPublication {
            adapter: self.name().to_string(),
            status: "Broadcasted".to_string(),
            reference: format!("0xonc{compact}{attempt:02x}"),
            persistence: "L1 Commitment Registry".to_string(),
            attempts: attempt,
            metadata,
        })
    }
}

fn compact_state_root(state_root: &str) -> String {
    let normalized = state_root.trim().trim_start_matches("0x");
    if normalized.len() <= 16 {
        return normalized.to_ascii_lowercase();
    }

    format!(
        "{}{}",
        &normalized[..8].to_ascii_lowercase(),
        &normalized[normalized.len() - 8..].to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ── AnchoringTarget ──────────────────────────────────────────

    #[test]
    fn test_anchoring_target_default() {
        assert_eq!(AnchoringTarget::default(), AnchoringTarget::Tableland);
    }

    #[test]
    fn test_anchoring_target_as_str() {
        assert_eq!(AnchoringTarget::Tableland.as_str(), "tableland");
        assert_eq!(AnchoringTarget::OnChain.as_str(), "on_chain");
        assert_eq!(AnchoringTarget::Both.as_str(), "both");
    }

    #[test]
    fn test_anchoring_target_execution_paths() {
        assert_eq!(AnchoringTarget::Tableland.execution_paths(), &["tableland"]);
        assert_eq!(AnchoringTarget::OnChain.execution_paths(), &["on_chain"]);
        assert_eq!(
            AnchoringTarget::Both.execution_paths(),
            &["tableland", "on_chain"]
        );
    }

    #[test]
    fn test_anchoring_target_serialization() {
        let json = serde_json::to_string(&AnchoringTarget::OnChain).unwrap();
        assert_eq!(json, r#""on_chain""#);

        let parsed: AnchoringTarget = serde_json::from_str(r#""both""#).unwrap();
        assert_eq!(parsed, AnchoringTarget::Both);

        // Unknown variants fail
        assert!(serde_json::from_str::<AnchoringTarget>(r#""unknown""#).is_err());
    }

    // ── AnchoringRequest ─────────────────────────────────────────

    #[test]
    fn test_anchoring_request_defaults() {
        let json = r#"{"state_root":"0xabcdef1234567890"}"#;
        let req: AnchoringRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.target, AnchoringTarget::Tableland); // default
        assert_eq!(req.idempotency_key, None);
        assert!(req.metadata.is_empty());
        assert_eq!(req.max_retry_attempts, 3);
    }

    #[test]
    fn test_anchoring_request_full_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert("chain".to_string(), "bitcoin".to_string());

        let req = AnchoringRequest {
            state_root: "0xabcdef".to_string(),
            target: AnchoringTarget::Both,
            idempotency_key: Some("idem-key-001".to_string()),
            metadata,
            max_retry_attempts: 5,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("0xabcdef"));
        assert!(json.contains("idem-key-001"));
        assert!(json.contains("both"));

        let roundtrip: AnchoringRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.state_root, "0xabcdef");
        assert_eq!(roundtrip.target, AnchoringTarget::Both);
        assert_eq!(roundtrip.idempotency_key, Some("idem-key-001".to_string()));
        assert_eq!(roundtrip.max_retry_attempts, 5);
    }

    #[test]
    fn test_anchoring_request_normalized_trims_whitespace() {
        let req = AnchoringRequest {
            state_root: "  0xabcdef  ".to_string(),
            target: AnchoringTarget::OnChain,
            idempotency_key: Some("  key-001  ".to_string()),
            metadata: HashMap::new(),
            max_retry_attempts: 0, // will clamp to 1
        };

        let n = req.normalized();
        assert_eq!(n.state_root, "0xabcdef");
        assert_eq!(n.idempotency_key, Some("key-001".to_string()));
        assert_eq!(n.max_retry_attempts, 1); // clamped from 0
    }

    #[test]
    fn test_anchoring_request_normalized_empty_key_becomes_none() {
        let req = AnchoringRequest {
            state_root: "hash".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: Some("   ".to_string()),
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let n = req.normalized();
        assert_eq!(n.idempotency_key, None, "whitespace-only key becomes None");
    }

    #[test]
    fn test_anchoring_request_idempotency_key_skip_serialize() {
        let req = AnchoringRequest {
            state_root: "hash".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("idempotency_key"), "None should be skipped");
    }

    // ── AnchoringPublication ─────────────────────────────────────

    #[test]
    fn test_anchoring_publication_serialization() {
        let pub_item = AnchoringPublication {
            adapter: "tableland".to_string(),
            status: "Finalized".to_string(),
            reference: "0xtblabcde03".to_string(),
            persistence: "Decentralized (Tableland)".to_string(),
            attempts: 1,
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&pub_item).unwrap();
        assert!(json.contains("Finalized"));
        assert!(json.contains("0xtblabcde03"));

        let roundtrip: AnchoringPublication = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.adapter, "tableland");
        assert_eq!(roundtrip.attempts, 1);
    }

    // ── AnchoringReceipt ─────────────────────────────────────────

    #[test]
    fn test_anchoring_receipt_serialization() {
        let receipt = AnchoringReceipt {
            receipt_id: "rcpt-001".to_string(),
            state_root: "0xabc".to_string(),
            target: AnchoringTarget::Both,
            idempotency_key: "idem-001".to_string(),
            idempotent_replay: false,
            status: "Published".to_string(),
            published_at: chrono::Utc::now(),
            total_attempts: 2,
            publications: vec![],
            audit_metadata: HashMap::new(),
            table_name: Some("conxian_state_shards".to_string()),
            transaction_hash: None,
            persistence: None,
        };

        let json = serde_json::to_string(&receipt).unwrap();
        assert!(json.contains("rcpt-001"));

        let roundtrip: AnchoringReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.receipt_id, "rcpt-001");
        assert_eq!(roundtrip.target, AnchoringTarget::Both);
        assert_eq!(roundtrip.total_attempts, 2);
    }

    #[test]
    fn test_anchoring_receipt_optional_fields_skipped() {
        let receipt = AnchoringReceipt {
            receipt_id: "rcpt-002".to_string(),
            state_root: "hash".to_string(),
            target: AnchoringTarget::OnChain,
            idempotency_key: "key".to_string(),
            idempotent_replay: true,
            status: "Published".to_string(),
            published_at: chrono::Utc::now(),
            total_attempts: 1,
            publications: vec![],
            audit_metadata: HashMap::new(),
            table_name: None,
            transaction_hash: None,
            persistence: None,
        };

        let json = serde_json::to_string(&receipt).unwrap();
        assert!(!json.contains("table_name"));
        assert!(!json.contains("transaction_hash"));
        assert!(!json.contains("\"persistence\""));
    }

    // ── AnchoringError ───────────────────────────────────────────

    #[test]
    fn test_anchoring_error_is_retryable() {
        let retryable = AnchoringError::AdapterFailure {
            adapter: "test".to_string(),
            code: "TIMEOUT".to_string(),
            message: "timeout".to_string(),
            retryable: true,
        };
        assert!(retryable.is_retryable());

        let non_retryable = AnchoringError::AdapterFailure {
            adapter: "test".to_string(),
            code: "INVALID".to_string(),
            message: "bad input".to_string(),
            retryable: false,
        };
        assert!(!non_retryable.is_retryable());

        // Non-adapter errors are never retryable
        assert!(!AnchoringError::Validation {
            message: "err".to_string()
        }
        .is_retryable());
        assert!(!AnchoringError::IdempotencyConflict {
            idempotency_key: "k".to_string(),
            existing_fingerprint: "a".to_string(),
            incoming_fingerprint: "b".to_string(),
            existing_state_root: "s1".to_string(),
            incoming_state_root: "s2".to_string(),
        }
        .is_retryable());
        assert!(!AnchoringError::RetryExhausted {
            adapter: "a".to_string(),
            attempts: 3,
            message: "m".to_string(),
        }
        .is_retryable());
    }

    #[test]
    fn test_anchoring_error_code() {
        assert_eq!(
            AnchoringError::Validation {
                message: "x".to_string()
            }
            .code(),
            "validation_error"
        );
        assert_eq!(
            AnchoringError::IdempotencyConflict {
                idempotency_key: "k".to_string(),
                existing_fingerprint: "a".to_string(),
                incoming_fingerprint: "b".to_string(),
                existing_state_root: "s1".to_string(),
                incoming_state_root: "s2".to_string(),
            }
            .code(),
            "idempotency_conflict"
        );
        assert_eq!(
            AnchoringError::AdapterFailure {
                adapter: "a".to_string(),
                code: "X".to_string(),
                message: "m".to_string(),
                retryable: false,
            }
            .code(),
            "adapter_failure"
        );
        assert_eq!(
            AnchoringError::RetryExhausted {
                adapter: "a".to_string(),
                attempts: 3,
                message: "m".to_string(),
            }
            .code(),
            "retry_exhausted"
        );
    }

    #[test]
    fn test_anchoring_error_display() {
        let err = AnchoringError::Validation {
            message: "bad input".to_string(),
        };
        assert_eq!(err.to_string(), "validation error: bad input");

        let err = AnchoringError::IdempotencyConflict {
            idempotency_key: "idem-42".to_string(),
            existing_fingerprint: "fp1".to_string(),
            incoming_fingerprint: "fp2".to_string(),
            existing_state_root: "s1".to_string(),
            incoming_state_root: "s2".to_string(),
        };
        assert!(err.to_string().contains("idem-42"));

        let err = AnchoringError::RetryExhausted {
            adapter: "on_chain".to_string(),
            attempts: 5,
            message: "all retries failed".to_string(),
        };
        assert!(err.to_string().contains("on_chain"));
        assert!(err.to_string().contains("5 attempts"));
    }

    #[test]
    fn test_anchoring_error_serialization_tagged() {
        // Verify serde(tag = "kind") works correctly
        let err = AnchoringError::AdapterFailure {
            adapter: "test-adapter".to_string(),
            code: "500".to_string(),
            message: "server error".to_string(),
            retryable: true,
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains(r#""kind":"adapter_failure""#));
        assert!(json.contains("test-adapter"));

        let roundtrip: AnchoringError = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.code(), "adapter_failure");
        assert!(roundtrip.is_retryable());
    }

    #[test]
    fn test_anchoring_error_is_std_error() {
        let err: &dyn std::error::Error = &AnchoringError::Validation {
            message: "test".to_string(),
        };
        // Just verify it can be used as a std error
        let _ = err.to_string();
    }

    // ── TablelandAnchoringPublisher ──────────────────────────────

    #[test]
    fn test_tableland_publisher_name() {
        assert_eq!(TablelandAnchoringPublisher.name(), "tableland");
    }

    #[test]
    fn test_tableland_publish_success() {
        let publisher = TablelandAnchoringPublisher;
        let req = AnchoringRequest {
            state_root: "0xabcdef1234567890abcdef1234567890".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let result = publisher.publish(&req, 1);
        assert!(result.is_ok());

        let pub_item = result.unwrap();
        assert_eq!(pub_item.adapter, "tableland");
        assert_eq!(pub_item.status, "Finalized");
        assert!(pub_item.reference.starts_with("0xtbl"));
        assert_eq!(pub_item.attempts, 1);
        assert!(pub_item.metadata.contains_key("commitment_type"));
    }

    #[test]
    fn test_tableland_publish_empty_state_root() {
        let publisher = TablelandAnchoringPublisher;
        let req = AnchoringRequest {
            state_root: "   ".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let err = publisher.publish(&req, 1).unwrap_err();
        assert_eq!(err.code(), "validation_error");
        assert!(err.to_string().contains("state_root"));
    }

    #[test]
    fn test_tableland_publish_with_metadata() {
        let publisher = TablelandAnchoringPublisher;
        let mut metadata = HashMap::new();
        metadata.insert("custom".to_string(), "value".to_string());

        let req = AnchoringRequest {
            state_root: "0xhash".to_string(),
            target: AnchoringTarget::Tableland,
            idempotency_key: None,
            metadata,
            max_retry_attempts: 3,
        };

        let pub_item = publisher.publish(&req, 2).unwrap();
        assert_eq!(pub_item.attempts, 2);
        assert_eq!(pub_item.metadata.get("custom").unwrap(), "value");
        assert!(pub_item.metadata.contains_key("table_name"));
    }

    // ── OnChainAnchoringPublisher ────────────────────────────────

    #[test]
    fn test_onchain_publisher_name() {
        assert_eq!(OnChainAnchoringPublisher.name(), "on_chain");
    }

    #[test]
    fn test_onchain_publish_success() {
        let publisher = OnChainAnchoringPublisher;
        let req = AnchoringRequest {
            state_root: "0xabcdef1234567890abcdef1234567890".to_string(),
            target: AnchoringTarget::OnChain,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let result = publisher.publish(&req, 1);
        assert!(result.is_ok());

        let pub_item = result.unwrap();
        assert_eq!(pub_item.adapter, "on_chain");
        assert_eq!(pub_item.status, "Broadcasted");
        assert!(pub_item.reference.starts_with("0xonc"));
        assert!(pub_item.metadata.contains_key("chain_network"));
        assert_eq!(
            pub_item.metadata.get("chain_network").unwrap(),
            "bitcoin-mainnet"
        );
    }

    #[test]
    fn test_onchain_publish_empty_state_root() {
        let publisher = OnChainAnchoringPublisher;
        let req = AnchoringRequest {
            state_root: "".to_string(),
            target: AnchoringTarget::OnChain,
            idempotency_key: None,
            metadata: HashMap::new(),
            max_retry_attempts: 3,
        };

        let err = publisher.publish(&req, 1).unwrap_err();
        assert_eq!(err.code(), "validation_error");
    }

    #[test]
    fn test_onchain_publish_custom_metadata_overrides_defaults() {
        let publisher = OnChainAnchoringPublisher;
        let mut metadata = HashMap::new();
        metadata.insert("chain_network".to_string(), "bitcoin-testnet".to_string());
        metadata.insert(
            "commitment_contract".to_string(),
            "custom-registry-v2".to_string(),
        );

        let req = AnchoringRequest {
            state_root: "0xhash".to_string(),
            target: AnchoringTarget::OnChain,
            idempotency_key: None,
            metadata,
            max_retry_attempts: 3,
        };

        let pub_item = publisher.publish(&req, 3).unwrap();
        assert_eq!(pub_item.attempts, 3);
        assert_eq!(
            pub_item.metadata.get("chain_network").unwrap(),
            "bitcoin-testnet"
        );
        assert_eq!(
            pub_item.metadata.get("commitment_contract").unwrap(),
            "custom-registry-v2"
        );
    }

    // ── compact_state_root ───────────────────────────────────────

    #[test]
    fn test_compact_state_root_short() {
        assert_eq!(compact_state_root("abc"), "abc");
        assert_eq!(compact_state_root("0xabc"), "abc");
        assert_eq!(compact_state_root("  abc  "), "abc");
    }

    #[test]
    fn test_compact_state_root_long() {
        let root = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let compact = compact_state_root(root);
        // First 8 + last 8 chars, lowercase
        assert_eq!(compact.len(), 16);
        assert!(compact.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compact_state_root_0x_prefix() {
        let with_prefix = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let without_prefix = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        assert_eq!(
            compact_state_root(with_prefix),
            compact_state_root(without_prefix)
        );
    }

    #[test]
    fn test_compact_state_root_uppercase() {
        assert_eq!(compact_state_root("0xABCDEF"), "abcdef");
    }
}
