use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use actix_web::web;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub version: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub request_count: AtomicU64,
    pub total_tvl_usd: AtomicU64,
    pub active_sovereign_nodes: AtomicU64,
    pub service_statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let mut statuses = HashMap::new();

        let services = vec![
            ("bisq", 45, "P2P", "Low", "On-chain", "Bitcoin", "N/A"),
            ("rgb", 12, "Client-side", "Low", "Off-chain", "Bitcoin", "Client-side"),
            ("bitvm", 88, "Optimistic", "Medium", "On-chain", "Bitcoin", "Fraud Proofs"),
            ("changelly", 120, "Centralized", "High", "N/A", "Centralized", "Centralized"),
            ("stacks", 65, "PoX", "Medium", "On-chain", "Bitcoin", "sBTC Bridge"),
            ("lightning", 5, "State Channels", "Low", "Off-chain", "Bitcoin", "N/A"),
            ("liquid", 25, "Federated", "Medium", "On-chain (Federated)", "Bitcoin", "Strong Federation"),
            ("rootstock", 35, "Powpeg", "Medium", "On-chain", "Bitcoin", "Powpeg"),
        ];

        for (name, latency, trust, risk, da, settlement, bridge) in services {
            let mut metadata = HashMap::new();
            match name {
                "stacks" => {
                    metadata.insert("block_height".to_string(), "840000".to_string());
                    metadata.insert("sbtc_bridge_status".to_string(), "active".to_string());
                },
                "lightning" => {
                    metadata.insert("channel_count".to_string(), "1542".to_string());
                    metadata.insert("capacity_btc".to_string(), "42.5".to_string());
                },
                _ => {}
            }

            statuses.insert(name.to_string(), ServiceStatus {
                name: name.to_string(),
                status: "active".to_string(),
                last_checked: Utc::now(),
                latency_ms: latency,
                trust_model: trust.to_string(),
                risk_level: risk.to_string(),
                data_availability: da.to_string(),
                settlement: settlement.to_string(),
                bridge_security: bridge.to_string(),
                version: Some("1.0.0".to_string()),
                metadata,
            });
        }

        Self {
            version: "0.1.0".to_string(),
            start_time: Utc::now(),
            request_count: AtomicU64::new(0),
            total_tvl_usd: AtomicU64::new(1_320_000_000),
            active_sovereign_nodes: AtomicU64::new(8),
            service_statuses: Arc::new(RwLock::new(statuses)),
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_service_status(&self, service: &str) -> ServiceStatus {
        let statuses = self.service_statuses.read().unwrap();
        statuses.get(service).cloned().unwrap_or_else(|| ServiceStatus {
            name: service.to_string(),
            status: "unknown".to_string(),
            last_checked: Utc::now(),
            latency_ms: 0,
            trust_model: "Unknown".to_string(),
            risk_level: "Unknown".to_string(),
            data_availability: "Unknown".to_string(),
            settlement: "Unknown".to_string(),
            bridge_security: "Unknown".to_string(),
            version: None,
            metadata: HashMap::new(),
        })
    }

    pub fn get_system_info(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "uptime_seconds": Utc::now().signed_duration_since(self.start_time).num_seconds(),
            "status": "operational",
            "total_requests": self.request_count.load(Ordering::SeqCst),
            "active_nodes": self.active_sovereign_nodes.load(Ordering::SeqCst),
        })
    }

    pub async fn start_monitoring(engine_data: web::Data<Engine>) {
        log::info!("Starting background service monitoring...");
        let statuses_clone = Arc::clone(&engine_data.service_statuses);

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                log::debug!("Updating service statuses...");

                let mut statuses = statuses_clone.write().unwrap();
                for status in statuses.values_mut() {
                    // Simulate minor latency fluctuations
                    let fluctuation = (Utc::now().timestamp() % 11) as i32 - 5;
                    status.latency_ms = (status.latency_ms as i32 + fluctuation).max(1) as u32;
                    status.last_checked = Utc::now();

                    // Protocol-specific simulated updates
                    match status.name.as_str() {
                        "stacks" => {
                            if let Some(height_str) = status.metadata.get_mut("block_height") {
                                let height: u64 = height_str.parse().unwrap_or(840000);
                                *height_str = (height + 1).to_string();
                            }
                        },
                        "lightning" => {
                            if let Some(capacity_str) = status.metadata.get_mut("capacity_btc") {
                                let capacity: f64 = capacity_str.parse().unwrap_or(42.5);
                                *capacity_str = format!("{:.1}", capacity + 0.1);
                            }
                        },
                        _ => {}
                    }

                    // Simulate random health change (very rare)
                    if Utc::now().timestamp() % 1000 == 0 {
                        status.status = "degraded".to_string();
                    } else {
                        status.status = "active".to_string();
                    }
                }
            }
        });
    }
}
