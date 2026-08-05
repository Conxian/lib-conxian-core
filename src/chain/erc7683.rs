//! ERC-7683 Cross-Chain Intent Mapping
//!
//! Implements the [ERC-7683](https://eips.ethereum.org/EIPS/eip-7683) standard
//! for cross-chain order expression. Provides full bidirectional mapping between
//! Conxian's [`CrossChainIntent`] (SDK) and the ERC-7683 wire format.
//!
//! ## ERC-7683 Key Fields
//!
//! | ERC-7683 Field | Conxian Mapping |
//! |---|---|
//! | `settlementContract` | `AssetIdentifier.contract_address` |
//! | `swapper` | `recipient` (signer identity) |
//! | `nonce` | Derived from `BitcoinPSBT.txid` |
//! | `originChainId` | `Chain::chain_id()` |
//! | `initiateDeadline` | `ResolvedCrossChainOrder.initiateDeadline` |
//! | `fillDeadline` | `ResolvedCrossChainOrder.fillDeadline` |
//! | `orderData` | `CrossChainIntent::to_order_data()` |

use conxius_enclave_sdk::protocol::chain_abstraction::Chain;
use conxius_enclave_sdk::protocol::intent::CrossChainIntent;
use serde::{Deserialize, Serialize};

/// Full ERC-7683 cross-chain order, compatible with the Solidity `CrossChainOrder` struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erc7683CrossChainOrder {
    /// Contract address that settles the order on the destination chain.
    pub settlement_contract: String,
    /// Address of the entity signing the intent (maps to Conxian `recipient`).
    pub swapper: String,
    /// Anti-replay nonce, derived from settlement-system sequence or PSBT txid.
    pub nonce: u64,
    /// Chain ID of the originating chain (EIP-155 for EVM; custom for non-EVM).
    pub origin_chain_id: u64,
    /// Timestamp (Unix seconds) after which the order cannot be initiated.
    pub initiate_deadline: u64,
    /// Timestamp (Unix seconds) after which the order cannot be filled.
    pub fill_deadline: u64,
    /// Chain-specific encoded order data (Conxian uses CBOR/JSON for this field).
    pub order_data: Vec<u8>,
}

impl Erc7683CrossChainOrder {
    /// Convert a Conxian [`CrossChainIntent`] into an ERC-7683 order.
    pub fn from_cross_chain_intent(
        intent: &CrossChainIntent,
        settlement_contract: String,
        swapper: String,
        nonce: u64,
        origin_chain_id: u64,
        initiate_deadline: u64,
        fill_deadline: u64,
    ) -> Self {
        Self {
            settlement_contract,
            swapper,
            nonce,
            origin_chain_id,
            initiate_deadline,
            fill_deadline,
            order_data: intent.to_order_data(),
        }
    }

    /// Extract a Conxian [`CrossChainIntent`] from the `order_data` payload.
    pub fn to_cross_chain_intent(&self) -> Option<CrossChainIntent> {
        serde_json::from_slice(&self.order_data).ok()
    }

    /// Validate that the order has not expired.
    pub fn is_initiable(&self, current_time_secs: u64) -> bool {
        current_time_secs <= self.initiate_deadline
    }

    /// Validate that the order can still be filled.
    pub fn is_fillable(&self, current_time_secs: u64) -> bool {
        current_time_secs <= self.fill_deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conxius_enclave_sdk::protocol::chain_abstraction::Chain;
    use conxius_enclave_sdk::protocol::intent::{AssetIdentifier, CrossChainIntent};

    fn sample_intent() -> CrossChainIntent {
        CrossChainIntent {
            input_asset: AssetIdentifier {
                chain: Chain::Bitcoin,
                contract_address: None,
                token_id: Some("btc".into()),
            },
            output_asset: AssetIdentifier {
                chain: Chain::Stacks,
                contract_address: Some("SP2PABAF9...".into()),
                token_id: Some("xUSD".into()),
            },
            input_amount: 1_000_000,
            output_amount: 25_000_000,
            destination_chain: Chain::Stacks,
            recipient: "SP2PABAF9...".into(),
        }
    }

    #[test]
    fn roundtrip_intent_through_erc7683() {
        let intent = sample_intent();
        let order = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "0xSettlementContract".into(),
            "SP2PABAF9...".into(),
            42, 1, 2000000000, 2100000000,
        );

        let recovered = order.to_cross_chain_intent().unwrap();
        assert_eq!(recovered.input_amount, intent.input_amount);
        assert_eq!(recovered.recipient, intent.recipient);
    }

    #[test]
    fn deadlines_enforced() {
        let intent = sample_intent();
        let order = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent, "0xSettlement".into(), "0xSwapper".into(), 1, 1, 100, 200,
        );
        assert!(order.is_initiable(50));
        assert!(!order.is_initiable(150));
        assert!(order.is_fillable(150));
        assert!(!order.is_fillable(250));
    }

    #[test]
    fn invalid_order_data_returns_none() {
        let order = Erc7683CrossChainOrder {
            settlement_contract: "0x".into(),
            swapper: "0x".into(),
            nonce: 0,
            origin_chain_id: 0,
            initiate_deadline: 0,
            fill_deadline: 0,
            order_data: b"not-valid-json".to_vec(),
        };
        assert!(order.to_cross_chain_intent().is_none());
    }
}
