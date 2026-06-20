use crate::control_model::{Chain, ChainFamily, TrustTier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxParams {
    pub amount_sats: u64,
    pub destination: String,
    pub data: Option<Vec<u8>>,
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
        // Skeletal implementation for now
        if address.starts_with("bc1") || address.starts_with("1") || address.starts_with("3") {
            Ok(())
        } else {
            Err("Invalid Bitcoin address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(1000) // 1000 sats dummy fee
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }
}

/// Adapter for EVM-compatible networks (Ethereum, Base, etc.).
pub struct EvmAdapter {
    /// The specific EVM chain this adapter instance represents.
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

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(21000) // 21k gas dummy
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_adapter() {
        let adapter = BitcoinAdapter;
        assert_eq!(adapter.family(), ChainFamily::BitcoinUtxo);
        assert!(adapter.validate_address("bc1q_safe").is_ok());
        assert!(adapter.validate_address("0x123").is_err());
    }

    #[test]
    fn test_evm_adapter() {
        let adapter = EvmAdapter {
            chain: Chain::Ethereum,
        };
        assert_eq!(adapter.family(), ChainFamily::Evm);
        assert!(adapter
            .validate_address("0x71C7656EC7ab88b098defB751B7401B5f6d8976F")
            .is_ok());
        assert!(adapter.validate_address("bc1q").is_err());
    }
}
