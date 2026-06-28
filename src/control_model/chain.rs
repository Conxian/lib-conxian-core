use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Bitcoin,
    Stacks,
    Liquid,
    Lightning,
    Babylon,
    Bob,
    Mezo,
    Citrea,
    Botanix,
    Ethereum,
    Base,
    Arbitrum,
    Optimism,
    Polygon,
    CosmosHub,
    Osmosis,
    Celestia,
    Solana,
    Eclipse,
}

pub enum BitcoinFeeBumpReason {
    PolicyAged,
    PolicyStuck,
    ManualIntervention,
    NetworkCongestion,
}
/// Tier 1, 2, and 3 chain families for universal support (ADR-006).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    /// Bitcoin/UTXO: Native, Stacks, Liquid, Babylon, BOB, Mezo.
    BitcoinUtxo,
    /// EVM: Ethereum, Base, Arbitrum, Optimism, Polygon, Botanix.
    Evm,
    /// Cosmos/IBC: Cosmos Hub, Osmosis, Celestia.
    CosmosIbc,
    /// Solana/SVM: Solana, Eclipse.
    SolanaSvm,
    /// Move: Sui, Aptos.
    Move,
    /// Substrate: Polkadot, Kusama.
    Substrate,
}
#[cfg(test)]
mod universal_chain_tests {
    use super::*;
    use crate::control_model::BridgeSystem;

    #[test]
    fn test_chain_family_variants() {
        let families = [
            ChainFamily::BitcoinUtxo,
            ChainFamily::Evm,
            ChainFamily::CosmosIbc,
            ChainFamily::SolanaSvm,
            ChainFamily::Move,
            ChainFamily::Substrate,
        ];
        assert_eq!(families.len(), 6);
    }

    #[test]
    fn test_bridge_system_expansion() {
        let systems = [
            BridgeSystem::ChainlinkCcip,
            BridgeSystem::NearChainSignatures,
            BridgeSystem::CircleCctp,
            BridgeSystem::NexusZkVM,
            BridgeSystem::Bitvm2,
        ];
        assert_eq!(systems.len(), 5);
    }

    #[test]
    fn test_chain_enum_variants() {
        let chains = [
            Chain::Babylon,
            Chain::Bob,
            Chain::Mezo,
            Chain::Citrea,
            Chain::Botanix,
            Chain::Ethereum,
            Chain::Base,
        ];
        assert!(chains.len() >= 7);
    }
}
