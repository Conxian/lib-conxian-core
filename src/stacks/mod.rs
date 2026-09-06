//! Decentralized Layer-2 Programmability: Stacks Nakamoto and sBTC
//! Aligned with CXIP 20 Section 8.0 and CON-709 (Pilot Lane)

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StacksError {
    InvalidTransaction,
    InvalidAddress,
    FinalityTimeout,
    MalformedFinalityEvidence,
    UnsupportedFinalityEvidence,
    StatusUnavailable,
    UnknownIntent,
    PegInFailed(String),
    PegOutFailed(String),
    SignerCoordinationError(String),
}

impl fmt::Display for StacksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => write!(f, "Invalid Stacks or Bitcoin transaction"),
            Self::InvalidAddress => write!(f, "Invalid Stacks address"),
            Self::FinalityTimeout => write!(f, "Transaction finality timeout"),
            Self::MalformedFinalityEvidence => write!(f, "Malformed Bitcoin finality evidence"),
            Self::UnsupportedFinalityEvidence => {
                write!(f, "Bitcoin finality evidence is unsupported in Core")
            }
            Self::StatusUnavailable => write!(f, "sBTC status evidence is unavailable"),
            Self::UnknownIntent => write!(f, "Unknown or empty sBTC intent"),
            Self::PegInFailed(msg) => write!(f, "Peg-in failed: {msg}"),
            Self::PegOutFailed(msg) => write!(f, "Peg-out failed: {msg}"),
            Self::SignerCoordinationError(msg) => write!(f, "Signer coordination error: {msg}"),
        }
    }
}

impl std::error::Error for StacksError {}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SBTCState {
    Pending,
    BitcoinConfirmed,
    SignersNotified,
    PegInMined,
    PegOutMined,
    Finalized,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SBTCIntent {
    pub intent_id: String,
    pub amount_sats: u64,
    pub stacks_address: String,
    pub bitcoin_txid: Option<String>,
    pub state: SBTCState,
    pub created_at_epoch: u64,
}

impl SBTCIntent {
    /// Enforces fail-closed parameter validation for sBTC intents.
    pub fn validate(&self) -> Result<(), StacksError> {
        if self.intent_id.trim().is_empty() {
            return Err(StacksError::UnknownIntent);
        }
        if self.amount_sats == 0 {
            return Err(StacksError::InvalidTransaction);
        }
        if self.stacks_address.trim().is_empty() {
            return Err(StacksError::InvalidAddress);
        }
        if let Some(ref txid) = self.bitcoin_txid {
            if txid.trim().is_empty() {
                return Err(StacksError::InvalidTransaction);
            }
        }
        Ok(())
    }
}

pub struct StacksNakamoto;

impl StacksNakamoto {
    /// A block number alone is not Bitcoin finality evidence. Core has no
    /// header-chain or transaction proof verifier for this operation.
    pub fn verify_bitcoin_finality_checked(stacks_block: u64) -> Result<bool, StacksError> {
        if stacks_block == 0 {
            return Err(StacksError::MalformedFinalityEvidence);
        }
        Err(StacksError::UnsupportedFinalityEvidence)
    }
}

/// Core interface for the Stacks + sBTC Adapter (CON-709 Pilot).
pub trait StacksAdapter {
    /// Initiates a peg-in (BTC -> sBTC)
    fn initiate_peg_in(&self, amount_sats: u64, btc_txid: &str) -> Result<SBTCIntent, StacksError>;

    /// Initiates a peg-out (sBTC -> BTC)
    fn initiate_peg_out(
        &self,
        amount_sats: u64,
        stacks_address: &str,
    ) -> Result<SBTCIntent, StacksError>;

    /// Verifies the status of an ongoing peg-in/out
    fn get_status(&self, intent_id: &str) -> Result<SBTCState, StacksError>;
}

pub struct SBTCBridge;

impl Default for SBTCBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl SBTCBridge {
    pub fn new() -> Self {
        Self
    }
}

impl StacksAdapter for SBTCBridge {
    fn initiate_peg_in(&self, amount_sats: u64, btc_txid: &str) -> Result<SBTCIntent, StacksError> {
        if amount_sats == 0 || btc_txid.trim().is_empty() {
            return Err(StacksError::InvalidTransaction);
        }

        let intent = SBTCIntent {
            intent_id: format!("sbtc-pegin-{}", btc_txid.trim()),
            amount_sats,
            stacks_address: "ST123...".to_string(), // Placeholder
            bitcoin_txid: Some(btc_txid.trim().to_string()),
            state: SBTCState::BitcoinConfirmed,
            created_at_epoch: 1718363200, // Placeholder
        };

        intent.validate()?;
        Ok(intent)
    }

    fn initiate_peg_out(
        &self,
        amount_sats: u64,
        stacks_address: &str,
    ) -> Result<SBTCIntent, StacksError> {
        if amount_sats == 0 {
            return Err(StacksError::InvalidTransaction);
        }
        if stacks_address.trim().is_empty() {
            return Err(StacksError::InvalidAddress);
        }

        let intent = SBTCIntent {
            intent_id: format!("sbtc-pegout-{}", stacks_address.trim()),
            amount_sats,
            stacks_address: stacks_address.trim().to_string(),
            bitcoin_txid: None,
            state: SBTCState::Pending,
            created_at_epoch: 1718363200, // Placeholder
        };

        intent.validate()?;
        Ok(intent)
    }

    fn get_status(&self, intent_id: &str) -> Result<SBTCState, StacksError> {
        if intent_id.trim().is_empty() {
            return Err(StacksError::UnknownIntent);
        }
        Err(StacksError::StatusUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nakamoto_finality() {
        assert_eq!(
            StacksNakamoto::verify_bitcoin_finality_checked(100),
            Err(StacksError::UnsupportedFinalityEvidence)
        );
        assert_eq!(
            StacksNakamoto::verify_bitcoin_finality_checked(0),
            Err(StacksError::MalformedFinalityEvidence)
        );
    }

    #[test]
    fn test_sbtc_pegin_interface() {
        let bridge = SBTCBridge::new();
        let intent = bridge.initiate_peg_in(1000000, "abc123").unwrap();
        assert_eq!(intent.amount_sats, 1000000);
        assert_eq!(intent.state, SBTCState::BitcoinConfirmed);
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_sbtc_pegout_interface() {
        let bridge = SBTCBridge::new();
        let intent = bridge.initiate_peg_out(500000, "ST_ADDRESS").unwrap();
        assert_eq!(intent.stacks_address, "ST_ADDRESS");
        assert_eq!(intent.state, SBTCState::Pending);
        assert!(intent.validate().is_ok());
    }

    #[test]
    fn test_invalid_btc_txid_and_zero_amount() {
        let bridge = SBTCBridge::new();
        assert_eq!(
            bridge.initiate_peg_in(1000000, ""),
            Err(StacksError::InvalidTransaction)
        );
        assert_eq!(
            bridge.initiate_peg_in(1000000, "   "),
            Err(StacksError::InvalidTransaction)
        );
        assert_eq!(
            bridge.initiate_peg_in(0, "abc123"),
            Err(StacksError::InvalidTransaction)
        );
    }

    #[test]
    fn test_invalid_pegout_parameters() {
        let bridge = SBTCBridge::new();
        assert_eq!(
            bridge.initiate_peg_out(0, "ST_ADDRESS"),
            Err(StacksError::InvalidTransaction)
        );
        assert_eq!(
            bridge.initiate_peg_out(100000, ""),
            Err(StacksError::InvalidAddress)
        );
        assert_eq!(
            bridge.initiate_peg_out(100000, "   "),
            Err(StacksError::InvalidAddress)
        );
    }

    #[test]
    fn test_sbtc_intent_validation() {
        let valid = SBTCIntent {
            intent_id: "intent-1".into(),
            amount_sats: 1000,
            stacks_address: "ST_ADDR".into(),
            bitcoin_txid: Some("tx-1".into()),
            state: SBTCState::Pending,
            created_at_epoch: 100,
        };
        assert!(valid.validate().is_ok());

        let mut zero_amount = valid.clone();
        zero_amount.amount_sats = 0;
        assert_eq!(zero_amount.validate(), Err(StacksError::InvalidTransaction));

        let mut empty_id = valid.clone();
        empty_id.intent_id = "".into();
        assert_eq!(empty_id.validate(), Err(StacksError::UnknownIntent));

        let mut empty_addr = valid.clone();
        empty_addr.stacks_address = "".into();
        assert_eq!(empty_addr.validate(), Err(StacksError::InvalidAddress));

        let mut empty_txid = valid.clone();
        empty_txid.bitcoin_txid = Some("".into());
        assert_eq!(empty_txid.validate(), Err(StacksError::InvalidTransaction));
    }

    #[test]
    fn test_status_requires_authoritative_evidence() {
        let bridge = SBTCBridge::new();
        assert_eq!(
            bridge.get_status("sbtc-intent"),
            Err(StacksError::StatusUnavailable)
        );
        assert_eq!(bridge.get_status(""), Err(StacksError::UnknownIntent));
    }
}
