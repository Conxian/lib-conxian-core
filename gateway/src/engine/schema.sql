-- CON-717 persistence model
-- Canonical orchestration projection table.
CREATE TABLE IF NOT EXISTS btc_tx_orchestration (
    tx_id TEXT PRIMARY KEY,
    state TEXT NOT NULL,
    latest_transition TEXT,
    latest_event_id TEXT,
    fee_rate_sat_vb BIGINT,
    attempt INTEGER NOT NULL DEFAULT 0,
    observed_confirmations INTEGER,
    recovery_cursor BIGINT NOT NULL,
    updated_at_epoch_ms BIGINT NOT NULL
);

-- Append-only lifecycle transition/event log.
CREATE TABLE IF NOT EXISTS btc_tx_events (
    event_id TEXT PRIMARY KEY,
    tx_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    transition TEXT NOT NULL,
    from_state TEXT NOT NULL,
    to_state TEXT NOT NULL,
    attempt INTEGER NOT NULL DEFAULT 0,
    fee_rate_sat_vb BIGINT,
    observed_confirmations INTEGER,
    rationale TEXT,
    fingerprint TEXT NOT NULL,
    created_at_epoch_ms BIGINT NOT NULL,
    UNIQUE (tx_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_btc_tx_events_tx_id_created
    ON btc_tx_events (tx_id, created_at_epoch_ms);

CREATE INDEX IF NOT EXISTS idx_btc_tx_events_transition
    ON btc_tx_events (transition);
