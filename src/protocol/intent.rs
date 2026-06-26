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
    }
}
