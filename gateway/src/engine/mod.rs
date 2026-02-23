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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReserveAsset {
    pub asset: String,
    pub total_supplied: f64,
    pub total_reserves: f64,
    pub collateral_ratio: f64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceInfo {
    pub asset: String,
    pub price_usd: f64,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComplianceStatus {
    pub status: String,
    pub last_audit: DateTime<Utc>,
    pub rules_active: Vec<String>,
    pub risk_score: u32,
}



pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub request_count: AtomicU64,
    pub total_tvl_usd: AtomicU64,
    pub active_sovereign_nodes: AtomicU64,
    pub service_statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
    pub reserves: Arc<RwLock<Vec<ReserveAsset>>>,
    pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,
    pub compliance: Arc<RwLock<ComplianceStatus>>,
}

impl Engine {
    fn calculate_risk_level(latency: u32, trust_model: &str) -> String {
        if latency > 250 || trust_model == "Centralized" {
            "High".to_string()
        } else if latency > 150 || trust_model == "Federated" || trust_model == "Optimistic" || trust_model == "Optimistic Rollup" || trust_model == "Powpeg" || trust_model == "Spiderchain" || trust_model == "Economic Layer" || trust_model == "Multi-layer" {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }
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
            ("babylon", 40, "Staking", "Low", "On-chain", "Bitcoin", "N/A"),
            ("bob", 55, "Optimistic Rollup", "Medium", "On-chain (EVM)", "Bitcoin", "Optimistic Bridge"),
            ("merlin", 42, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("botanix", 38, "Spiderchain", "Medium", "On-chain (EVM)", "Bitcoin", "Multisig"),
            ("b2network", 45, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("citrea", 52, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("bitlayer", 60, "Optimistic", "Medium", "On-chain", "Bitcoin", "BitVM Bridge"),
            ("alpen", 48, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("mezo", 58, "Economic Layer", "Medium", "On-chain", "Bitcoin", "tBTC Bridge"),
            ("zulu", 50, "Multi-layer", "Medium", "On-chain", "Bitcoin", "Decentralized Bridge"),
            ("bison", 42, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("hemi", 45, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),
            ("taproot-assets", 15, "Client-side", "Low", "On-chain", "Bitcoin", "N/A"),
            ("nubit", 28, "Data Availability", "Low", "Off-chain (DA)", "Bitcoin", "N/A"),
            ("lorenzo", 46, "Staking", "Low", "On-chain", "Bitcoin", "N/A"),
        ];

        for (name, latency, trust, risk, da, settlement, bridge) in services {
            let mut metadata = HashMap::new();
            match name {
                "bisq" => {
                    metadata.insert("active_offers".to_string(), "124".to_string());
                    metadata.insert("volume_24h_btc".to_string(), "12.5".to_string());
                },
                "rgb" => {
                    metadata.insert("contract_count".to_string(), "85".to_string());
                },
                "bitvm" => {
                    metadata.insert("proof_window_blocks".to_string(), "144".to_string());
                },
                "stacks" => {
                    metadata.insert("block_height".to_string(), "840000".to_string());
                    metadata.insert("sbtc_bridge_status".to_string(), "active".to_string());
                },
                "lightning" => {
                    metadata.insert("channel_count".to_string(), "1542".to_string());
                    metadata.insert("capacity_btc".to_string(), "42.5".to_string());
                },
                "liquid" => {
                    metadata.insert("pegged_btc".to_string(), "3541.2".to_string());
                },
                "rootstock" => {
                    metadata.insert("mining_hashrate_ph".to_string(), "245.8".to_string());
                },
                "babylon" => {
                    metadata.insert("staked_btc".to_string(), "1250.0".to_string());
                },
                "bob" => {
                    metadata.insert("tvl_usd".to_string(), "45000000".to_string());
                },
                "merlin" => {
                    metadata.insert("tvl_usd".to_string(), "1500000000".to_string());
                },
                "botanix" => {
                    metadata.insert("nodes_active".to_string(), "21".to_string());
                },
                "b2network" => {
                    metadata.insert("block_height".to_string(), "12543".to_string());
                },
                "citrea" => {
                    metadata.insert("tvl_usd".to_string(), "12500000".to_string());
                },
                "bitlayer" => {
                    metadata.insert("tvl_usd".to_string(), "8500000".to_string());
                },
                "alpen" => {
                    metadata.insert("tvl_usd".to_string(), "5000000".to_string());
                },
                "mezo" => {
                    metadata.insert("tvl_usd".to_string(), "120000000".to_string());
                    metadata.insert("staked_tbtc".to_string(), "1850.5".to_string());
                },
                "zulu" => {
                    metadata.insert("block_height".to_string(), "5421".to_string());
                },
                "bison" => {
                    metadata.insert("tvl_usd".to_string(), "3200000".to_string());
                },
                "hemi" => {
                    metadata.insert("tvl_usd".to_string(), "2100000".to_string());
                },
                "taproot-assets" => {
                    metadata.insert("asset_count".to_string(), "142".to_string());
                },
                "nubit" => {
                    metadata.insert("da_status".to_string(), "active".to_string());
                },
                "lorenzo" => {
                    metadata.insert("staked_btc".to_string(), "850.2".to_string());
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

        let reserves = vec![
            ReserveAsset { asset: "Liquid (L-BTC)".to_string(), total_supplied: 452.4, total_reserves: 521.8, collateral_ratio: 115.3, status: "Audited".to_string() },
            ReserveAsset { asset: "Stacks (sBTC)".to_string(), total_supplied: 281.2, total_reserves: 352.5, collateral_ratio: 125.3, status: "Audited".to_string() },
            ReserveAsset { asset: "Rootstock (RBTC)".to_string(), total_supplied: 122.5, total_reserves: 143.1, collateral_ratio: 116.8, status: "Audited".to_string() },
            ReserveAsset { asset: "Wormhole NTT".to_string(), total_supplied: 551.0, total_reserves: 1320.0, collateral_ratio: 111.1, status: "Verified".to_string() },
        ];


        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), PriceInfo { asset: "BTC".to_string(), price_usd: 65000.0, last_updated: Utc::now(), source: "Conxian Oracle".to_string() });
        prices.insert("STX".to_string(), PriceInfo { asset: "STX".to_string(), price_usd: 2.5, last_updated: Utc::now(), source: "Conxian Oracle".to_string() });

        let compliance = ComplianceStatus {
            status: "compliant".to_string(),
            last_audit: Utc::now(),
            rules_active: vec!["KYC".to_string(), "AML".to_string(), "NetworkIntegrity".to_string()],
            risk_score: 15,
        };

        Self {
            version: "0.1.0".to_string(),
            start_time: Utc::now(),
            request_count: AtomicU64::new(0),
            total_tvl_usd: AtomicU64::new(1_320_000_000),
            active_sovereign_nodes: AtomicU64::new(8),
            service_statuses: Arc::new(RwLock::new(statuses)),
            reserves: Arc::new(RwLock::new(reserves)),
            prices: Arc::new(RwLock::new(prices)),
            compliance: Arc::new(RwLock::new(compliance)),
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

    pub fn get_reserves(&self) -> Vec<ReserveAsset> {
        self.reserves.read().unwrap().clone()
    }

    pub fn get_prices(&self) -> HashMap<String, PriceInfo> {
        self.prices.read().unwrap().clone()
    }

    pub fn get_compliance_status(&self) -> ComplianceStatus {
        self.compliance.read().unwrap().clone()
    }

    pub fn get_all_service_statuses(&self) -> HashMap<String, ServiceStatus> {
        self.service_statuses.read().unwrap().clone()
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
        let engine_clone = engine_data.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                log::debug!("Updating service statuses and reserves...");

                {
                    let mut statuses = engine_clone.service_statuses.write().unwrap();
                    for status in statuses.values_mut() {
                        let fluctuation = (Utc::now().timestamp() % 11) as i32 - 5;
                        status.latency_ms = (status.latency_ms as i32 + fluctuation).max(1) as u32;
                        status.last_checked = Utc::now();
                        status.risk_level = Self::calculate_risk_level(status.latency_ms, &status.trust_model);

                        match status.name.as_str() {
                            "bisq" => {
                                if let Some(v) = status.metadata.get_mut("active_offers") {
                                    let offers: u32 = v.parse().unwrap_or(124);
                                    *v = (offers + (Utc::now().timestamp() % 3) as u32).to_string();
                                }
                            },
                            "rgb" => {
                                if let Some(v) = status.metadata.get_mut("contract_count") {
                                    let count: u32 = v.parse().unwrap_or(85);
                                    if Utc::now().timestamp() % 5 == 0 {
                                        *v = (count + 1).to_string();
                                    }
                                }
                            },
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
                            "liquid" => {
                                if let Some(v) = status.metadata.get_mut("pegged_btc") {
                                    let pegged: f64 = v.parse().unwrap_or(3541.2);
                                    *v = format!("{:.1}", pegged + (Utc::now().timestamp() % 10) as f64 / 10.0);
                                }
                            },
                            "rootstock" => {
                                if let Some(v) = status.metadata.get_mut("mining_hashrate_ph") {
                                    let hashrate: f64 = v.parse().unwrap_or(245.8);
                                    *v = format!("{:.1}", hashrate + (Utc::now().timestamp() % 5) as f64 - 2.0);
                                }
                            },
                            "babylon" => {
                                if let Some(v) = status.metadata.get_mut("staked_btc") {
                                    let staked: f64 = v.parse().unwrap_or(1250.0);
                                    *v = format!("{:.1}", staked + 0.5);
                                }
                            },
                            "bob" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(45000000.0);
                                    *v = format!("{:.0}", tvl + 1000.0);
                                }
                            },
                            "merlin" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(1500000000.0);
                                    *v = format!("{:.0}", tvl + 5000.0);
                                }
                            },
                            "botanix" => {
                                if let Some(v) = status.metadata.get_mut("nodes_active") {
                                    let nodes: u32 = v.parse().unwrap_or(21);
                                    if Utc::now().timestamp() % 60 == 0 {
                                        *v = (nodes + 1).to_string();
                                    }
                                }
                            },
                            "b2network" => {
                                if let Some(v) = status.metadata.get_mut("block_height") {
                                    let height: u64 = v.parse().unwrap_or(12543);
                                    *v = (height + 1).to_string();
                                }
                            },
                            "citrea" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(12500000.0);
                                    *v = format!("{:.0}", tvl + 2500.0);
                                }
                            },
                            "bitlayer" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(8500000.0);
                                    *v = format!("{:.0}", tvl + 1800.0);
                                }
                            },
                            "alpen" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(5000000.0);
                                    *v = format!("{:.0}", tvl + 1200.0);
                                }
                            },
                            "mezo" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(120000000.0);
                                    *v = format!("{:.0}", tvl + 25000.0);
                                }
                            },
                            "zulu" => {
                                if let Some(v) = status.metadata.get_mut("block_height") {
                                    let height: u64 = v.parse().unwrap_or(5421);
                                    *v = (height + 1).to_string();
                                }
                            },
                            "bison" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(3200000.0);
                                    *v = format!("{:.0}", tvl + 800.0);
                                }
                            },
                            "hemi" => {
                                if let Some(v) = status.metadata.get_mut("tvl_usd") {
                                    let tvl: f64 = v.parse().unwrap_or(2100000.0);
                                    *v = format!("{:.0}", tvl + 500.0);
                                }
                            },
                            "taproot-assets" => {
                                if let Some(v) = status.metadata.get_mut("asset_count") {
                                    let count: u32 = v.parse().unwrap_or(142);
                                    if Utc::now().timestamp() % 10 == 0 {
                                        *v = (count + 1).to_string();
                                    }
                                }
                            },
                            "nubit" => {
                                if Utc::now().timestamp() % 60 == 0 {
                                     status.latency_ms = 25;
                                }
                            },
                            "lorenzo" => {
                                if let Some(v) = status.metadata.get_mut("staked_btc") {
                                    let staked: f64 = v.parse().unwrap_or(850.2);
                                    *v = format!("{:.1}", staked + 0.2);
                                }
                            },
                            _ => {}
                        }
                    }
                }



                {
                    let mut compliance = engine_clone.compliance.write().unwrap();
                    let current_requests = engine_clone.request_count.load(Ordering::SeqCst);
                    // Simulate dynamic risk score based on activity
                    compliance.risk_score = (10 + (current_requests % 20) as u32).min(100);
                    if compliance.risk_score > 80 {
                        compliance.status = "warning".to_string();
                    } else {
                        compliance.status = "compliant".to_string();
                    }
                }

                {
                    let mut prices = engine_clone.prices.write().unwrap();
                    for price in prices.values_mut() {
                        let fluctuation = (Utc::now().timestamp() % 101) as f64 / 10000.0 - 0.005;
                        price.price_usd *= 1.0 + fluctuation;
                        price.last_updated = Utc::now();
                    }
                }

                engine_clone.update_dynamic_stats();

                {
                    let mut reserves = engine_clone.reserves.write().unwrap();
                    let current_tvl = engine_clone.total_tvl_usd.load(Ordering::SeqCst);
                    for reserve in reserves.iter_mut() {
                        if reserve.asset == "Wormhole NTT" {
                            reserve.total_reserves = (current_tvl as f64) / 1_000_000.0;
                        }
                        reserve.total_supplied += (Utc::now().timestamp() % 5) as f64 / 10.0;
                    }
                }
            }
        });
    }
}

impl Engine {
    pub fn create_lightning_invoice(&self, amount_msat: u64, description: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "invoice": format!("lnbc{}1p...", amount_msat / 1000),
            "payment_hash": "d7a8fbb307d7809469ca9abcb0082e4f",
            "description": description,
            "expiry": 3600
        })
    }

    pub fn pay_lightning_invoice(&self, _invoice: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "status": "success",
            "preimage": "6f2e4b3c...",
            "destination": "03abcd...",
            "amount_msat": 10000
        })
    }

    pub fn get_stacks_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "contract_id": contract_id,
            "source_code": "(define-public (hello) (ok \"world\"))",
            "abi": {
                "functions": [{"name": "hello", "access": "public", "outputs": {"type": "string"}}]
            },
            "status": "active"
        })
    }

    pub fn get_rgb_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "contract_id": contract_id,
            "schema": "FungibleAsset",
            "state": "Verified",
            "last_transition": Utc::now()
        })
    }

    pub fn get_bitvm_proof(&self, proof_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "proof_id": proof_id,
            "status": "Verified",
            "verifier_count": 5,
            "challenge_period_blocks": 144
        })
    }

    pub fn get_exchange_rate(&self, from: &str, to: &str) -> serde_json::Value {
        self.increment_requests();
        let rate = match (from, to) {
            ("BTC", "USD") => 65000.0,
            ("USD", "BTC") => 1.0 / 65000.0,
            ("STX", "BTC") => 0.000038,
            _ => 1.0,
        };
        serde_json::json!({
            "from": from,
            "to": to,
            "rate": rate,
            "timestamp": Utc::now()
        })
    }

    pub fn is_healthy(&self) -> bool {
        let statuses = self.service_statuses.read().unwrap();
        // Check if at least some services are active and checked recently
        if statuses.is_empty() { return false; }
        let now = Utc::now();
        statuses.values().any(|s| (now - s.last_checked).num_seconds() < 60)
    }

    pub fn check_compliance(&self, address: &str) -> serde_json::Value {
        self.increment_requests();
        let is_compliant = !address.contains("bad");
        serde_json::json!({
            "address": address,
            "compliant": is_compliant,
            "risk_score": if is_compliant { 10 } else { 95 },
            "timestamp": Utc::now()
        })
    }

    pub fn calculate_total_tvl(&self) -> u64 {
        let statuses = self.service_statuses.read().unwrap();
        let mut total = 0u64;
        for status in statuses.values() {
            if let Some(tvl_str) = status.metadata.get("tvl_usd") {
                if let Ok(tvl) = tvl_str.parse::<f64>() {
                    total += tvl as u64;
                }
            }
        }
        total
    }

    pub fn update_dynamic_stats(&self) {
        let new_tvl = self.calculate_total_tvl();
        self.total_tvl_usd.store(new_tvl, Ordering::SeqCst);
    }

    pub fn get_b2_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("b2network");
        serde_json::json!({
            "block_height": status.metadata.get("block_height").cloned().unwrap_or_default(),
            "proof_status": "Verified",
            "sequencer_batches": 1254,
            "da_layer": "Bitcoin"
        })
    }

    pub fn get_citrea_proof(&self, batch_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "batch_id": batch_id,
            "status": "Finalized",
            "zk_proof": "0xabc...",
            "settlement_tx": "0x123...",
            "timestamp": Utc::now()
        })
    }
}
