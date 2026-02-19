use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: DateTime<Utc>,
    pub latency_ms: u32,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub request_count: AtomicU64,
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
        // In a real implementation, this would probe the actual service.
        // For now, we simulate a healthy response with a small random-like latency.
        let latency_ms = match service {
            "bisq" => 45,
            "rgb" => 12,
            "bitvm" => 88,
            "changelly" => 120,
            _ => 0,
        };

        ServiceStatus {
            name: service.to_string(),
            status: "active".to_string(),
            last_checked: Utc::now(),
            latency_ms,
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
