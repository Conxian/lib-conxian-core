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
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub request_count: AtomicU64,
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
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_service_status(&self, service: &str) -> ServiceStatus {
        let (latency_ms, trust_model, risk_level) = match service {
            "bisq" => (45, "P2P", "Low"),
            "rgb" => (12, "Client-side", "Low"),
            "bitvm" => (88, "Optimistic", "Medium"),
            "changelly" => (120, "Centralized", "High"),
            "stacks" => (65, "PoX/Sidechain", "Medium"),
            "lightning" => (5, "State Channels", "Low"),
            "liquid" => (25, "Federated", "Medium"),
            "rootstock" => (35, "Powpeg/Sidechain", "Medium"),
            _ => (0, "Unknown", "Unknown"),
        };

        ServiceStatus {
            name: service.to_string(),
            status: "active".to_string(),
            last_checked: Utc::now(),
            latency_ms,
            trust_model: trust_model.to_string(),
            risk_level: risk_level.to_string(),
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
