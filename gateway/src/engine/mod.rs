use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: DateTime<Utc>,
    pub latency_ms: u32,
    pub trust_model: String,
    pub risk_level: String,
    pub data_availability: String,
    pub settlement: String,
    pub bridge_security: String,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub request_count: AtomicU64,
    pub total_tvl_usd: AtomicU64,
    pub active_sovereign_nodes: AtomicU64,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            version: "0.1.0".to_string(),
            start_time: Utc::now(),
            request_count: AtomicU64::new(0),
            total_tvl_usd: AtomicU64::new(1_320_000_000), // Mock $1.32B TVL
            active_sovereign_nodes: AtomicU64::new(3), // Mock 3 active nodes
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_service_status(&self, service: &str) -> ServiceStatus {
        let (latency_ms, trust_model, risk_level, da, settlement, bridge) = match service {
            "bisq" => (45, "P2P", "Low", "On-chain", "Bitcoin", "N/A"),
            "rgb" => (12, "Client-side", "Low", "Off-chain", "Bitcoin", "Client-side"),
            "bitvm" => (88, "Optimistic", "Medium", "On-chain", "Bitcoin", "Fraud Proofs"),
            "changelly" => (120, "Centralized", "High", "N/A", "Centralized", "Centralized"),
            "stacks" => (65, "PoX", "Medium", "On-chain", "Bitcoin", "sBTC Bridge"),
            "lightning" => (5, "State Channels", "Low", "Off-chain", "Bitcoin", "N/A"),
            "liquid" => (25, "Federated", "Medium", "On-chain (Federated)", "Bitcoin", "Strong Federation"),
            "rootstock" => (35, "Powpeg", "Medium", "On-chain", "Bitcoin", "Powpeg"),
            _ => (0, "Unknown", "Unknown", "Unknown", "Unknown", "Unknown"),
        };

        ServiceStatus {
            name: service.to_string(),
            status: "active".to_string(),
            last_checked: Utc::now(),
            latency_ms,
            trust_model: trust_model.to_string(),
            risk_level: risk_level.to_string(),
            data_availability: da.to_string(),
            settlement: settlement.to_string(),
            bridge_security: bridge.to_string(),
        }
    }

    pub fn get_system_info(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "uptime_seconds": Utc::now().signed_duration_since(self.start_time).num_seconds(),
            "status": "operational",
            "total_requests": self.request_count.load(Ordering::SeqCst),
        })
    }
}
