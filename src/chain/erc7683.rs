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

use conxius_enclave_sdk::protocol::intent::{CrossChainIntent, ResolvedCrossChainOrder};
use serde::{Deserialize, Serialize};

/// Error conditions encountered during ERC-7683 order validation and conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erc7683Error {
    /// The settlement contract address is empty or whitespace.
    EmptySettlementContract,
    /// The swapper address is empty or whitespace.
    EmptySwapper,
    /// The open deadline is zero or past the fill deadline.
    InvalidDeadlines { open_deadline: u32, fill_deadline: u32 },
    /// The order data payload is empty.
    EmptyOrderData,
    /// The order data payload cannot be deserialized as a valid [`CrossChainIntent`].
    InvalidOrderData(String),
}

impl std::fmt::Display for Erc7683Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySettlementContract => write!(f, "settlement contract address cannot be empty"),
            Self::EmptySwapper => write!(f, "swapper address cannot be empty"),
            Self::InvalidDeadlines { open_deadline, fill_deadline } => {
                write!(
                    f,
                    "invalid deadlines: open_deadline ({open_deadline}) must be > 0 and <= fill_deadline ({fill_deadline})"
                )
            }
            Self::EmptyOrderData => write!(f, "order data payload cannot be empty"),
            Self::InvalidOrderData(reason) => write!(f, "invalid order data payload: {reason}"),
        }
    }
}

impl std::error::Error for Erc7683Error {}

/// Full ERC-7683 cross-chain order, compatible with the Solidity `CrossChainOrder` struct.
///
/// This struct mirrors [`ResolvedCrossChainOrder`] from the SDK and adds
/// the `settlement_contract` field required by ERC-7683 for destination-chain
/// settlement routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Erc7683CrossChainOrder {
    /// Contract address that settles the order on the destination chain.
    pub settlement_contract: String,
    /// Address of the entity signing the intent.
    pub swapper: String,
    /// Anti-replay nonce, derived from settlement-system sequence or PSBT txid.
    pub nonce: u64,
    /// Chain ID of the originating chain (EIP-155 for EVM; custom for non-EVM).
    pub origin_chain_id: u32,
    /// Timestamp (Unix seconds) after which the order cannot be initiated.
    pub open_deadline: u32,
    /// Timestamp (Unix seconds) after which the order cannot be filled.
    pub fill_deadline: u32,
    /// Chain-specific encoded order data (Conxian uses JSON for this field).
    pub order_data: Vec<u8>,
}

impl Erc7683CrossChainOrder {
    /// Build an ERC-7683 order from a Conxian [`CrossChainIntent`] and settlement metadata.
    pub fn from_cross_chain_intent(
        intent: &CrossChainIntent,
        settlement_contract: String,
        swapper: String,
        nonce: u64,
        origin_chain_id: u32,
        open_deadline: u32,
        fill_deadline: u32,
    ) -> Self {
        Self {
            settlement_contract,
            swapper,
            nonce,
            origin_chain_id,
            open_deadline,
            fill_deadline,
            order_data: intent.to_order_data(),
        }
    }

    /// Perform fail-closed validation of all structural parameters and internal intent integrity.
    pub fn validate(&self) -> Result<(), Erc7683Error> {
        if self.settlement_contract.trim().is_empty() {
            return Err(Erc7683Error::EmptySettlementContract);
        }
        if self.swapper.trim().is_empty() {
            return Err(Erc7683Error::EmptySwapper);
        }
        if self.open_deadline == 0 || self.open_deadline > self.fill_deadline {
            return Err(Erc7683Error::InvalidDeadlines {
                open_deadline: self.open_deadline,
                fill_deadline: self.fill_deadline,
            });
        }
        if self.order_data.is_empty() {
            return Err(Erc7683Error::EmptyOrderData);
        }

        // Verify that order_data deserializes into a valid CrossChainIntent
        self.to_cross_chain_intent()
            .map(|_| ())
            .ok_or_else(|| Erc7683Error::InvalidOrderData("failed to deserialize CrossChainIntent from order_data".into()))
    }

    /// Convert to the SDK's [`ResolvedCrossChainOrder`].
    pub fn to_resolved_order(&self, user: String) -> ResolvedCrossChainOrder {
        ResolvedCrossChainOrder {
            user,
            origin_chain_id: self.origin_chain_id,
            open_deadline: self.open_deadline,
            fill_deadline: self.fill_deadline,
            swapper: self.swapper.clone(),
            nonce: self.nonce,
            input_assets: vec![],
            output_assets: vec![],
        }
    }

    /// Extract a Conxian [`CrossChainIntent`] from the `order_data` payload.
    pub fn to_cross_chain_intent(&self) -> Option<CrossChainIntent> {
        serde_json::from_slice(&self.order_data).ok()
    }

    /// Validate that the order has not expired for initiation.
    pub fn is_open(&self, current_time_secs: u32) -> bool {
        current_time_secs <= self.open_deadline
    }

    /// Validate that the order can still be filled.
    pub fn is_fillable(&self, current_time_secs: u32) -> bool {
        current_time_secs <= self.fill_deadline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conxius_enclave_sdk::protocol::asset::{AssetIdentifier, Chain};
    use conxius_enclave_sdk::protocol::intent::CrossChainIntent;

    fn sample_intent() -> CrossChainIntent {
        CrossChainIntent {
            input_asset: AssetIdentifier {
                chain: Chain::BITCOIN,
                symbol: "BTC".into(),
            },
            output_asset: AssetIdentifier {
                chain: Chain::STACKS,
                symbol: "xUSD".into(),
            },
            input_amount: 1_000_000,
            output_amount: 25_000_000,
            destination_chain: Chain::STACKS,
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
            42,
            1,
            2000000000,
            2100000000,
        );

        assert!(order.validate().is_ok());
        let recovered = order.to_cross_chain_intent().unwrap();
        assert_eq!(recovered.input_amount, intent.input_amount);
        assert_eq!(recovered.recipient, intent.recipient);
    }

    #[test]
    fn validation_rejects_empty_settlement_contract() {
        let intent = sample_intent();
        let mut order = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "   ".into(),
            "SP2PABAF9...".into(),
            42,
            1,
            1000,
            2000,
        );

        assert_eq!(order.validate(), Err(Erc7683Error::EmptySettlementContract));
    }

    #[test]
    fn validation_rejects_empty_swapper() {
        let intent = sample_intent();
        let mut order = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "0xSettlement".into(),
            "".into(),
            42,
            1,
            1000,
            2000,
        );

        assert_eq!(order.validate(), Err(Erc7683Error::EmptySwapper));
    }

    #[test]
    fn validation_rejects_invalid_deadlines() {
        let intent = sample_intent();

        // Zero open_deadline
        let order_zero = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "0xSettlement".into(),
            "0xSwapper".into(),
            1,
            1,
            0,
            100,
        );
        assert_eq!(
            order_zero.validate(),
            Err(Erc7683Error::InvalidDeadlines {
                open_deadline: 0,
                fill_deadline: 100
            })
        );

        // Inverted deadlines
        let order_inverted = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "0xSettlement".into(),
            "0xSwapper".into(),
            1,
            1,
            200,
            100,
        );
        assert_eq!(
            order_inverted.validate(),
            Err(Erc7683Error::InvalidDeadlines {
                open_deadline: 200,
                fill_deadline: 100
            })
        );
    }

    #[test]
    fn validation_rejects_empty_and_corrupted_order_data() {
        let mut order = Erc7683CrossChainOrder {
            settlement_contract: "0xSettlement".into(),
            swapper: "0xSwapper".into(),
            nonce: 1,
            origin_chain_id: 1,
            open_deadline: 100,
            fill_deadline: 200,
            order_data: vec![],
        };
        assert_eq!(order.validate(), Err(Erc7683Error::EmptyOrderData));

        order.order_data = b"not-valid-json".to_vec();
        assert!(matches!(order.validate(), Err(Erc7683Error::InvalidOrderData(_))));
    }

    #[test]
    fn deadlines_enforced() {
        let intent = sample_intent();
        let order = Erc7683CrossChainOrder::from_cross_chain_intent(
            &intent,
            "0xSettlement".into(),
            "0xSwapper".into(),
            1,
            1,
            100,
            200,
        );
        assert!(order.is_open(50));
        assert!(!order.is_open(150));
        assert!(order.is_fillable(150));
        assert!(!order.is_fillable(250));
    }
}
