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

impl TransportCapability {
    /// Validates capability bounds, ensuring a non-empty chain ID and non-empty features vector.
    pub fn validate(&self) -> Result<(), TransportError> {
        if self.chain_id.trim().is_empty() {
            return Err(TransportError::InvalidInput(
                "capability chain_id cannot be empty or whitespace".into(),
            ));
        }
        if self.features.is_empty() {
            return Err(TransportError::InvalidInput(
                "capability features vector cannot be empty".into(),
            ));
        }
        Ok(())
    }
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
        if chain_id.trim().is_empty() {
            return false;
        }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoEntry {
    pub txid: String,
    pub vout: u32,
    pub value_sats: u64,
    pub script_pubkey: String,
    pub confirmations: u64,
}

/// Errors that transport backends may return.
#[derive(Debug, PartialEq, Eq)]
pub enum TransportError {
    /// Invalid input parameter or boundary constraint failure.
    InvalidInput(String),
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
            Self::InvalidInput(msg) => write!(f, "invalid transport input: {msg}"),
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

    fn broadcast(&self, chain_id: &str, raw_tx: &[u8]) -> Result<String, TransportError> {
        if chain_id.trim().is_empty() {
            return Err(TransportError::InvalidInput(
                "chain_id cannot be empty or whitespace".into(),
            ));
        }
        if raw_tx.is_empty() {
            return Err(TransportError::InvalidInput(
                "raw_tx payload cannot be empty".into(),
            ));
        }
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
        address: &str,
    ) -> Result<Vec<UtxoEntry>, TransportError> {
        if chain_id.trim().is_empty() {
            return Err(TransportError::InvalidInput(
                "chain_id cannot be empty or whitespace".into(),
            ));
        }
        if address.trim().is_empty() {
            return Err(TransportError::InvalidInput(
                "address cannot be empty or whitespace".into(),
            ));
        }
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
        let result = transport.broadcast("stacks:mainnet", b"raw_tx");
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
    #[test]
    fn capability_validation_fails_closed_on_invalid_inputs() {
        let empty_chain = TransportCapability {
            chain_id: "  ".into(),
            features: vec![TransportFeature::Broadcast],
        };
        assert!(matches!(
            empty_chain.validate(),
            Err(TransportError::InvalidInput(_))
        ));

        let empty_features = TransportCapability {
            chain_id: "bitcoin:mainnet".into(),
            features: vec![],
        };
        assert!(matches!(
            empty_features.validate(),
            Err(TransportError::InvalidInput(_))
        ));

        let valid_capability = TransportCapability {
            chain_id: "bitcoin:mainnet".into(),
            features: vec![TransportFeature::Broadcast],
        };
        assert!(valid_capability.validate().is_ok());
    }

    #[test]
    fn transport_adapter_rejects_empty_or_invalid_inputs() {
        let transport = MockTransport::new();

        // Broadcast parameter checks
        assert_eq!(
            transport.broadcast("", b"raw_tx"),
            Err(TransportError::InvalidInput("chain_id cannot be empty or whitespace".into()))
        );
        assert_eq!(
            transport.broadcast("bitcoin:mainnet", b""),
            Err(TransportError::InvalidInput("raw_tx payload cannot be empty".into()))
        );

        // Query UTXOs parameter checks
        assert_eq!(
            transport.query_utxos("  ", "bc1qtest"),
            Err(TransportError::InvalidInput("chain_id cannot be empty or whitespace".into()))
        );
        assert_eq!(
            transport.query_utxos("bitcoin:mainnet", ""),
            Err(TransportError::InvalidInput("address cannot be empty or whitespace".into()))
        );
    }

    #[test]
    fn transport_error_display_formatting() {
        let err = TransportError::InvalidInput("test error".into());
        assert_eq!(format!("{err}"), "invalid transport input: test error");
    }

}
