use crate::gateway_engine::{
    BitcoinTxLifecycleEngine, BitcoinTxTransition, BitcoinTxTransitionError,
    BitcoinTxTransitionInput, BitcoinTxTransitionOutcome,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinTxTransitionIngress {
    #[serde(alias = "bitcoin_tx_id", alias = "orchestration_id")]
    pub tx_id: String,
    #[serde(alias = "lifecycle_event", alias = "event")]
    pub transition: BitcoinTxTransition,
    #[serde(default, alias = "request_id", alias = "idempotency")]
    pub idempotency_key: Option<String>,
    #[serde(default, alias = "decision_rationale", alias = "reason")]
    pub rationale: Option<String>,
    #[serde(default, alias = "fee_rate", alias = "sat_per_vbyte")]
    pub fee_rate_sat_vb: Option<u64>,
    #[serde(default, alias = "confirmations", alias = "confirmations_observed")]
    pub observed_confirmations: Option<u32>,
    #[serde(default, alias = "broadcast_attempt")]
    pub attempt: Option<u32>,
}

impl From<BitcoinTxTransitionIngress> for BitcoinTxTransitionInput {
    fn from(ingress: BitcoinTxTransitionIngress) -> Self {
        Self {
            tx_id: ingress.tx_id,
            transition: ingress.transition,
            idempotency_key: ingress.idempotency_key,
            rationale: ingress.rationale,
            fee_rate_sat_vb: ingress.fee_rate_sat_vb,
            observed_confirmations: ingress.observed_confirmations,
            attempt: ingress.attempt,
        }
    }
}

pub fn apply_bitcoin_tx_transition_ingress(
    engine: &BitcoinTxLifecycleEngine,
    ingress: BitcoinTxTransitionIngress,
) -> Result<BitcoinTxTransitionOutcome, BitcoinTxTransitionError> {
    engine.apply_bitcoin_tx_transition(ingress.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway_engine::{BitcoinTxLifecycleState, BitcoinTxTransition};

    #[test]
    fn ingress_aliases_remain_backward_compatible() {
        let legacy_payload = serde_json::json!({
            "bitcoin_tx_id": "tx-legacy-1",
            "lifecycle_event": "sign",
            "request_id": "legacy-request-1",
            "decision_rationale": "legacy ingress",
            "fee_rate": 11,
            "broadcast_attempt": 1
        });

        let ingress: BitcoinTxTransitionIngress =
            serde_json::from_value(legacy_payload).expect("legacy payload should parse");

        assert_eq!(ingress.tx_id, "tx-legacy-1");
        assert_eq!(ingress.transition, BitcoinTxTransition::Sign);
        assert_eq!(ingress.idempotency_key.as_deref(), Some("legacy-request-1"));
        assert_eq!(ingress.rationale.as_deref(), Some("legacy ingress"));
        assert_eq!(ingress.fee_rate_sat_vb, Some(11));
        assert_eq!(ingress.attempt, Some(1));
    }

    #[test]
    fn ingress_apply_uses_engine_transition_logic() {
        let engine = BitcoinTxLifecycleEngine::in_memory().expect("engine should initialize");
        let outcome = apply_bitcoin_tx_transition_ingress(
            &engine,
            BitcoinTxTransitionIngress {
                tx_id: "tx-api-1".to_string(),
                transition: BitcoinTxTransition::Sign,
                idempotency_key: Some("req-1".to_string()),
                rationale: Some("api ingress".to_string()),
                fee_rate_sat_vb: Some(9),
                observed_confirmations: None,
                attempt: Some(0),
            },
        )
        .expect("ingress transition should apply");

        assert_eq!(outcome.to_state, BitcoinTxLifecycleState::Signed);
        assert!(!outcome.idempotent_replay);

        let orchestration = engine
            .get_bitcoin_tx_orchestration("tx-api-1")
            .expect("state read should succeed")
            .expect("state should exist");
        assert_eq!(orchestration.state, BitcoinTxLifecycleState::Signed);
    }
}
