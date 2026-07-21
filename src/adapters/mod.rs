use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxParams {
    pub amount_sats: u64,
    pub destination: String,
    pub data: Option<Vec<u8>>,
}

/// Typed failures for adapter-level state-proof operations.
///
/// The core crate owns the adapter contract, but it does not own chain RPC,
/// light-client, or consensus-proof backends. Adapters therefore reject
/// evidence they cannot verify instead of treating a plausible string as
/// authoritative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateProofError {
    /// The caller did not provide the state root required for verification.
    MissingStateRoot,
    /// The supplied proof cannot be parsed as the adapter's declared input.
    InvalidProof { reason: String },
    /// The input shape is understood, but this core adapter has no verifier.
    Unsupported { chain: Chain, reason: String },
    /// A verified state source is not available in this library boundary.
    Unavailable { chain: Chain, reason: String },
    /// Parsed evidence names a different state root than the request.
    MismatchedStateRoot { expected: String, actual: String },
}

impl fmt::Display for StateProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStateRoot => write!(f, "state proof verification requires a state root"),
            Self::InvalidProof { reason } => write!(f, "invalid state proof: {reason}"),
            Self::Unsupported { chain, reason } => {
                write!(f, "unsupported state proof for {chain:?}: {reason}")
            }
            Self::Unavailable { chain, reason } => {
                write!(f, "state proof source unavailable for {chain:?}: {reason}")
            }
            Self::MismatchedStateRoot { expected, actual } => write!(
                f,
                "state proof root mismatch: expected {expected}, got {actual}"
            ),
        }
    }
}

impl std::error::Error for StateProofError {}

fn reject_unverified_state_proof(
    chain: Chain,
    state_root: &str,
    proof: &str,
) -> Result<bool, StateProofError> {
    if state_root.trim().is_empty() {
        return Err(StateProofError::MissingStateRoot);
    }
    if proof.trim().is_empty() {
        return Err(StateProofError::InvalidProof {
            reason: "proof must not be empty".to_string(),
        });
    }

    Err(StateProofError::Unsupported {
        chain,
        reason: "no cryptographic state-proof backend is wired into core".to_string(),
    })
}

fn unavailable_state_root(chain: Chain) -> Result<String, StateProofError> {
    Err(StateProofError::Unavailable {
        chain,
        reason: "state roots must come from a verified downstream source".to_string(),
    })
}

/// CXIP-21: Universal interface for chain-specific orchestration.
/// This trait allows the Vault SDK to support multiple ecosystems with uniform risk enforcement.
pub trait UniversalChainAdapter {
    /// Returns the chain family (e.g., Bitcoin, EVM).
    fn family(&self) -> ChainFamily;

    /// Returns the specific chain identifier.
    fn chain(&self) -> Chain;

    /// Validates an address for the target chain.
    fn validate_address(&self, address: &str) -> Result<(), String>;

    /// Estimates the fee for a transaction.
    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String>;

    /// Returns the trust tier of the chain's bridge/messaging lane.
    fn trust_tier(&self) -> TrustTier;

    /// Verifies a state proof for the chain (e.g., light client or ZK proof).
    ///
    /// The core adapters do not acquire or verify chain evidence themselves.
    /// They return a typed failure until a downstream verifier is wired.
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError>;

    /// Retrieves the current state root from a verified source.
    fn get_state_root(&self) -> Result<String, StateProofError>;
}

/// Adapter for the Bitcoin network, providing native UTXO-based support.
pub struct BitcoinAdapter;

impl UniversalChainAdapter for BitcoinAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::BitcoinUtxo
    }

    fn chain(&self) -> Chain {
        Chain::Bitcoin
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.starts_with("bc1") || address.starts_with("1") || address.starts_with("3") {
            Ok(())
        } else {
            Err("Invalid Bitcoin address".to_string())
        }
    }

    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String> {
        let base_fee = 1000u64;
        let data_weight = tx_params.data.as_ref().map(|d| d.len() as u64).unwrap_or(0);
        Ok(base_fee + (data_weight * 10))
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(Chain::Bitcoin, state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(Chain::Bitcoin)
    }
}

/// Adapter for EVM-compatible networks (Ethereum, Base, etc.).
pub struct EvmAdapter {
    pub chain: Chain,
}

impl UniversalChainAdapter for EvmAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::Evm
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.starts_with("0x") && address.len() == 42 {
            Ok(())
        } else {
            Err("Invalid EVM address".to_string())
        }
    }

    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String> {
        let base_gas = 21000u64;
        let data_gas = tx_params
            .data
            .as_ref()
            .map(|d| d.len() as u64 * 16)
            .unwrap_or(0);
        Ok(base_gas + data_gas)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(self.chain.clone(), state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(self.chain.clone())
    }
}

/// Adapter for Cosmos-based networks using IBC for trust-minimized communication.
pub struct CosmosAdapter {
    pub chain: Chain,
}

impl UniversalChainAdapter for CosmosAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::CosmosIbc
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if (address.starts_with("cosmos") || address.starts_with("osmo")) && address.len() >= 39 {
            Ok(())
        } else {
            Err("Invalid Cosmos/IBC address: must be bech32 with valid prefix".to_string())
        }
    }

    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String> {
        let ibc_fixed_cost = 1000u64;
        let data_cost = tx_params
            .data
            .as_ref()
            .map(|d| d.len() as u64 * 5)
            .unwrap_or(0);
        Ok(ibc_fixed_cost + data_cost)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(self.chain.clone(), state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(self.chain.clone())
    }
}

/// Adapter for Solana and SVM-compatible networks.
pub struct SolanaAdapter;

impl UniversalChainAdapter for SolanaAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::SolanaSvm
    }

    fn chain(&self) -> Chain {
        Chain::Solana
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.len() >= 32
            && address.len() <= 44
            && !address.contains('0')
            && !address.contains('O')
            && !address.contains('I')
            && !address.contains('l')
        {
            Ok(())
        } else {
            Err("Invalid Solana address: must be valid Base58 public key".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(5000)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(Chain::Solana, state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(Chain::Solana)
    }
}

/// Adapter for Move-based networks (Aptos, Sui).
pub struct MoveAdapter {
    pub chain: Chain,
}

impl UniversalChainAdapter for MoveAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::Move
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.starts_with("0x") && (address.len() == 66 || address.len() == 64) {
            Ok(())
        } else {
            Err(
                "Invalid Move address: expected 0x followed by 64 hex characters (Aptos/Sui)"
                    .to_string(),
            )
        }
    }

    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String> {
        let base_fee = 1000u64;
        let storage_fee = tx_params
            .data
            .as_ref()
            .map(|d| d.len() as u64 * 2)
            .unwrap_or(0);
        Ok(base_fee + storage_fee)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(self.chain.clone(), state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(self.chain.clone())
    }
}

/// Adapter for Substrate-based networks (Polkadot, Kusama).
pub struct SubstrateAdapter {
    pub chain: Chain,
}

impl UniversalChainAdapter for SubstrateAdapter {
    fn family(&self) -> ChainFamily {
        ChainFamily::Substrate
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }

    fn validate_address(&self, address: &str) -> Result<(), String> {
        if address.len() >= 47 && address.len() <= 49 {
            Ok(())
        } else {
            Err("Invalid Substrate address: expected SS58 format (47-49 chars)".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(100)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, StateProofError> {
        reject_unverified_state_proof(self.chain.clone(), state_root, proof)
    }

    fn get_state_root(&self) -> Result<String, StateProofError> {
        unavailable_state_root(self.chain.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_adapter_fee() {
        let adapter = BitcoinAdapter;
        let tx = TxParams {
            amount_sats: 100_000,
            destination: "bc1q".to_string(),
            data: Some(vec![0u8; 100]),
        };
        assert_eq!(adapter.estimate_fee(&tx).unwrap(), 2000);
    }

    #[test]
    fn test_evm_adapter_fee() {
        let adapter = EvmAdapter {
            chain: Chain::Ethereum,
        };
        let tx = TxParams {
            amount_sats: 100_000,
            destination: "0x".to_string(),
            data: Some(vec![0u8; 10]),
        };
        assert_eq!(adapter.estimate_fee(&tx).unwrap(), 21160);
    }

    #[test]
    fn test_cosmos_adapter_validation() {
        let adapter = CosmosAdapter {
            chain: Chain::CosmosHub,
        };
        assert!(adapter
            .validate_address("cosmos1q...long_enough_address_39_chars")
            .is_ok());
        assert!(adapter.validate_address("too_short").is_err());
    }

    #[test]
    fn test_solana_adapter_validation() {
        let adapter = SolanaAdapter;
        assert!(adapter
            .validate_address("H6AR6iE_not_really_base58_but_right_length")
            .is_err());
        assert!(adapter
            .validate_address("H6AR6iE78245782457824578245782457824578")
            .is_ok());
    }

    #[test]
    fn test_move_adapter_validation() {
        let adapter = MoveAdapter {
            chain: Chain::Aptos,
        };
        let long_addr = format!("0x{}", "a".repeat(64));
        assert!(adapter.validate_address(&long_addr).is_ok());
    }

    #[test]
    fn test_state_proof_adapters_fail_closed_for_unverified_evidence() {
        let adapters: Vec<Box<dyn UniversalChainAdapter>> = vec![
            Box::new(BitcoinAdapter),
            Box::new(EvmAdapter {
                chain: Chain::Ethereum,
            }),
            Box::new(CosmosAdapter {
                chain: Chain::CosmosHub,
            }),
            Box::new(SolanaAdapter),
            Box::new(MoveAdapter {
                chain: Chain::Aptos,
            }),
            Box::new(SubstrateAdapter {
                chain: Chain::Polkadot,
            }),
        ];

        for adapter in adapters {
            assert!(matches!(
                adapter.verify_state_proof("root", "mutated-proof"),
                Err(StateProofError::Unsupported { .. })
            ));
            assert!(matches!(
                adapter.verify_state_proof("", "mutated-proof"),
                Err(StateProofError::MissingStateRoot)
            ));
            assert!(matches!(
                adapter.verify_state_proof("root", ""),
                Err(StateProofError::InvalidProof { .. })
            ));
            assert!(matches!(
                adapter.get_state_root(),
                Err(StateProofError::Unavailable { .. })
            ));
        }
    }
}
