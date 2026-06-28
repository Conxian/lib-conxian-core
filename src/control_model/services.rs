use serde::{Deserialize, Serialize};

/// Basic status of an ecosystem service.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BasicServiceStatus {
    pub service_name: String,
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceResponse {
    pub service: String,
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Common trait for all Conxian ecosystem services.
pub trait ConxianService {
    fn name(&self) -> &str;
    fn status(&self) -> BasicServiceStatus;
    fn handle_request(&self, payload: &str) -> ServiceResponse;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ReserveAsset {
    pub asset: String,
    pub total_supplied: f64,
    pub total_reserves: f64,
    pub collateral_ratio: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PriceInfo {
    // f64 does not implement Eq
    pub asset: String,
    pub price_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ComplianceStatus {
    pub status: String,
    pub last_audit: chrono::DateTime<chrono::Utc>,
    pub rules_active: Vec<String>,
    pub risk_score: u32,
    pub zkml_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FinancialMetrics {
    pub mrr_usd: f64,
    pub arr_usd: f64,
    pub churn_rate_pct: f64,
    pub protocol_fees_collected_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
