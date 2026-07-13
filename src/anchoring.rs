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
        normalized[..8].to_ascii_lowercase(),
        normalized[normalized.len() - 8..].to_ascii_lowercase()
    )
}
