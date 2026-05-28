use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

pub const BTC_TX_SCHEMA_SQL: &str = include_str!("schema.sql");

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppendEventOutcome {
    Inserted,
    Duplicate(BtcTxEventRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistenceError {
    Io(String),
    Serialization(String),
    IdempotencyConflict {
        tx_id: String,
        idempotency_key: String,
        existing_fingerprint: String,
        incoming_fingerprint: String,
    },
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(message) => write!(f, "io error: {message}"),
            PersistenceError::Serialization(message) => write!(f, "serialization error: {message}"),
            PersistenceError::IdempotencyConflict {
                tx_id,
                idempotency_key,
                ..
            } => write!(
                f,
                "idempotency conflict for tx_id={tx_id}, idempotency_key={idempotency_key}"
            ),
        }
    }
}

impl std::error::Error for PersistenceError {}

pub trait BitcoinTxPersistence: Send + Sync {
    fn upsert_orchestration(
        &self,
        record: BtcTxOrchestrationRecord,
    ) -> Result<(), PersistenceError>;
    fn append_event(&self, event: BtcTxEventRecord)
        -> Result<AppendEventOutcome, PersistenceError>;
    fn get_orchestration(
        &self,
        tx_id: &str,
    ) -> Result<Option<BtcTxOrchestrationRecord>, PersistenceError>;
    fn list_orchestrations(&self) -> Result<Vec<BtcTxOrchestrationRecord>, PersistenceError>;
    fn list_events(&self, tx_id: &str) -> Result<Vec<BtcTxEventRecord>, PersistenceError>;
}

#[derive(Default)]
pub struct InMemoryBitcoinTxPersistence {
    state: RwLock<InMemoryState>,
}

#[derive(Default)]
struct InMemoryState {
    orchestrations: BTreeMap<String, BtcTxOrchestrationRecord>,
    events: Vec<BtcTxEventRecord>,
    idempotency_index: BTreeMap<(String, String), usize>,
}

impl InMemoryBitcoinTxPersistence {
    fn append_event_locked(
        state: &mut InMemoryState,
        event: BtcTxEventRecord,
    ) -> Result<AppendEventOutcome, PersistenceError> {
        let idempotency_key = (event.tx_id.clone(), event.idempotency_key.clone());

        if let Some(index) = state.idempotency_index.get(&idempotency_key).copied() {
            let existing = state
                .events
                .get(index)
                .cloned()
                .expect("idempotency index points to an existing event");
            if existing.fingerprint == event.fingerprint {
                return Ok(AppendEventOutcome::Duplicate(existing));
            }

            return Err(PersistenceError::IdempotencyConflict {
                tx_id: event.tx_id,
                idempotency_key: event.idempotency_key,
                existing_fingerprint: existing.fingerprint,
                incoming_fingerprint: event.fingerprint,
            });
        }

        let index = state.events.len();
        state.events.push(event.clone());
        state.idempotency_index.insert(idempotency_key, index);
        Ok(AppendEventOutcome::Inserted)
    }
}

impl BitcoinTxPersistence for InMemoryBitcoinTxPersistence {
    fn upsert_orchestration(
        &self,
        record: BtcTxOrchestrationRecord,
    ) -> Result<(), PersistenceError> {
        let mut state = self
            .state
            .write()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        state.orchestrations.insert(record.tx_id.clone(), record);
        Ok(())
    }

    fn append_event(
        &self,
        event: BtcTxEventRecord,
    ) -> Result<AppendEventOutcome, PersistenceError> {
        let mut state = self
            .state
            .write()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        Self::append_event_locked(&mut state, event)
    }

    fn get_orchestration(
        &self,
        tx_id: &str,
    ) -> Result<Option<BtcTxOrchestrationRecord>, PersistenceError> {
        let state = self
            .state
            .read()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        Ok(state.orchestrations.get(tx_id).cloned())
    }

    fn list_orchestrations(&self) -> Result<Vec<BtcTxOrchestrationRecord>, PersistenceError> {
        let state = self
            .state
            .read()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        Ok(state.orchestrations.values().cloned().collect())
    }

    fn list_events(&self, tx_id: &str) -> Result<Vec<BtcTxEventRecord>, PersistenceError> {
        let state = self
            .state
            .read()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        Ok(state
            .events
            .iter()
            .filter(|event| event.tx_id == tx_id)
            .cloned()
            .collect())
    }
}

#[derive(Clone)]
pub struct JsonFileBitcoinTxPersistence {
    path: PathBuf,
    io_lock: Arc<Mutex<()>>,
}

#[derive(Default, Serialize, Deserialize)]
struct JsonFileState {
    orchestrations: BTreeMap<String, BtcTxOrchestrationRecord>,
    events: Vec<BtcTxEventRecord>,
}

impl JsonFileBitcoinTxPersistence {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            io_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load_state(&self) -> Result<JsonFileState, PersistenceError> {
        if !self.path.exists() {
            return Ok(JsonFileState::default());
        }

        let raw = fs::read_to_string(&self.path)
            .map_err(|err| PersistenceError::Io(format!("{}: {err}", self.path.display())))?;

        if raw.trim().is_empty() {
            return Ok(JsonFileState::default());
        }

        serde_json::from_str::<JsonFileState>(&raw).map_err(|err| {
            PersistenceError::Serialization(format!("{}: {err}", self.path.display()))
        })
    }

    fn write_state(&self, state: &JsonFileState) -> Result<(), PersistenceError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                PersistenceError::Io(format!("failed to create {}: {err}", parent.display()))
            })?;
        }

        let payload = serde_json::to_string_pretty(state).map_err(|err| {
            PersistenceError::Serialization(format!("{}: {err}", self.path.display()))
        })?;

        let temp_path = self.path.with_extension("tmp");
        fs::write(&temp_path, payload)
            .map_err(|err| PersistenceError::Io(format!("{}: {err}", temp_path.display())))?;
        fs::rename(&temp_path, &self.path).map_err(|err| {
            PersistenceError::Io(format!(
                "failed to move {} to {}: {err}",
                temp_path.display(),
                self.path.display()
            ))
        })?;
        Ok(())
    }

    fn append_event_in_state(
        state: &mut JsonFileState,
        event: BtcTxEventRecord,
    ) -> Result<AppendEventOutcome, PersistenceError> {
        let lookup = state
            .events
            .iter()
            .find(|existing| {
                existing.tx_id == event.tx_id && existing.idempotency_key == event.idempotency_key
            })
            .cloned();

        if let Some(existing) = lookup {
            if existing.fingerprint == event.fingerprint {
                return Ok(AppendEventOutcome::Duplicate(existing));
            }

            return Err(PersistenceError::IdempotencyConflict {
                tx_id: event.tx_id,
                idempotency_key: event.idempotency_key,
                existing_fingerprint: existing.fingerprint,
                incoming_fingerprint: event.fingerprint,
            });
        }

        state.events.push(event);
        Ok(AppendEventOutcome::Inserted)
    }
}

impl BitcoinTxPersistence for JsonFileBitcoinTxPersistence {
    fn upsert_orchestration(
        &self,
        record: BtcTxOrchestrationRecord,
    ) -> Result<(), PersistenceError> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        let mut state = self.load_state()?;
        state.orchestrations.insert(record.tx_id.clone(), record);
        self.write_state(&state)
    }

    fn append_event(
        &self,
        event: BtcTxEventRecord,
    ) -> Result<AppendEventOutcome, PersistenceError> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        let mut state = self.load_state()?;
        let outcome = Self::append_event_in_state(&mut state, event)?;
        if matches!(outcome, AppendEventOutcome::Inserted) {
            self.write_state(&state)?;
        }
        Ok(outcome)
    }

    fn get_orchestration(
        &self,
        tx_id: &str,
    ) -> Result<Option<BtcTxOrchestrationRecord>, PersistenceError> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        let state = self.load_state()?;
        Ok(state.orchestrations.get(tx_id).cloned())
    }

    fn list_orchestrations(&self) -> Result<Vec<BtcTxOrchestrationRecord>, PersistenceError> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        let state = self.load_state()?;
        Ok(state.orchestrations.values().cloned().collect())
    }

    fn list_events(&self, tx_id: &str) -> Result<Vec<BtcTxEventRecord>, PersistenceError> {
        let _guard = self
            .io_lock
            .lock()
            .map_err(|err| PersistenceError::Io(format!("lock poisoned: {err}")))?;
        let state = self.load_state()?;
        Ok(state
            .events
            .iter()
            .filter(|event| event.tx_id == tx_id)
            .cloned()
            .collect())
    }
}
