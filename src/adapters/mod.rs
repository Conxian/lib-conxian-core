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

    /// Verifies a state proof for the chain (e.g., light client or ZK proof).
    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, String>;

    /// Retrieves the current state root from a verified source.
    fn get_state_root(&self) -> Result<String, String>;
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

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(1000) // 1000 sats dummy fee
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        // Bitcoin is L1, so "state proof" is typically SPV or full node validation
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("btc_l1_root".to_string())
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

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(21000) // 21k gas dummy
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("evm_root".to_string())
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
        if address.starts_with("cosmos") && address.len() > 6 {
            Ok(())
        } else {
            Err("Invalid Cosmos address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(500) // IBC-specific dummy fee
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        // Logic for IBC light client verification would go here
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("cosmos_ibc_root".to_string())
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
        if address.len() >= 32 && address.len() <= 44 {
            Ok(())
        } else {
            Err("Invalid Solana address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(5000)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("solana_root".to_string())
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
        if address.starts_with("0x") && address.len() == 66 {
            Ok(())
        } else {
            Err("Invalid Move address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(1000)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Managed
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("move_root".to_string())
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
        if address.len() >= 47 {
            Ok(())
        } else {
            Err("Invalid Substrate address".to_string())
        }
    }

    fn estimate_fee(&self, _tx_params: &TxParams) -> Result<u64, String> {
        Ok(100)
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
        Ok(true)
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("substrate_root".to_string())
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
        assert!(adapter.verify_state_proof("root", "proof").is_ok());
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
    }
}
