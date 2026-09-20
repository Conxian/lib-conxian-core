//! Chain Transport Capability Adapters
//!
//! Transport-neutral backend architecture. Core owns deterministic,
//! transport-neutral contracts. Network clients, credentials, provider
//! selection, retries, and persistence are isolated behind these adapters.
//!
//! ## Design
//!
//! ```text
//! Core (transport-neutral)  →  TransportAdapter (trait)  →  Backend impl
//!                                                           ├── EsploraBackend
//!                                                           ├── ElectrumBackend
//!                                                           └── MockBackend (tests)
//! ```
//!
//! Each backend proves its provenance via [`TransportCapability`] so
//! callers can enforce per-trust-tier transport policies.

use serde::{Deserialize, Serialize};

/// Transport capability token — proves a backend can serve a specific chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransportCapability {
    /// Chain identifier (Bitcoin mainnet, Stacks mainnet, etc.).
    pub chain_id: String,
    /// Feature set supported by this transport.
    pub features: Vec<TransportFeature>,
}

/// Features a transport backend may support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportFeature {
    /// Full-node UTXO set queries.
    UtxoLookup,
    /// Transaction broadcast.
    Broadcast,
    /// Historical transaction retrieval.
    History,
    /// Mempool monitoring.
    Mempool,
    /// Fee estimation.
    FeeEstimation,
    /// SPV verification (merkle proofs).
    SpvVerification,
}

/// A transport adapter that routes chain interactions to a concrete backend.
///
/// Implementations prove their capabilities at construction time and
/// fail-closed when asked for unsupported features.
pub trait TransportAdapter: Send + Sync {
    /// Return the capabilities this backend provides.
    fn capabilities(&self) -> &[TransportCapability];

    /// Check whether a specific feature is available for a chain.
    fn supports(&self, chain_id: &str, feature: TransportFeature) -> bool {
        self.capabilities()
            .iter()
            .any(|c| c.chain_id == chain_id && c.features.contains(&feature))
    }

    /// Broadcast a raw transaction to the chain.
    ///
    /// Returns `Err` if the backend doesn't support broadcast or the
    /// broadcast fails.
    fn broadcast(&self, chain_id: &str, raw_tx: &[u8]) -> Result<String, TransportError>;

    /// Query UTXOs for an address.
    fn query_utxos(&self, chain_id: &str, address: &str) -> Result<Vec<UtxoEntry>, TransportError>;
}

/// A UTXO entry returned by a transport backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoEntry {
    pub txid: String,
    pub vout: u32,
    pub value_sats: u64,
    pub script_pubkey: String,
    pub confirmations: u64,
}

/// Errors that transport backends may return.
#[derive(Debug)]
pub enum TransportError {
    /// Requested feature is not supported by this backend for this chain.
    UnsupportedFeature(TransportFeature, String),
    /// Backend is not reachable.
    BackendUnavailable(String),
    /// Chain is not configured in this backend.
    ChainNotSupported(String),
    /// Generic transport-level error.
    Transport(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFeature(feature, chain) => {
                write!(f, "unsupported feature '{feature:?}' for chain '{chain}'")
            }
            Self::BackendUnavailable(msg) => write!(f, "backend unavailable: {msg}"),
            Self::ChainNotSupported(msg) => write!(f, "chain not supported: {msg}"),
            Self::Transport(msg) => write!(f, "transport error: {msg}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// A no-op transport for testing and compile-time verification.
pub struct MockTransport {
    capabilities: Vec<TransportCapability>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            capabilities: vec![TransportCapability {
                chain_id: "bitcoin:mainnet".into(),
                features: vec![
                    TransportFeature::UtxoLookup,
                    TransportFeature::Broadcast,
                    TransportFeature::History,
                    TransportFeature::FeeEstimation,
                ],
            }],
        }
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportAdapter for MockTransport {
    fn capabilities(&self) -> &[TransportCapability] {
        &self.capabilities
    }

    fn broadcast(&self, chain_id: &str, _raw_tx: &[u8]) -> Result<String, TransportError> {
        if !self.supports(chain_id, TransportFeature::Broadcast) {
            return Err(TransportError::UnsupportedFeature(
                TransportFeature::Broadcast,
                chain_id.into(),
            ));
        }
        Ok("mock_txid_0000000000000000000000000000000000000000000000000000000000000000".into())
    }

    fn query_utxos(
        &self,
        chain_id: &str,
        _address: &str,
    ) -> Result<Vec<UtxoEntry>, TransportError> {
        if !self.supports(chain_id, TransportFeature::UtxoLookup) {
            return Err(TransportError::UnsupportedFeature(
                TransportFeature::UtxoLookup,
                chain_id.into(),
            ));
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_transport_supports_bitcoin() {
        let transport = MockTransport::new();
        assert!(transport.supports("bitcoin:mainnet", TransportFeature::Broadcast));
        assert!(!transport.supports("stacks:mainnet", TransportFeature::Broadcast));
    }

    #[test]
    fn unsupported_feature_returns_error() {
        let transport = MockTransport::new();
        let result = transport.broadcast("stacks:mainnet", b"");
        assert!(matches!(
            result,
            Err(TransportError::UnsupportedFeature(_, _))
        ));
    }

    #[test]
    fn broadcast_returns_mock_txid() {
        let transport = MockTransport::new();
        let txid = transport.broadcast("bitcoin:mainnet", b"raw_tx").unwrap();
        assert!(txid.starts_with("mock_txid_"));
    }
}
