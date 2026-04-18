use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCard {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@type")]
    pub r#type: String,
    pub work_intent: WorkIntent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkIntent {
    pub sender_address: String,
    pub receiver_address: String,
    pub task_id: String,
    pub amount_sbtc: u64,
}
