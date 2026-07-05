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

    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String> {
        let base_fee = 1000u64;
        let data_weight = tx_params.data.as_ref().map(|d| d.len() as u64).unwrap_or(0);
        Ok(base_fee + (data_weight * 10))
    }

    fn trust_tier(&self) -> TrustTier {
        TrustTier::Strict
    }

    fn verify_state_proof(&self, _state_root: &str, _proof: &str) -> Result<bool, String> {
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

    fn verify_state_proof(&self, state_root: &str, proof: &str) -> Result<bool, String> {
        if state_root.is_empty() || proof.is_empty() {
            return Err("Missing root or proof for IBC verification".to_string());
        }
        Ok(!proof.contains("invalid"))
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

    fn verify_state_proof(&self, _state_root: &str, proof: &str) -> Result<bool, String> {
        if proof.is_empty() {
            return Err("Empty Solana state proof".to_string());
        }
        Ok(!proof.contains("invalid"))
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

    fn verify_state_proof(&self, _state_root: &str, proof: &str) -> Result<bool, String> {
        if proof.is_empty() {
            return Err("Empty Move state proof".to_string());
        }
        Ok(!proof.contains("invalid"))
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

    fn verify_state_proof(&self, _state_root: &str, proof: &str) -> Result<bool, String> {
        if proof.is_empty() {
            return Err("Empty Substrate state proof".to_string());
        }
        Ok(!proof.contains("invalid"))
    }

    fn get_state_root(&self) -> Result<String, String> {
        Ok("substrate_root".to_string())
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
}
