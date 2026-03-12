use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use actix_web::web;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RiskAssessment {
    pub overall_level: String,
    pub da_score: u32,
    pub settlement_score: u32,
    pub bridge_score: u32,
    pub exit_mechanism_score: u32,
    pub operators_score: u32,
    pub decentralization_score: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub last_checked: DateTime<Utc>,
    pub latency_ms: u32,
    pub trust_model: String,
    pub risk_level: String,
    pub risk_assessment: Option<RiskAssessment>,
    pub data_availability: String,
    pub settlement: String,
    pub bridge_security: String,
    pub tvl_usd: f64,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AffiliateInfo {
    pub partner_id: String,
    pub status: String,
    pub commission_rate: f64,
    pub active_campaigns: u32,
    pub total_referrals: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketingInfo {
    pub channel: String,
    pub status: String,
    pub active_offers: Vec<String>,
    pub reach: u64,
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
    pub affiliates: Arc<RwLock<HashMap<String, AffiliateInfo>>>,
    pub marketing: Arc<RwLock<Vec<MarketingInfo>>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    fn evaluate_risk(latency: u32, trust_model: &str, da: &str, bridge: &str) -> RiskAssessment {
        let mut da_score = 90;
        let mut settlement_score = 85;
        let mut bridge_score = 80;
        let mut dec_score = 75;
        let mut exit_score = 85;
        let mut ops_score = 80;

        if da.contains("Off-chain") { da_score -= 30; }
        if bridge.contains("Federated") || bridge.contains("Multisig") { bridge_score -= 25; ops_score -= 20; }
        if bridge.contains("Non-custodial") { bridge_score += 10; exit_score += 10; }
        if trust_model == "Centralized" {
            da_score = 10; settlement_score = 10; bridge_score = 10; dec_score = 10; exit_score = 10; ops_score = 10;
        }

        let avg = (da_score + settlement_score + bridge_score + dec_score + exit_score + ops_score) / 6;
        let level = if avg < 40 || latency > 250 {
            "High".to_string()
        } else if avg < 70 || latency > 150 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        RiskAssessment {
            overall_level: level,
            da_score,
            settlement_score,
            bridge_score,
            decentralization_score: dec_score,
            exit_mechanism_score: exit_score,
            operators_score: ops_score,
        }
    }

    fn initialize_services() -> HashMap<String, ServiceStatus> {
        let mut statuses = HashMap::new();
        let services = vec![
            ("bisq", 45, "P2P", "On-chain", "Bitcoin", "N/A", 0.0),
            ("rgb", 12, "Client-side", "Off-chain", "Bitcoin", "Client-side", 0.0),
            ("bitvm", 88, "Optimistic", "On-chain", "Bitcoin", "Fraud Proofs", 0.0),
            ("bitvm2", 75, "Optimistic (SNARK)", "On-chain", "Bitcoin", "ZK-Fraud Proofs", 0.0),
            ("changelly", 120, "Centralized", "N/A", "Centralized", "Centralized", 0.0),
            ("stacks", 65, "PoX", "On-chain", "Bitcoin", "sBTC Bridge", 0.0),
            ("lightning", 5, "State Channels", "Off-chain", "Bitcoin", "N/A", 0.0),
            ("liquid", 25, "Federated", "On-chain (Federated)", "Bitcoin", "Strong Federation", 0.0),
            ("rootstock", 35, "Powpeg", "On-chain", "Bitcoin", "Powpeg", 0.0),
            ("babylon", 40, "Staking", "On-chain", "Bitcoin", "N/A", 0.0),
            ("bob", 55, "Optimistic Rollup", "On-chain (EVM)", "Bitcoin", "Optimistic Bridge", 45000000.0),
            ("merlin", 42, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 1500000000.0),
            ("botanix", 38, "Spiderchain", "On-chain (EVM)", "Bitcoin", "Multisig", 0.0),
            ("b2network", 45, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 0.0),
            ("citrea", 52, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 12500000.0),
            ("bitlayer", 60, "Optimistic", "On-chain", "Bitcoin", "BitVM Bridge", 8500000.0),
            ("alpen", 48, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 5000000.0),
            ("mezo", 58, "Economic Layer", "On-chain", "Bitcoin", "tBTC Bridge", 120000000.0),
            ("zulu", 50, "Multi-layer", "On-chain", "Bitcoin", "Decentralized Bridge", 0.0),
            ("bison", 42, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 3200000.0),
            ("hemi", 45, "ZK Rollup", "On-chain (ZK)", "Bitcoin", "ZK Bridge", 2100000.0),
            ("taproot-assets", 15, "Client-side", "On-chain", "Bitcoin", "N/A", 0.0),
            ("nubit", 28, "Data Availability", "Off-chain (DA)", "Bitcoin", "N/A", 0.0),
            ("lorenzo", 46, "Staking", "On-chain", "Bitcoin", "N/A", 0.0),
            ("core-dao", 32, "Satoshi Plus", "On-chain", "Bitcoin", "Non-custodial", 250000000.0),
        ];

        for (name, latency, trust, da, settlement, bridge, tvl) in services {
            let mut metadata = HashMap::new();
            match name {
                "bisq" => {
                    metadata.insert("active_offers".to_string(), "124".to_string());
                    metadata.insert("volume_24h_btc".to_string(), "12.5".to_string());
                },
                "stacks" => {
                    metadata.insert("block_height".to_string(), "840000".to_string());
                    metadata.insert("sbtc_bridge_status".to_string(), "active".to_string());
                },
                "lightning" => {
                    metadata.insert("channel_count".to_string(), "1542".to_string());
                    metadata.insert("capacity_btc".to_string(), "42.5".to_string());
                },
                "mezo" => {
                    metadata.insert("staked_tbtc".to_string(), "1850.5".to_string());
                },
                "core-dao" => {
                    metadata.insert("dual_token_staking".to_string(), "enabled".to_string());
                },
                "bob" => {
                    metadata.insert("connected_chains".to_string(), "Bitcoin,Ethereum".to_string());
                },
                "merlin" => {
                    metadata.insert("zk_proving_status".to_string(), "Active".to_string());
                },
                "hemi" => {
                    metadata.insert("bitcoin_finality_depth".to_string(), "6".to_string());
                },
                "lorenzo" => {
                    metadata.insert("staked_btc".to_string(), "150.0".to_string());
                },
                "b2network" => {
                    metadata.insert("block_height".to_string(), "12540".to_string());
                },
                _ => {}
            }

            let ra = Self::evaluate_risk(latency, trust, da, bridge);

            statuses.insert(name.to_string(), ServiceStatus {
                name: name.to_string(),
                status: "active".to_string(),
                last_checked: Utc::now(),
                latency_ms: latency,
                trust_model: trust.to_string(),
                risk_level: ra.overall_level.clone(),
                risk_assessment: Some(ra),
                data_availability: da.to_string(),
                settlement: settlement.to_string(),
                bridge_security: bridge.to_string(),
                tvl_usd: tvl,
                version: Some("1.1.0".to_string()),
                metadata,
            });
        }
        statuses
    }

    pub fn new() -> Self {
        let statuses = Self::initialize_services();

        let reserves = vec![
            ReserveAsset { asset: "Liquid (L-BTC)".to_string(), total_supplied: 452.4, total_reserves: 521.8, collateral_ratio: 115.3, status: "Audited".to_string() },
            ReserveAsset { asset: "Stacks (sBTC)".to_string(), total_supplied: 281.2, total_reserves: 352.5, collateral_ratio: 125.3, status: "Audited".to_string() },
            ReserveAsset { asset: "Rootstock (RBTC)".to_string(), total_supplied: 122.5, total_reserves: 143.1, collateral_ratio: 116.8, status: "Audited".to_string() },
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

        let mut affiliates = HashMap::new();
        affiliates.insert("CONXIAN_GLOBAL".to_string(), AffiliateInfo {
            partner_id: "CONXIAN_GLOBAL".to_string(),
            status: "active".to_string(),
            commission_rate: 0.15,
            active_campaigns: 5,
            total_referrals: 12450,
        });

        let marketing = vec![
            MarketingInfo {
                channel: "X/Twitter".to_string(),
                status: "active".to_string(),
                active_offers: vec!["L2_SUMMER".to_string(), "STX_STAKING".to_string()],
                reach: 500000,
            },
        ];

        Self {
            version: "0.2.0".to_string(),
            start_time: Utc::now(),
            request_count: AtomicU64::new(0),
            total_tvl_usd: AtomicU64::new(0),
            active_sovereign_nodes: AtomicU64::new(10),
            service_statuses: Arc::new(RwLock::new(statuses)),
            reserves: Arc::new(RwLock::new(reserves)),
            prices: Arc::new(RwLock::new(prices)),
            compliance: Arc::new(RwLock::new(compliance)),
            affiliates: Arc::new(RwLock::new(affiliates)),
            marketing: Arc::new(RwLock::new(marketing)),
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
            risk_assessment: None,
            data_availability: "Unknown".to_string(),
            settlement: "Unknown".to_string(),
            bridge_security: "Unknown".to_string(),
            tvl_usd: 0.0,
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

    pub fn get_affiliates(&self) -> HashMap<String, AffiliateInfo> {
        self.affiliates.read().unwrap().clone()
    }

    pub fn get_marketing(&self) -> Vec<MarketingInfo> {
        self.marketing.read().unwrap().clone()
    }

    pub fn get_system_info(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.version,
            "uptime_seconds": Utc::now().signed_duration_since(self.start_time).num_seconds(),
            "status": "operational",
            "total_requests": self.request_count.load(Ordering::SeqCst),
            "active_nodes": self.active_sovereign_nodes.load(Ordering::SeqCst),
            "total_tvl_usd": self.total_tvl_usd.load(Ordering::SeqCst),
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

                        let ra = Self::evaluate_risk(status.latency_ms, &status.trust_model, &status.data_availability, &status.bridge_security);
                        status.risk_level = ra.overall_level.clone();
                        status.risk_assessment = Some(ra);

                        // Simulated TVL growth
                        if status.tvl_usd > 0.0 {
                            status.tvl_usd += status.tvl_usd * 0.0001;
                        }
                    }
                }

                {
                    let mut compliance = engine_clone.compliance.write().unwrap();
                    let current_requests = engine_clone.request_count.load(Ordering::SeqCst);
                    compliance.risk_score = (10 + (current_requests % 20) as u32).min(100);
                    compliance.status = if compliance.risk_score > 80 { "warning".to_string() } else { "compliant".to_string() };
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
            }
        });
    }

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

    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("liquid");
        serde_json::json!({
            "asset": "L-BTC",
            "pegged_amount": status.metadata.get("pegged_btc").cloned().unwrap_or_default(),
            "federation_status": "Operational",
            "last_audit": Utc::now()
        })
    }

    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("rootstock");
        serde_json::json!({
            "asset": "RBTC",
            "mining_hashrate": status.metadata.get("mining_hashrate_ph").cloned().unwrap_or_default(),
            "peg_status": "active",
            "bridge_contract": "0x123..."
        })
    }

    pub fn get_babylon_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("babylon");
        serde_json::json!({
            "staked_btc": status.metadata.get("staked_btc").cloned().unwrap_or_default(),
            "active_validators": 125,
            "security_score": 98.5
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

    pub fn calculate_total_tvl(&self) -> f64 {
        let statuses = self.service_statuses.read().unwrap();
        let mut total = 0.0;
        for status in statuses.values() {
            total += status.tvl_usd;
        }
        total
    }

    pub fn update_dynamic_stats(&self) {
        let new_tvl = self.calculate_total_tvl();
        self.total_tvl_usd.store(new_tvl as u64, Ordering::SeqCst);
    }

    pub fn get_core_dao_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("core-dao");
        serde_json::json!({
            "hashrate_contribution_pct": 15.4,
            "dual_token_staking": status.metadata.get("dual_token_staking").cloned().unwrap_or_default(),
            "active_validators": 21,
            "total_staked_btc": 2500.0,
            "satoshi_plus_status": "Active"
        })
    }

    pub fn get_lorenzo_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("lorenzo");
        serde_json::json!({
            "staked_btc": status.metadata.get("staked_btc").cloned().unwrap_or_else(|| "150.0".to_string()),
            "reward_token": "stBTC",
            "active_pools": 3,
            "yield_apy": 4.5
        })
    }

    pub fn get_hemi_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("hemi");
        serde_json::json!({
            "sequencer_status": "Active",
            "proof_submission": "On-chain",
            "bitcoin_finality_depth": status.metadata.get("bitcoin_finality_depth").cloned().unwrap_or_else(|| "6".to_string()),
            "ethereum_finality_depth": 32
        })
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

    pub fn get_bob_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bob");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "connected_chains": status.metadata.get("connected_chains").cloned().unwrap_or_else(|| "Bitcoin,Ethereum".to_string()).split(',').collect::<Vec<&str>>(),
            "optimistic_bridge_status": "Active",
            "exit_period_blocks": 2016
        })
    }

    pub fn get_merlin_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("merlin");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "zk_proving_status": status.metadata.get("zk_proving_status").cloned().unwrap_or_else(|| "Active".to_string()),
            "sequencer_yield_pct": 12.5,
            "active_users": 45000
        })
    }

    pub fn get_mezo_yield(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("mezo");
        serde_json::json!({
            "staked_tbtc": status.metadata.get("staked_tbtc").cloned().unwrap_or_else(|| "1850.5".to_string()),
            "current_yield_apy": 6.2,
            "economic_security_usd": 150000000.0,
            "hbt_token_status": "Active"
        })
    }

    pub fn get_nubit_da_info(&self) -> serde_json::Value {
        self.increment_requests();
        let _status = self.get_service_status("nubit");
        serde_json::json!({
            "da_throughput_mbps": 15.5,
            "consensus_latency_ms": 250,
            "active_da_nodes": 450,
            "integrated_layers": ["B2Network", "Citrea"]
        })
    }

    pub fn get_bison_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bison");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "zk_roll_uptime_pct": 99.98,
            "proof_generation_latency_min": 15,
            "settlement_frequency_hours": 1
        })
    }

    pub fn get_zulu_info(&self) -> serde_json::Value {
        self.increment_requests();
        let _status = self.get_service_status("zulu");
        serde_json::json!({
            "layer_type": "Multi-layer",
            "evm_compatibility": "Full",
            "bridge_mode": "Decentralized",
            "active_canals": 12
        })
    }

    pub fn get_botanix_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let _status = self.get_service_status("botanix");
        serde_json::json!({
            "spiderchain_nodes": 144,
            "multisig_threshold": "100-of-144",
            "evm_block_height": 1245000,
            "status": "Active"
        })
    }

    pub fn get_bitlayer_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitlayer");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "bitvm_challenge_status": "Healthy",
            "active_validators": 21,
            "block_time_sec": 2
        })
    }

    pub fn get_alpen_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "zk_proof_type": "SNARK",
            "settlement_batch_size": 250,
            "finality_depth_bitcoin": 3
        })
    }

    pub fn get_taproot_assets_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let _status = self.get_service_status("taproot-assets");
        serde_json::json!({
            "total_assets_issued": 125,
            "total_transfers_24h": 450,
            "lightning_integration": "Enabled",
            "status": "Active"
        })
    }

    pub fn get_bitvm2_info(&self) -> serde_json::Value {
        self.increment_requests();
        let _status = self.get_service_status("bitvm2");
        serde_json::json!({
            "paradigm": "ZK-Fraud Proofs",
            "challenge_period_blocks": 144,
            "active_verifiers": 15,
            "status": "Operational"
        })
    }

    pub fn get_risk_assessments(&self) -> HashMap<String, Option<RiskAssessment>> {
        let statuses = self.service_statuses.read().unwrap();
        let mut assessments = HashMap::new();
        for (name, status) in statuses.iter() {
            assessments.insert(name.clone(), status.risk_assessment.clone());
        }
        assessments
    }
}
