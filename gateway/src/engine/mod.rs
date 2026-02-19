use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: DateTime<Utc>,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            version: "0.1.0".to_string(),
            start_time: Utc::now(),
        }
    }

    pub fn get_service_status(&self, service: &str) -> ServiceStatus {
        ServiceStatus {
            name: service.to_string(),
            status: "active".to_string(),
            last_checked: Utc::now(),
        }
    }

    pub fn get_system_info(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "uptime": Utc::now().signed_duration_since(self.start_time).num_seconds(),
            "status": "operational"
        })
    }
}
