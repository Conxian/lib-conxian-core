//! ERC-7683: Cross-Chain Intent Standard
//! Solver selection and bidding logic

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Solver {
    pub id: String,
    pub address: String,
    pub reputation_score: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bid {
    pub solver_id: String,
    pub amount_sats: u64,
    pub estimated_latency_blocks: u32,
    pub fee_sats: u64,
}

/// FDC3 Instrument for financial context integration.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Fdc3Instrument {
    pub ticker: String,
    pub name: Option<String>,
    pub isin: Option<String>,
    pub conxian_asset_id: String,
}

/// Typed errors for intent handling and FDC3 resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentError {
    /// The FDC3 instrument is missing mandatory fields (ticker or asset ID).
    InvalidFdc3Instrument(String),
    /// A solver bid contains invalid parameters (empty ID, zero amount, fee exceeding amount, zero blocks).
    InvalidBid(String),
    /// Requested intent amount is zero or invalid.
    InvalidAmount(u64),
    /// Destination address or payload is empty or whitespace.
    InvalidDestination(String),
    /// Solver ID is empty or whitespace.
    EmptySolverId,
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidFdc3Instrument(reason) => write!(f, "invalid FDC3 instrument: {reason}"),
            Self::InvalidBid(reason) => write!(f, "invalid bid parameters: {reason}"),
            Self::InvalidAmount(amt) => write!(f, "invalid intent amount: {amt} (must be > 0)"),
            Self::InvalidDestination(reason) => write!(f, "invalid intent destination: {reason}"),
            Self::EmptySolverId => write!(f, "solver ID cannot be empty or whitespace"),
        }
    }
}

impl std::error::Error for IntentError {}

impl Bid {
    /// Validates bid parameters according to fail-closed protocol invariants.
    pub fn validate(&self) -> Result<(), IntentError> {
        if self.solver_id.trim().is_empty() {
            return Err(IntentError::EmptySolverId);
        }
        if self.amount_sats == 0 {
            return Err(IntentError::InvalidBid(
                "amount_sats must be greater than zero".to_string(),
            ));
        }
        if self.estimated_latency_blocks == 0 {
            return Err(IntentError::InvalidBid(
                "estimated_latency_blocks must be at least 1".to_string(),
            ));
        }
        if self.fee_sats > self.amount_sats {
            return Err(IntentError::InvalidBid(format!(
                "fee_sats ({}) cannot exceed amount_sats ({})",
                self.fee_sats, self.amount_sats
            )));
        }
        Ok(())
    }
}

impl Fdc3Instrument {
    /// Validates FDC3 instrument fields against protocol invariants.
    pub fn validate(&self) -> Result<(), IntentError> {
        if self.ticker.trim().is_empty() {
            return Err(IntentError::InvalidFdc3Instrument(
                "ticker cannot be empty or whitespace".to_string(),
            ));
        }
        if self.conxian_asset_id.trim().is_empty() {
            return Err(IntentError::InvalidFdc3Instrument(
                "conxian_asset_id cannot be empty or whitespace".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct IntentManager;

impl IntentManager {
    /// Ranks bids based on yield, cost, and latency
    /// Score = (Amount * 0.4) - (Fee * 0.2) - (Latency * 0.4)
    pub fn rank_bids(bids: &[Bid]) -> Vec<Bid> {
        let mut sorted_bids = bids.to_vec();
        sorted_bids.sort_by(|a, b| {
            // Normalize latency to sats-equivalent impact for ranking
            // 1 block of latency is worth ~50,000 sats in this model
            let score_a = (a.amount_sats as f64 * 0.4)
                - (a.fee_sats as f64 * 0.2)
                - (a.estimated_latency_blocks as f64 * 50_000.0 * 0.4);
            let score_b = (b.amount_sats as f64 * 0.4)
                - (b.fee_sats as f64 * 0.2)
                - (b.estimated_latency_blocks as f64 * 50_000.0 * 0.4);
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_bids
    }

    /// Ranks bids with fail-closed validation of each bid parameter.
    pub fn rank_bids_checked(bids: &[Bid]) -> Result<Vec<Bid>, IntentError> {
        for bid in bids {
            bid.validate()?;
        }
        Ok(Self::rank_bids(bids))
    }

    /// Resolves an FDC3 instrument into a Cross-Chain Intent context.
    /// Integrated with RailProxy intent resolution path (CON-1406).
    pub fn resolve_fdc3_intent(
        instrument: &Fdc3Instrument,
        amount: u64,
        destination: &str,
    ) -> Result<String, String> {
        Self::resolve_fdc3_intent_checked(instrument, amount, destination)
            .map_err(|e| e.to_string())
    }

    /// Resolves an FDC3 instrument into a Cross-Chain Intent context with fail-closed checks.
    pub fn resolve_fdc3_intent_checked(
        instrument: &Fdc3Instrument,
        amount: u64,
        destination: &str,
    ) -> Result<String, IntentError> {
        instrument.validate()?;

        if amount == 0 {
            return Err(IntentError::InvalidAmount(0));
        }

        if destination.trim().is_empty() {
            return Err(IntentError::InvalidDestination(
                "destination cannot be empty or whitespace".to_string(),
            ));
        }

        Ok(format!(
            "fdc3_intent:{}:{}:{}:{}",
            instrument.conxian_asset_id.trim(),
            instrument.ticker.trim(),
            amount,
            destination.trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bid_ranking() {
        let bids = vec![
            Bid {
                solver_id: "fast_but_expensive".to_string(),
                amount_sats: 1_000_000,
                estimated_latency_blocks: 1,
                fee_sats: 10_000,
            },
            Bid {
                solver_id: "slow_but_cheap".to_string(),
                amount_sats: 1_000_000,
                estimated_latency_blocks: 10,
                fee_sats: 1_000,
            },
        ];

        let ranked = IntentManager::rank_bids(&bids);
        assert_eq!(ranked[0].solver_id, "fast_but_expensive");

        let ranked_checked = IntentManager::rank_bids_checked(&bids).unwrap();
        assert_eq!(ranked_checked[0].solver_id, "fast_but_expensive");
    }

    #[test]
    fn test_bid_validation_failures() {
        // Empty solver_id
        let b1 = Bid {
            solver_id: "   ".to_string(),
            amount_sats: 100,
            estimated_latency_blocks: 1,
            fee_sats: 10,
        };
        assert_eq!(b1.validate(), Err(IntentError::EmptySolverId));

        // Zero amount_sats
        let b2 = Bid {
            solver_id: "solver1".to_string(),
            amount_sats: 0,
            estimated_latency_blocks: 1,
            fee_sats: 0,
        };
        assert!(matches!(b2.validate(), Err(IntentError::InvalidBid(_))));

        // Zero latency blocks
        let b3 = Bid {
            solver_id: "solver1".to_string(),
            amount_sats: 100,
            estimated_latency_blocks: 0,
            fee_sats: 10,
        };
        assert!(matches!(b3.validate(), Err(IntentError::InvalidBid(_))));

        // Fee exceeding amount
        let b4 = Bid {
            solver_id: "solver1".to_string(),
            amount_sats: 100,
            estimated_latency_blocks: 1,
            fee_sats: 200,
        };
        assert!(matches!(b4.validate(), Err(IntentError::InvalidBid(_))));
    }

    #[test]
    fn test_fdc3_instrument_validation_and_resolution() {
        let valid_instrument = Fdc3Instrument {
            ticker: "BTC".to_string(),
            name: Some("Bitcoin".to_string()),
            isin: None,
            conxian_asset_id: "asset_001".to_string(),
        };

        assert!(valid_instrument.validate().is_ok());

        let result =
            IntentManager::resolve_fdc3_intent_checked(&valid_instrument, 100000, "bc1qtest")
                .unwrap();
        assert_eq!(result, "fdc3_intent:asset_001:BTC:100000:bc1qtest");

        // Test invalid ticker
        let invalid_ticker = Fdc3Instrument {
            ticker: "  ".to_string(),
            name: None,
            isin: None,
            conxian_asset_id: "asset_001".to_string(),
        };
        assert!(matches!(
            invalid_ticker.validate(),
            Err(IntentError::InvalidFdc3Instrument(_))
        ));

        // Test zero amount
        assert_eq!(
            IntentManager::resolve_fdc3_intent_checked(&valid_instrument, 0, "bc1qtest"),
            Err(IntentError::InvalidAmount(0))
        );

        // Test empty destination
        assert!(matches!(
            IntentManager::resolve_fdc3_intent_checked(&valid_instrument, 100, "  "),
            Err(IntentError::InvalidDestination(_))
        ));
    }

    #[test]
    fn test_intent_error_display() {
        assert_eq!(
            IntentError::InvalidFdc3Instrument("missing ticker".to_string()).to_string(),
            "invalid FDC3 instrument: missing ticker"
        );
        assert_eq!(
            IntentError::InvalidBid("amount zero".to_string()).to_string(),
            "invalid bid parameters: amount zero"
        );
        assert_eq!(
            IntentError::InvalidAmount(0).to_string(),
            "invalid intent amount: 0 (must be > 0)"
        );
        assert_eq!(
            IntentError::InvalidDestination("empty".to_string()).to_string(),
            "invalid intent destination: empty"
        );
        assert_eq!(
            IntentError::EmptySolverId.to_string(),
            "solver ID cannot be empty or whitespace"
        );
    }
}
