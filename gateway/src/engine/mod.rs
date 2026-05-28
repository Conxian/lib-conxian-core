pub mod persistence;

use persistence::{
    AppendEventOutcome, BitcoinTxPersistence, BtcTxEventRecord, BtcTxOrchestrationRecord,
    InMemoryBitcoinTxPersistence, PersistenceError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinTxLifecycleState {
    #[default]
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
    pub fn as_str(self) -> &'static str {
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinTxTransition {
    Sign,
    QueueBroadcast,
    MempoolObserved,
    ConfirmationsObserved,
    Finalize,
    ReorgDetected,
    MarkDeadLetter,
}

impl BitcoinTxTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            BitcoinTxTransition::Sign => "sign",
            BitcoinTxTransition::QueueBroadcast => "queue_broadcast",
            BitcoinTxTransition::MempoolObserved => "mempool_observed",
            BitcoinTxTransition::ConfirmationsObserved => "confirmations_observed",
            BitcoinTxTransition::Finalize => "finalize",
            BitcoinTxTransition::ReorgDetected => "reorg_detected",
            BitcoinTxTransition::MarkDeadLetter => "mark_dead_letter",
        }
    }
}

impl FromStr for BitcoinTxTransition {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sign" => Ok(BitcoinTxTransition::Sign),
            "queue_broadcast" => Ok(BitcoinTxTransition::QueueBroadcast),
            "mempool_observed" => Ok(BitcoinTxTransition::MempoolObserved),
            "confirmations_observed" => Ok(BitcoinTxTransition::ConfirmationsObserved),
            "finalize" => Ok(BitcoinTxTransition::Finalize),
            "reorg_detected" => Ok(BitcoinTxTransition::ReorgDetected),
            "mark_dead_letter" => Ok(BitcoinTxTransition::MarkDeadLetter),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinTxOrchestration {
    pub tx_id: String,
    pub state: BitcoinTxLifecycleState,
    pub latest_transition: Option<BitcoinTxTransition>,
    pub latest_event_id: Option<String>,
    pub fee_rate_sat_vb: Option<u64>,
    pub attempt: u32,
    pub observed_confirmations: Option<u32>,
    pub recovery_cursor: u64,
    pub updated_at_epoch_ms: u64,
}

impl BitcoinTxOrchestration {
    fn new(tx_id: String) -> Self {
        let now = now_epoch_ms();
        Self {
            tx_id,
            state: BitcoinTxLifecycleState::Draft,
            latest_transition: None,
            latest_event_id: None,
            fee_rate_sat_vb: None,
            attempt: 0,
            observed_confirmations: None,
            recovery_cursor: now,
            updated_at_epoch_ms: now,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinTxEvent {
    pub event_id: String,
    pub tx_id: String,
    pub idempotency_key: String,
    pub transition: BitcoinTxTransition,
    pub from_state: BitcoinTxLifecycleState,
    pub to_state: BitcoinTxLifecycleState,
    pub attempt: u32,
    pub fee_rate_sat_vb: Option<u64>,
    pub observed_confirmations: Option<u32>,
    pub rationale: Option<String>,
    pub fingerprint: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinTxTransitionInput {
    pub tx_id: String,
    pub transition: BitcoinTxTransition,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub fee_rate_sat_vb: Option<u64>,
    #[serde(default)]
    pub observed_confirmations: Option<u32>,
    #[serde(default)]
    pub attempt: Option<u32>,
}

impl BitcoinTxTransitionInput {
    fn normalize(self) -> Result<Self, BitcoinTxTransitionError> {
        let tx_id = self.tx_id.trim().to_string();
        if tx_id.is_empty() {
            return Err(BitcoinTxTransitionError::Validation(
                "tx_id is required".to_string(),
            ));
        }

        let idempotency_key = self
            .idempotency_key
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty());

        let rationale = self
            .rationale
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self {
            tx_id,
            transition: self.transition,
            idempotency_key,
            rationale,
            fee_rate_sat_vb: self.fee_rate_sat_vb,
            observed_confirmations: self.observed_confirmations,
            attempt: self.attempt,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinTxTransitionOutcome {
    pub tx_id: String,
    pub event_id: String,
    pub idempotency_key: String,
    pub from_state: BitcoinTxLifecycleState,
    pub to_state: BitcoinTxLifecycleState,
    pub idempotent_replay: bool,
    pub mutated_state: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BitcoinTxTransitionError {
    Validation(String),
    InvalidTransition {
        tx_id: String,
        from: BitcoinTxLifecycleState,
        transition: BitcoinTxTransition,
    },
    UnknownPersistedState(String),
    UnknownPersistedTransition(String),
    IdempotencyConflict {
        tx_id: String,
        idempotency_key: String,
        existing_fingerprint: String,
        incoming_fingerprint: String,
    },
    Persistence(String),
}

impl std::fmt::Display for BitcoinTxTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitcoinTxTransitionError::Validation(message) => {
                write!(f, "validation error: {message}")
            }
            BitcoinTxTransitionError::InvalidTransition {
                tx_id,
                from,
                transition,
            } => write!(
                f,
                "invalid transition for tx {}: state={} transition={}",
                tx_id,
                from.as_str(),
                transition.as_str()
            ),
            BitcoinTxTransitionError::UnknownPersistedState(value) => {
                write!(f, "unknown persisted state: {value}")
            }
            BitcoinTxTransitionError::UnknownPersistedTransition(value) => {
                write!(f, "unknown persisted transition: {value}")
            }
            BitcoinTxTransitionError::IdempotencyConflict {
                tx_id,
                idempotency_key,
                ..
            } => write!(
                f,
                "idempotency conflict for tx_id={tx_id}, idempotency_key={idempotency_key}"
            ),
            BitcoinTxTransitionError::Persistence(message) => {
                write!(f, "persistence error: {message}")
            }
        }
    }
}

impl std::error::Error for BitcoinTxTransitionError {}

pub struct BitcoinTxLifecycleEngine {
    orchestrations: RwLock<HashMap<String, BitcoinTxOrchestration>>,
    persistence: Arc<dyn BitcoinTxPersistence>,
    event_sequence: AtomicU64,
}

impl Default for BitcoinTxLifecycleEngine {
    fn default() -> Self {
        Self::in_memory().expect("in-memory lifecycle engine should initialize")
    }
}

impl BitcoinTxLifecycleEngine {
    pub fn in_memory() -> Result<Self, BitcoinTxTransitionError> {
        Self::new(Arc::new(InMemoryBitcoinTxPersistence::default()))
    }

    pub fn new(
        persistence: Arc<dyn BitcoinTxPersistence>,
    ) -> Result<Self, BitcoinTxTransitionError> {
        let mut cached = HashMap::new();
        for record in persistence
            .list_orchestrations()
            .map_err(map_persistence_error)?
            .into_iter()
        {
            let orchestration = orchestration_from_record(record)?;
            cached.insert(orchestration.tx_id.clone(), orchestration);
        }

        Ok(Self {
            orchestrations: RwLock::new(cached),
            persistence,
            event_sequence: AtomicU64::new(now_epoch_ms()),
        })
    }

    pub fn apply_bitcoin_tx_transition(
        &self,
        input: BitcoinTxTransitionInput,
    ) -> Result<BitcoinTxTransitionOutcome, BitcoinTxTransitionError> {
        let input = input.normalize()?;
        let mut orchestration = self
            .get_bitcoin_tx_orchestration(&input.tx_id)?
            .unwrap_or_else(|| BitcoinTxOrchestration::new(input.tx_id.clone()));

        let from_state = orchestration.state;
        let to_state = resolve_transition_target(
            &input.tx_id,
            from_state,
            input.transition,
            input.observed_confirmations,
        )?;

        let idempotency_key = input.idempotency_key.clone().unwrap_or_else(|| {
            derive_idempotency_key(&input, orchestration.attempt, orchestration.fee_rate_sat_vb)
        });

        let effective_attempt = input.attempt.unwrap_or(orchestration.attempt);
        let effective_fee_rate_sat_vb = input.fee_rate_sat_vb.or(orchestration.fee_rate_sat_vb);
        let fingerprint = build_fingerprint(
            &input.tx_id,
            input.transition,
            effective_attempt,
            effective_fee_rate_sat_vb,
            input.observed_confirmations,
            input.rationale.as_deref(),
        );

        let event_id = format!(
            "evt-{}-{}",
            sanitize_id(&input.tx_id),
            self.event_sequence.fetch_add(1, Ordering::SeqCst)
        );

        let event = BtcTxEventRecord {
            event_id: event_id.clone(),
            tx_id: input.tx_id.clone(),
            idempotency_key: idempotency_key.clone(),
            transition: input.transition.as_str().to_string(),
            from_state: from_state.as_str().to_string(),
            to_state: to_state.as_str().to_string(),
            attempt: effective_attempt,
            fee_rate_sat_vb: effective_fee_rate_sat_vb,
            observed_confirmations: input.observed_confirmations,
            rationale: input.rationale.clone(),
            fingerprint: fingerprint.clone(),
            created_at_epoch_ms: now_epoch_ms(),
        };

        match self
            .persistence
            .append_event(event)
            .map_err(map_persistence_error)?
        {
            AppendEventOutcome::Duplicate(existing) => {
                let from_state = parse_state(&existing.from_state)?;
                let to_state = parse_state(&existing.to_state)?;
                Ok(BitcoinTxTransitionOutcome {
                    tx_id: existing.tx_id,
                    event_id: existing.event_id,
                    idempotency_key: existing.idempotency_key,
                    from_state,
                    to_state,
                    idempotent_replay: true,
                    mutated_state: false,
                })
            }
            AppendEventOutcome::Inserted => {
                orchestration.state = to_state;
                orchestration.latest_event_id = Some(event_id.clone());
                orchestration.latest_transition = Some(input.transition);
                orchestration.attempt = update_attempt(
                    orchestration.attempt,
                    input.transition,
                    input.attempt,
                    from_state,
                    to_state,
                );
                orchestration.fee_rate_sat_vb = effective_fee_rate_sat_vb;
                orchestration.observed_confirmations = input
                    .observed_confirmations
                    .or(orchestration.observed_confirmations);
                orchestration.recovery_cursor = now_epoch_ms();
                orchestration.updated_at_epoch_ms = orchestration.recovery_cursor;

                let orchestration_record = orchestration_to_record(&orchestration);
                self.persistence
                    .upsert_orchestration(orchestration_record)
                    .map_err(map_persistence_error)?;

                let mut cached = self.orchestrations.write().map_err(|err| {
                    BitcoinTxTransitionError::Persistence(format!("lock poisoned: {err}"))
                })?;
                cached.insert(orchestration.tx_id.clone(), orchestration.clone());

                Ok(BitcoinTxTransitionOutcome {
                    tx_id: orchestration.tx_id,
                    event_id,
                    idempotency_key,
                    from_state,
                    to_state,
                    idempotent_replay: false,
                    mutated_state: true,
                })
            }
        }
    }

    pub fn get_bitcoin_tx_orchestration(
        &self,
        tx_id: &str,
    ) -> Result<Option<BitcoinTxOrchestration>, BitcoinTxTransitionError> {
        let tx_id = tx_id.trim();
        if tx_id.is_empty() {
            return Err(BitcoinTxTransitionError::Validation(
                "tx_id is required".to_string(),
            ));
        }

        if let Some(orchestration) = self
            .orchestrations
            .read()
            .map_err(|err| BitcoinTxTransitionError::Persistence(format!("lock poisoned: {err}")))?
            .get(tx_id)
            .cloned()
        {
            return Ok(Some(orchestration));
        }

        let persisted = self
            .persistence
            .get_orchestration(tx_id)
            .map_err(map_persistence_error)?;

        let Some(record) = persisted else {
            return Ok(None);
        };

        let orchestration = orchestration_from_record(record)?;
        self.orchestrations
            .write()
            .map_err(|err| BitcoinTxTransitionError::Persistence(format!("lock poisoned: {err}")))?
            .insert(tx_id.to_string(), orchestration.clone());

        Ok(Some(orchestration))
    }

    pub fn list_bitcoin_tx_events(
        &self,
        tx_id: &str,
    ) -> Result<Vec<BitcoinTxEvent>, BitcoinTxTransitionError> {
        let tx_id = tx_id.trim();
        if tx_id.is_empty() {
            return Err(BitcoinTxTransitionError::Validation(
                "tx_id is required".to_string(),
            ));
        }

        self.persistence
            .list_events(tx_id)
            .map_err(map_persistence_error)?
            .into_iter()
            .map(event_from_record)
            .collect()
    }
}

fn parse_state(value: &str) -> Result<BitcoinTxLifecycleState, BitcoinTxTransitionError> {
    BitcoinTxLifecycleState::from_str(value)
        .map_err(|_| BitcoinTxTransitionError::UnknownPersistedState(value.to_string()))
}

fn parse_transition(value: &str) -> Result<BitcoinTxTransition, BitcoinTxTransitionError> {
    BitcoinTxTransition::from_str(value)
        .map_err(|_| BitcoinTxTransitionError::UnknownPersistedTransition(value.to_string()))
}

fn event_from_record(record: BtcTxEventRecord) -> Result<BitcoinTxEvent, BitcoinTxTransitionError> {
    Ok(BitcoinTxEvent {
        event_id: record.event_id,
        tx_id: record.tx_id,
        idempotency_key: record.idempotency_key,
        transition: parse_transition(&record.transition)?,
        from_state: parse_state(&record.from_state)?,
        to_state: parse_state(&record.to_state)?,
        attempt: record.attempt,
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        observed_confirmations: record.observed_confirmations,
        rationale: record.rationale,
        fingerprint: record.fingerprint,
        created_at_epoch_ms: record.created_at_epoch_ms,
    })
}

fn orchestration_from_record(
    record: BtcTxOrchestrationRecord,
) -> Result<BitcoinTxOrchestration, BitcoinTxTransitionError> {
    Ok(BitcoinTxOrchestration {
        tx_id: record.tx_id,
        state: parse_state(&record.state)?,
        latest_transition: record
            .latest_transition
            .as_deref()
            .map(parse_transition)
            .transpose()?,
        latest_event_id: record.latest_event_id,
        fee_rate_sat_vb: record.fee_rate_sat_vb,
        attempt: record.attempt,
        observed_confirmations: record.observed_confirmations,
        recovery_cursor: record.recovery_cursor,
        updated_at_epoch_ms: record.updated_at_epoch_ms,
    })
}

fn orchestration_to_record(orchestration: &BitcoinTxOrchestration) -> BtcTxOrchestrationRecord {
    BtcTxOrchestrationRecord {
        tx_id: orchestration.tx_id.clone(),
        state: orchestration.state.as_str().to_string(),
        latest_transition: orchestration
            .latest_transition
            .map(|transition| transition.as_str().to_string()),
        latest_event_id: orchestration.latest_event_id.clone(),
        fee_rate_sat_vb: orchestration.fee_rate_sat_vb,
        attempt: orchestration.attempt,
        observed_confirmations: orchestration.observed_confirmations,
        recovery_cursor: orchestration.recovery_cursor,
        updated_at_epoch_ms: orchestration.updated_at_epoch_ms,
    }
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
        .observed_confirmations
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());

    format!(
        "{}:{}:{}:{}:{}",
        input.tx_id,
        input.transition.as_str(),
        attempt,
        fee_rate,
        confirmations
    )
}

fn build_fingerprint(
    tx_id: &str,
    transition: BitcoinTxTransition,
    attempt: u32,
    fee_rate_sat_vb: Option<u64>,
    observed_confirmations: Option<u32>,
    rationale: Option<&str>,
) -> String {
    let fee = fee_rate_sat_vb
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let confirmations = observed_confirmations
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let rationale = rationale.unwrap_or("-");

    format!(
        "{}|{}|{}|{}|{}|{}",
        tx_id,
        transition.as_str(),
        attempt,
        fee,
        confirmations,
        rationale
    )
}

fn update_attempt(
    current_attempt: u32,
    transition: BitcoinTxTransition,
    explicit_attempt: Option<u32>,
    from_state: BitcoinTxLifecycleState,
    to_state: BitcoinTxLifecycleState,
) -> u32 {
    if let Some(explicit) = explicit_attempt {
        return explicit;
    }

    if transition == BitcoinTxTransition::QueueBroadcast
        && from_state != BitcoinTxLifecycleState::BroadcastPending
        && to_state == BitcoinTxLifecycleState::BroadcastPending
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

fn resolve_transition_target(
    tx_id: &str,
    current: BitcoinTxLifecycleState,
    transition: BitcoinTxTransition,
    observed_confirmations: Option<u32>,
) -> Result<BitcoinTxLifecycleState, BitcoinTxTransitionError> {
    use BitcoinTxLifecycleState as State;
    use BitcoinTxTransition as Transition;

    let target = match (current, transition) {
        (State::Draft, Transition::Sign) => State::Signed,
        (State::Draft, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::Signed, Transition::Sign) => State::Signed,
        (State::Signed, Transition::QueueBroadcast) => State::BroadcastPending,
        (State::Signed, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::BroadcastPending, Transition::QueueBroadcast) => State::BroadcastPending,
        (State::BroadcastPending, Transition::MempoolObserved) => State::InMempool,
        (State::BroadcastPending, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::InMempool, Transition::MempoolObserved) => State::InMempool,
        (State::InMempool, Transition::ConfirmationsObserved) => {
            if observed_confirmations.unwrap_or(0) >= 6 {
                State::Confirmed
            } else {
                State::PendingConfirmations
            }
        }
        (State::InMempool, Transition::ReorgDetected) => State::Reorged,
        (State::InMempool, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::PendingConfirmations, Transition::ConfirmationsObserved) => {
            if observed_confirmations.unwrap_or(0) >= 6 {
                State::Confirmed
            } else {
                State::PendingConfirmations
            }
        }
        (State::PendingConfirmations, Transition::ReorgDetected) => State::Reorged,
        (State::PendingConfirmations, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::Confirmed, Transition::ConfirmationsObserved) => State::Confirmed,
        (State::Confirmed, Transition::Finalize) => State::Finalized,
        (State::Confirmed, Transition::ReorgDetected) => State::Reorged,

        (State::Finalized, Transition::Finalize) => State::Finalized,

        (State::Reorged, Transition::QueueBroadcast) => State::BroadcastPending,
        (State::Reorged, Transition::MarkDeadLetter) => State::DeadLetter,

        (State::DeadLetter, Transition::MarkDeadLetter) => State::DeadLetter,

        _ => {
            return Err(BitcoinTxTransitionError::InvalidTransition {
                tx_id: tx_id.to_string(),
                from: current,
                transition,
            })
        }
    };

    Ok(target)
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway_engine::persistence::JsonFileBitcoinTxPersistence;
    use std::sync::Arc;

    #[test]
    fn duplicate_transition_is_idempotent_and_does_not_append_event_twice() {
        let persistence = Arc::new(InMemoryBitcoinTxPersistence::default());
        let engine = BitcoinTxLifecycleEngine::new(persistence).expect("engine should initialize");

        let first = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-1".to_string(),
                transition: BitcoinTxTransition::Sign,
                idempotency_key: Some("request-1".to_string()),
                rationale: Some("signature complete".to_string()),
                fee_rate_sat_vb: Some(12),
                observed_confirmations: None,
                attempt: None,
            })
            .expect("first transition should apply");

        let second = engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-1".to_string(),
                transition: BitcoinTxTransition::Sign,
                idempotency_key: Some("request-1".to_string()),
                rationale: Some("signature complete".to_string()),
                fee_rate_sat_vb: Some(12),
                observed_confirmations: None,
                attempt: None,
            })
            .expect("duplicate transition should be idempotent");

        assert!(!first.idempotent_replay);
        assert!(second.idempotent_replay);
        assert_eq!(first.event_id, second.event_id);

        let events = engine
            .list_bitcoin_tx_events("btc-tx-1")
            .expect("events should load");
        assert_eq!(events.len(), 1);

        let state = engine
            .get_bitcoin_tx_orchestration("btc-tx-1")
            .expect("state should load")
            .expect("state should exist");
        assert_eq!(state.state, BitcoinTxLifecycleState::Signed);
    }

    #[test]
    fn restart_recovery_rehydrates_orchestration_state_from_persistence() {
        let storage_path =
            std::env::temp_dir().join(format!("con717-recovery-{}.json", now_epoch_ms()));

        let persistence = Arc::new(JsonFileBitcoinTxPersistence::new(storage_path.clone()));

        let engine = BitcoinTxLifecycleEngine::new(persistence.clone())
            .expect("engine should initialize with file persistence");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-2".to_string(),
                transition: BitcoinTxTransition::Sign,
                idempotency_key: Some("req-sign".to_string()),
                rationale: None,
                fee_rate_sat_vb: Some(10),
                observed_confirmations: None,
                attempt: Some(0),
            })
            .expect("sign transition should persist");

        engine
            .apply_bitcoin_tx_transition(BitcoinTxTransitionInput {
                tx_id: "btc-tx-2".to_string(),
                transition: BitcoinTxTransition::QueueBroadcast,
                idempotency_key: Some("req-queue".to_string()),
                rationale: Some("ready for mempool".to_string()),
                fee_rate_sat_vb: Some(15),
                observed_confirmations: None,
                attempt: Some(1),
            })
            .expect("queue transition should persist");

        drop(engine);

        let restarted = BitcoinTxLifecycleEngine::new(persistence)
            .expect("restart should hydrate orchestration projection");

        let recovered = restarted
            .get_bitcoin_tx_orchestration("btc-tx-2")
            .expect("recovered state should be readable")
            .expect("orchestration should be recovered");

        assert_eq!(
            recovered.state,
            BitcoinTxLifecycleState::BroadcastPending,
            "state should survive restart"
        );
        assert_eq!(recovered.attempt, 1);
        assert_eq!(recovered.fee_rate_sat_vb, Some(15));

        let recovered_events = restarted
            .list_bitcoin_tx_events("btc-tx-2")
            .expect("recovered events should be readable");
        assert_eq!(recovered_events.len(), 2);

        let _ = std::fs::remove_file(storage_path);
    }
}
