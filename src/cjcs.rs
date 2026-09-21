//! Canonical Job Card System (CJCS)
//!
//! Provides JSON-LD structured work intents for sBTC settlements and decentralized
//! task orchestration. Enforces strict, fail-closed parameter validation and typed error handling.

use serde::{Deserialize, Serialize};

/// Typed error variants for the Canonical Job Card System (CJCS).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CjcsError {
    /// Context field is empty or malformed.
    InvalidContext,
    /// Type field is empty or unsupported.
    InvalidType,
    /// Sender address is empty or invalid.
    InvalidSenderAddress,
    /// Receiver address is empty or invalid.
    InvalidReceiverAddress,
    /// Task ID is empty or invalid.
    InvalidTaskId,
    /// Settlement amount is zero or invalid.
    InvalidAmount,
    /// Failed to serialize the job card to JSON.
    SerializationFailed(String),
    /// Failed to deserialize the job card from JSON.
    DeserializationFailed(String),
}

impl std::fmt::Display for CjcsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidContext => write!(f, "invalid or empty CJCS @context"),
            Self::InvalidType => write!(f, "invalid or empty CJCS @type"),
            Self::InvalidSenderAddress => write!(f, "invalid or empty sender address"),
            Self::InvalidReceiverAddress => write!(f, "invalid or empty receiver address"),
            Self::InvalidTaskId => write!(f, "invalid or empty task ID"),
            Self::InvalidAmount => write!(f, "amount_sbtc must be greater than zero"),
            Self::SerializationFailed(err) => write!(f, "failed to serialize job card: {err}"),
            Self::DeserializationFailed(err) => write!(f, "failed to deserialize job card: {err}"),
        }
    }
}

impl std::error::Error for CjcsError {}

/// Represents an underlying work intent for a job card.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct WorkIntent {
    pub sender_address: String,
    pub receiver_address: String,
    pub task_id: String,
    pub amount_sbtc: u64,
}

impl WorkIntent {
    /// Create a new `WorkIntent` with fail-closed parameter validation.
    pub fn new(
        sender_address: impl Into<String>,
        receiver_address: impl Into<String>,
        task_id: impl Into<String>,
        amount_sbtc: u64,
    ) -> Result<Self, CjcsError> {
        let intent = Self {
            sender_address: sender_address.into(),
            receiver_address: receiver_address.into(),
            task_id: task_id.into(),
            amount_sbtc,
        };
        intent.validate()?;
        Ok(intent)
    }

    /// Enforces fail-closed validation rules on the work intent parameters.
    pub fn validate(&self) -> Result<(), CjcsError> {
        if self.sender_address.trim().is_empty() {
            return Err(CjcsError::InvalidSenderAddress);
        }
        if self.receiver_address.trim().is_empty() {
            return Err(CjcsError::InvalidReceiverAddress);
        }
        if self.task_id.trim().is_empty() {
            return Err(CjcsError::InvalidTaskId);
        }
        if self.amount_sbtc == 0 {
            return Err(CjcsError::InvalidAmount);
        }
        Ok(())
    }
}

/// Represents a Canonical Job Card containing JSON-LD context and a work intent.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct JobCard {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub r#type: String,
    pub work_intent: WorkIntent,
}

impl JobCard {
    /// Default canonical JSON-LD schema context for CJCS job cards.
    pub const DEFAULT_CONTEXT: &'static str = "https://schema.conxian.io/v1/cjcs.jsonld";
    /// Default card type for standard work intent cards.
    pub const DEFAULT_TYPE: &'static str = "WorkIntentJobCard";

    /// Create a new `JobCard` using default schema context and card type.
    pub fn new(work_intent: WorkIntent) -> Result<Self, CjcsError> {
        Self::new_with_context(Self::DEFAULT_CONTEXT, Self::DEFAULT_TYPE, work_intent)
    }

    /// Create a new `JobCard` with custom context and type parameters.
    pub fn new_with_context(
        context: impl Into<String>,
        r#type: impl Into<String>,
        work_intent: WorkIntent,
    ) -> Result<Self, CjcsError> {
        let card = Self {
            context: context.into(),
            r#type: r#type.into(),
            work_intent,
        };
        card.validate()?;
        Ok(card)
    }

    /// Validates the job card context, type, and underlying work intent.
    pub fn validate(&self) -> Result<(), CjcsError> {
        if self.context.trim().is_empty() {
            return Err(CjcsError::InvalidContext);
        }
        if self.r#type.trim().is_empty() {
            return Err(CjcsError::InvalidType);
        }
        self.work_intent.validate()
    }

    /// Serializes the job card to a canonical JSON string.
    pub fn to_json(&self) -> Result<String, CjcsError> {
        self.validate()?;
        serde_json::to_string(self).map_err(|e| CjcsError::SerializationFailed(e.to_string()))
    }

    /// Deserializes and validates a job card from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, CjcsError> {
        let card: Self = serde_json::from_str(json_str)
            .map_err(|e| CjcsError::DeserializationFailed(e.to_string()))?;
        card.validate()?;
        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_work_intent_and_job_card() {
        let intent =
            WorkIntent::new("SP123_SENDER", "SP456_RECEIVER", "task-888", 100_000).unwrap();
        let card = JobCard::new(intent.clone()).unwrap();

        assert_eq!(card.context, JobCard::DEFAULT_CONTEXT);
        assert_eq!(card.r#type, JobCard::DEFAULT_TYPE);
        assert_eq!(card.work_intent, intent);

        let json = card.to_json().unwrap();
        assert!(json.contains("@context"));
        assert!(json.contains("task-888"));

        let roundtrip = JobCard::from_json(&json).unwrap();
        assert_eq!(roundtrip, card);
    }

    #[test]
    fn test_invalid_work_intent_parameters() {
        assert_eq!(
            WorkIntent::new("", "receiver", "task-1", 100).unwrap_err(),
            CjcsError::InvalidSenderAddress
        );
        assert_eq!(
            WorkIntent::new("sender", "  ", "task-1", 100).unwrap_err(),
            CjcsError::InvalidReceiverAddress
        );
        assert_eq!(
            WorkIntent::new("sender", "receiver", "", 100).unwrap_err(),
            CjcsError::InvalidTaskId
        );
        assert_eq!(
            WorkIntent::new("sender", "receiver", "task-1", 0).unwrap_err(),
            CjcsError::InvalidAmount
        );
    }

    #[test]
    fn test_invalid_job_card_context_and_type() {
        let intent = WorkIntent::new("sender", "receiver", "task-1", 100).unwrap();

        assert_eq!(
            JobCard::new_with_context("", "WorkIntentJobCard", intent.clone()).unwrap_err(),
            CjcsError::InvalidContext
        );
        assert_eq!(
            JobCard::new_with_context(JobCard::DEFAULT_CONTEXT, "  ", intent).unwrap_err(),
            CjcsError::InvalidType
        );
    }

    #[test]
    fn test_deserialization_failures() {
        assert!(matches!(
            JobCard::from_json("invalid json"),
            Err(CjcsError::DeserializationFailed(_))
        ));

        let invalid_amount_json = r#"{
            "@context": "https://schema.conxian.io/v1/cjcs.jsonld",
            "@type": "WorkIntentJobCard",
            "work_intent": {
                "sender_address": "sender",
                "receiver_address": "receiver",
                "task_id": "task-1",
                "amount_sbtc": 0
            }
        }"#;

        assert_eq!(
            JobCard::from_json(invalid_amount_json).unwrap_err(),
            CjcsError::InvalidAmount
        );
    }

    #[test]
    fn test_error_display_formatting() {
        assert_eq!(
            CjcsError::InvalidContext.to_string(),
            "invalid or empty CJCS @context"
        );
        assert_eq!(
            CjcsError::InvalidType.to_string(),
            "invalid or empty CJCS @type"
        );
        assert_eq!(
            CjcsError::InvalidSenderAddress.to_string(),
            "invalid or empty sender address"
        );
        assert_eq!(
            CjcsError::InvalidReceiverAddress.to_string(),
            "invalid or empty receiver address"
        );
        assert_eq!(
            CjcsError::InvalidTaskId.to_string(),
            "invalid or empty task ID"
        );
        assert_eq!(
            CjcsError::InvalidAmount.to_string(),
            "amount_sbtc must be greater than zero"
        );
        assert_eq!(
            CjcsError::SerializationFailed("error".into()).to_string(),
            "failed to serialize job card: error"
        );
        assert_eq!(
            CjcsError::DeserializationFailed("error".into()).to_string(),
            "failed to deserialize job card: error"
        );
    }
}
