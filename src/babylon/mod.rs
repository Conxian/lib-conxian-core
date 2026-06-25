//! Babylon Bitcoin Staking Adapter
//! Aligned with CXIP 20 and G-43

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StakingIntent {
    pub intent_id: String,
    pub amount_sats: u64,
    pub locking_period_blocks: u32,
    pub status: StakingStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum StakingStatus {
    Proposed,
    Locked,
    Slashed,
    Unbonding,
    Withdrawn,
}

pub struct BabylonAdapter;

impl BabylonAdapter {
    pub fn create_staking_intent(&self, amount: u64, blocks: u32) -> StakingIntent {
        StakingIntent {
            intent_id: format!("babylon-{}", amount),
            amount_sats: amount,
            locking_period_blocks: blocks,
            status: StakingStatus::Proposed,
        }
    }
}
