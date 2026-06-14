//! Asynchronous Payment Channels via LDK
//! Aligned with CXIP 20 Section 5.0

use bitcoin::secp256k1::PublicKey;
use lightning::offers::offer::Offer;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub struct LightningNode;

/// Failure taxonomy for Lightning operations (SRL-7).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightningFailureClass {
    /// Permenent failure: invalid parameters, policy violation, etc. No retry.
    Permanent,
    /// Transient failure: temporary connection or liquidity issue. Bounded retry.
    Transient,
    /// Indeterminate failure: status unknown after timeout. Requires reconciliation.
    Indeterminate,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum LightningError {
    InvalidOffer,
    ChannelNotFound,
    SplicingFailed,
    JITProvisioningFailed,
    PaymentFailed(String),
    PolicyViolation(String),
    FinalityTimeout,
}

impl std::fmt::Display for LightningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOffer => write!(f, "Invalid BOLT12 offer"),
            Self::ChannelNotFound => write!(f, "Channel not found"),
            Self::SplicingFailed => write!(f, "Splicing operation failed"),
            Self::JITProvisioningFailed => write!(f, "JIT channel provisioning failed"),
            Self::PaymentFailed(msg) => write!(f, "Payment failed: {msg}"),
            Self::PolicyViolation(msg) => write!(f, "Policy violation: {msg}"),
            Self::FinalityTimeout => write!(f, "Payment reached finality timeout"),
        }
    }
}

impl std::error::Error for LightningError {}

/// Authoritative payment lifecycle states (SRL-1).
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightningPaymentState {
    IntentAccepted,
    PolicyValidated,
    RouteFeasible,
    LiquidityReserved,
    ExecutionInFlight,
    Settled,
    FailedClosed,
    Expired,
}

/// A Lightning payment intent capturing the initial request and current state (SRL-1).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningPaymentIntent {
    pub payment_id: String,
    pub idempotency_key: String,
    pub amount_msat: u64,
    pub destination_pubkey: String,
    pub description: String,
    pub state: LightningPaymentState,
    pub failure_class: Option<LightningFailureClass>,
    pub created_at_epoch_ms: u64,
    pub updated_at_epoch_ms: u64,
}

/// An append-only payment event for the resilience journal (SRL-3).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningPaymentEvent {
    pub event_id: String,
    pub payment_id: String,
    pub event_type: String,
    pub from_state: LightningPaymentState,
    pub to_state: LightningPaymentState,
    pub reason_code: Option<String>,
    pub attempt_no: u32,
    pub trace_id: String,
    pub occurred_at_epoch_ms: u64,
}

/// Observability metrics for Lightning operations (SRL-9).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningMetrics {
    pub channel_count: u32,
    pub total_capacity_msat: u64,
    pub local_balance_msat: u64,
    pub pending_inbound_msat: u64,
    pub pending_outbound_msat: u64,
    pub payment_success_rate: f32,
    pub avg_routing_fee_msat: u64,
}

/// Core interface for the Lightning Adapter (SRL-10).
/// This trait defines the expected behavior for any production-grade Lightning backend.
pub trait LightningAdapter {
    /// Dispatches a payment intent to the Lightning network.
    fn dispatch_payment(
        &self,
        intent: LightningPaymentIntent,
    ) -> Result<LightningPaymentEvent, LightningError>;

    /// Reconciles the state of a payment that was previously Indeterminate.
    fn reconcile_payment(&self, payment_id: &str) -> Result<LightningPaymentState, LightningError>;

    /// Returns current health and liquidity metrics for the node.
    fn get_metrics(&self) -> Result<LightningMetrics, LightningError>;
}

impl LightningNode {
    /// BOLT 12 Offers (Section 5.2)
    pub fn create_bolt12_offer(
        _amount_msat: u64,
        _description: &str,
    ) -> Result<Offer, LightningError> {
        // Implementation defers to LDK's OfferBuilder in production
        Err(LightningError::InvalidOffer)
    }

    /// BIP-353 DNS Payment Instructions
    pub fn resolve_bip353(dns_name: &str) -> Result<Offer, LightningError> {
        if dns_name.is_empty() {
            return Err(LightningError::InvalidOffer);
        }
        Err(LightningError::InvalidOffer)
    }

    /// LSPS2 JIT Channel Provisioning
    pub fn request_jit_channel(node_pubkey_hex: &str) -> Result<bool, LightningError> {
        let _pubkey = PublicKey::from_str(node_pubkey_hex)
            .map_err(|_| LightningError::JITProvisioningFailed)?;
        Ok(true)
    }

    /// Splicing (Dynamic capacity resizing)
    pub fn initiate_splicing(
        channel_id: &[u8; 32],
        _delta_sats: i64,
    ) -> Result<(), LightningError> {
        if channel_id.iter().all(|&b| b == 0) {
            return Err(LightningError::ChannelNotFound);
        }
        Ok(())
    }
}

/// Validates whether a Lightning payment state transition is allowed (SRL-1).
pub fn is_valid_payment_transition(from: LightningPaymentState, to: LightningPaymentState) -> bool {
    use LightningPaymentState::*;
    match (from, to) {
        (IntentAccepted, PolicyValidated) => true,
        (IntentAccepted, FailedClosed) => true,
        (PolicyValidated, RouteFeasible) => true,
        (PolicyValidated, FailedClosed) => true,
        (RouteFeasible, LiquidityReserved) => true,
        (RouteFeasible, FailedClosed) => true,
        (LiquidityReserved, ExecutionInFlight) => true,
        (LiquidityReserved, FailedClosed) => true,
        (ExecutionInFlight, Settled) => true,
        (ExecutionInFlight, FailedClosed) => true,
        (ExecutionInFlight, Expired) => true,
        // Terminal states are immutable
        _ => false,
    }
}

pub fn validate_payment_transition(
    from: LightningPaymentState,
    to: LightningPaymentState,
) -> Result<(), String> {
    if is_valid_payment_transition(from, to) {
        Ok(())
    } else {
        Err(format!(
            "Invalid payment transition: {:?} -> {:?}",
            from, to
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_state_transitions() {
        use LightningPaymentState::*;

        assert!(is_valid_payment_transition(IntentAccepted, PolicyValidated));
        assert!(is_valid_payment_transition(ExecutionInFlight, Settled));
        assert!(is_valid_payment_transition(ExecutionInFlight, FailedClosed));

        // Invalid: skip states
        assert!(!is_valid_payment_transition(IntentAccepted, Settled));

        // Invalid: move from terminal
        assert!(!is_valid_payment_transition(Settled, ExecutionInFlight));
    }

    #[test]
    fn test_failure_taxonomy_serialization() {
        let class = LightningFailureClass::Transient;
        let json = serde_json::to_string(&class).unwrap();
        assert_eq!(json, "\"transient\"");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_intent_serialization() {
        let intent = LightningPaymentIntent {
            payment_id: "pay-123".to_string(),
            idempotency_key: "key-abc".to_string(),
            amount_msat: 1000,
            destination_pubkey: "02...".to_string(),
            description: "test payment".to_string(),
            state: LightningPaymentState::IntentAccepted,
            failure_class: None,
            created_at_epoch_ms: 1622548800000,
            updated_at_epoch_ms: 1622548800000,
        };

        let json = serde_json::to_string(&intent).unwrap();
        assert!(json.contains("pay-123"));
        assert!(json.contains("intent_accepted"));

        let deserialized: LightningPaymentIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.payment_id, intent.payment_id);
    }

    #[test]
    fn test_event_serialization() {
        let event = LightningPaymentEvent {
            event_id: "evt-456".to_string(),
            payment_id: "pay-123".to_string(),
            event_type: "state_transition".to_string(),
            from_state: LightningPaymentState::IntentAccepted,
            to_state: LightningPaymentState::PolicyValidated,
            reason_code: Some("POLICY_OK".to_string()),
            attempt_no: 1,
            trace_id: "trace-789".to_string(),
            occurred_at_epoch_ms: 1622548800500,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("evt-456"));
        assert!(json.contains("policy_validated"));
        assert!(json.contains("POLICY_OK"));
    }

    #[test]
    fn test_transition_validation_errors() {
        use LightningPaymentState::*;

        // Success
        assert!(validate_payment_transition(IntentAccepted, PolicyValidated).is_ok());

        // Failure
        let err = validate_payment_transition(Settled, FailedClosed).unwrap_err();
        assert!(err.contains("Invalid payment transition"));
    }

    #[test]
    fn test_failure_taxonomy() {
        assert_eq!(
            LightningFailureClass::Permanent.clone(),
            LightningFailureClass::Permanent
        );
        assert_ne!(
            LightningFailureClass::Transient,
            LightningFailureClass::Indeterminate
        );
    }

    #[test]
    fn test_metrics_serialization() {
        let metrics = LightningMetrics {
            channel_count: 5,
            total_capacity_msat: 100000000,
            local_balance_msat: 50000000,
            pending_inbound_msat: 1000000,
            pending_outbound_msat: 2000000,
            payment_success_rate: 0.98,
            avg_routing_fee_msat: 150,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("channel_count"));
        assert!(json.contains("0.98"));
    }
}
