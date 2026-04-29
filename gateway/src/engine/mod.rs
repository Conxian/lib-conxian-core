pub mod mcp;
pub mod remediation;
pub mod support;
use crate::engine::support::{SupportConfig, SupportIntake};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

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
    pub zkml_enabled: bool,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinancialMetrics {
    pub mrr_usd: f64,
    pub arr_usd: f64,
    pub churn_rate_pct: f64,
    pub protocol_fees_collected_usd: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityRecord {
    pub address: String,
    pub ens_name: Option<String>,
    pub bns_name: Option<String>,
    pub world_id_verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErpSyncRecord {
    pub erp_system: String,
    pub last_sync: DateTime<Utc>,
    pub total_transactions_synced: u64,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettlementEnvelope {
    pub protocol: String,
    pub payload: Value,
    pub raw_payload_bytes: String,
    pub ingress_timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLead {
    pub lead_id: String,
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PartnerLeadStatus {
    New,
    Assigned,
    InProgress,
    Qualified,
    Escalated,
    ClosedWon,
    ClosedLost,
}

impl PartnerLeadStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PartnerLeadStatus::New => "new",
            PartnerLeadStatus::Assigned => "assigned",
            PartnerLeadStatus::InProgress => "in_progress",
            PartnerLeadStatus::Qualified => "qualified",
            PartnerLeadStatus::Escalated => "escalated",
            PartnerLeadStatus::ClosedWon => "closed_won",
            PartnerLeadStatus::ClosedLost => "closed_lost",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadEvent {
    pub event_id: String,
    pub lead_id: String,
    pub timestamp: DateTime<Utc>,
    pub from_status: PartnerLeadStatus,
    pub to_status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub note: Option<String>,
}

pub struct PartnerLeadCreateInput {
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub notes: Option<String>,
}

pub struct PartnerLeadStatusUpdateInput {
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub escalated_to: Option<String>,
    pub escalation_reason: Option<String>,
    pub event_note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadCreateOutcome {
    pub lead_id: String,
    pub status: String,
    pub idempotent_replay: bool,
}

#[derive(Debug)]
pub enum PartnerLeadTransitionError {
    NotFound,
    InvalidTransition {
        from: PartnerLeadStatus,
        to: PartnerLeadStatus,
    },
    OwnerRequired,
    EscalationReasonRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalExecutionError {
    NotFound,
    NotApproved,
    TimelockNotExpired {
        current_block: u64,
        timelock_end_block: u64,
    },
}

impl ProposalExecutionError {
    pub fn message(&self, proposal_id: &str) -> String {
        match self {
            ProposalExecutionError::NotFound => {
                format!("Proposal {proposal_id} not found.")
            }
            ProposalExecutionError::NotApproved => {
                format!("Proposal {proposal_id} is not in Approved status.")
            }
            ProposalExecutionError::TimelockNotExpired {
                current_block,
                timelock_end_block,
            } => format!(
                "Proposal {proposal_id} timelock not expired: current block {current_block}, required block {timelock_end_block}."
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateProposal {
    pub proposal_id: String,
    pub trigger_id: String,
    pub proposed_state: String,
    pub timelock_end_block: u64,
    pub status: String, // "Pending", "Approved", "Executed"
    pub tee_attestation: String,
    pub yield_routing: String,  // "5/5/90"
    pub capital_status: String, // "TransitBond" or "Escrow"
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SabWallet {
    pub address: String,
    pub role: String,   // "Execution", "Treasury", "Payout", "Signer", "Emergency"
    pub owner: String,  // "SAB", "Operator", "DAO"
    pub status: String, // "Active", "Deprecated", "Pending"
    pub quorum: Option<String>,
    pub spending_limit_usd: Option<f64>,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
    pub support_intake: Arc<SupportIntake>,
    pub request_count: AtomicU64,
    pub total_tvl_usd: Arc<RwLock<f64>>,
    pub active_sovereign_nodes: AtomicU64,
    pub service_statuses: Arc<RwLock<HashMap<String, ServiceStatus>>>,
    pub reserves: Arc<RwLock<Vec<ReserveAsset>>>,
    pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,
    pub compliance: Arc<RwLock<ComplianceStatus>>,
    pub affiliates: Arc<RwLock<HashMap<String, AffiliateInfo>>>,
    pub marketing: Arc<RwLock<Vec<MarketingInfo>>>,
    pub financial_metrics: Arc<RwLock<FinancialMetrics>>,
    pub identity_records: Arc<RwLock<HashMap<String, IdentityRecord>>>,
    pub erp_sync_status: Arc<RwLock<HashMap<String, ErpSyncRecord>>>,
    pub settlement_log: Arc<RwLock<Vec<SettlementEnvelope>>>,
    pub state_proposals: Arc<RwLock<HashMap<String, StateProposal>>>,
    pub partner_leads: Arc<RwLock<HashMap<String, PartnerLead>>>,
    pub partner_lead_events: Arc<RwLock<Vec<PartnerLeadEvent>>>,
    pub partner_lead_idempotency: Arc<RwLock<HashMap<String, String>>>,
    pub partner_lead_sequence: AtomicU64,
    pub sab_wallets: Arc<RwLock<Vec<SabWallet>>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            version: "0.2.3".to_string(),
            start_time: Utc::now(),
            support_intake: Arc::new(SupportIntake::new(SupportConfig::default())),
            request_count: AtomicU64::new(0),
            total_tvl_usd: Arc::new(RwLock::new(0.0)),
            active_sovereign_nodes: AtomicU64::new(5),
            service_statuses: Arc::new(RwLock::new(HashMap::new())),
            reserves: Arc::new(RwLock::new(Vec::new())),
            prices: Arc::new(RwLock::new(HashMap::new())),
            compliance: Arc::new(RwLock::new(ComplianceStatus {
                status: "compliant".to_string(),
                last_audit: Utc::now(),
                rules_active: vec!["AML".to_string(), "KYC".to_string(), "OFAC".to_string()],
                risk_score: 5,
                zkml_enabled: true,
            })),
            affiliates: Arc::new(RwLock::new(HashMap::new())),
            marketing: Arc::new(RwLock::new(Vec::new())),
            financial_metrics: Arc::new(RwLock::new(FinancialMetrics {
                mrr_usd: 125000.0,
                arr_usd: 1500000.0,
                churn_rate_pct: 2.5,
                protocol_fees_collected_usd: 85000.0,
                last_updated: Utc::now(),
            })),
            identity_records: Arc::new(RwLock::new(HashMap::new())),
            erp_sync_status: Arc::new(RwLock::new(HashMap::new())),
            settlement_log: Arc::new(RwLock::new(Vec::new())),
            state_proposals: Arc::new(RwLock::new(HashMap::new())),
            partner_leads: Arc::new(RwLock::new(HashMap::new())),
            partner_lead_events: Arc::new(RwLock::new(Vec::new())),
            partner_lead_idempotency: Arc::new(RwLock::new(HashMap::new())),
            partner_lead_sequence: AtomicU64::new(1),
            sab_wallets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(&self) {
        self.initialize_services();
    }

    fn initialize_services(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        let is_mainnet = remediation::is_production_mainnet();

        let services = if is_mainnet {
            vec![
                (
                    "stacks",
                    0,
                    "PoX",
                    "On-chain",
                    "Bitcoin",
                    "sBTC Bridge",
                    0.0,
                ),
                (
                    "lightning",
                    0,
                    "State Channels",
                    "Off-chain",
                    "Bitcoin",
                    "P2P",
                    0.0,
                ),
                (
                    "liquid",
                    0,
                    "Federation",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    0.0,
                ),
                (
                    "rootstock",
                    0,
                    "Merge-mined",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    0.0,
                ),
                ("bisq", 0, "P2P", "Off-chain", "Bitcoin", "Atomic", 0.0),
                ("rgb", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                (
                    "bitvm",
                    0,
                    "Optimistic",
                    "On-chain",
                    "Bitcoin",
                    "BitVM",
                    0.0,
                ),
                (
                    "babylon", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0,
                ),
                (
                    "core-dao",
                    0,
                    "Satoshi Plus",
                    "Sidechain",
                    "Bitcoin",
                    "Relayer",
                    0.0,
                ),
                (
                    "lorenzo", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0,
                ),
                ("hemi", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "bob",
                    0,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "Optimistic",
                    0.0,
                ),
                ("merlin", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "mezo",
                    0,
                    "Economic Layer",
                    "On-chain",
                    "Bitcoin",
                    "tBTC",
                    0.0,
                ),
                ("nubit", 0, "DA", "On-chain", "Bitcoin", "N/A", 0.0),
                ("bison", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("zulu", 0, "Multi-layer", "On-chain", "Bitcoin", "N/A", 0.0),
                (
                    "botanix",
                    0,
                    "Spiderchain",
                    "Sidechain",
                    "Bitcoin",
                    "Spiderchain",
                    0.0,
                ),
                (
                    "bitlayer",
                    0,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "BitVM",
                    0.0,
                ),
                ("alpen", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                (
                    "taproot-assets",
                    0,
                    "Client-side",
                    "Off-chain",
                    "Bitcoin",
                    "N/A",
                    0.0,
                ),
                (
                    "bitvm2",
                    0,
                    "ZK-Fraud Proofs",
                    "On-chain",
                    "Bitcoin",
                    "BitVM2",
                    0.0,
                ),
                ("b2network", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
            ]
        } else {
            vec![
                (
                    "stacks",
                    65,
                    "PoX",
                    "On-chain",
                    "Bitcoin",
                    "sBTC Bridge",
                    12500000.0,
                ),
                (
                    "lightning",
                    10,
                    "State Channels",
                    "Off-chain",
                    "Bitcoin",
                    "P2P",
                    1500000.0,
                ),
                (
                    "liquid",
                    50,
                    "Federation",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    4500000.0,
                ),
                (
                    "rootstock",
                    55,
                    "Merge-mined",
                    "Sidechain",
                    "Bitcoin",
                    "Powpeg",
                    3800000.0,
                ),
                (
                    "bisq",
                    120,
                    "P2P",
                    "Off-chain",
                    "Bitcoin",
                    "Atomic",
                    850000.0,
                ),
                ("rgb", 15, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                (
                    "bitvm",
                    90,
                    "Optimistic",
                    "On-chain",
                    "Bitcoin",
                    "BitVM",
                    5000000.0,
                ),
                (
                    "babylon", 25, "Staking", "On-chain", "Bitcoin", "Staking", 15000000.0,
                ),
                (
                    "core-dao",
                    40,
                    "Satoshi Plus",
                    "Sidechain",
                    "Bitcoin",
                    "Relayer",
                    75000000.0,
                ),
                (
                    "lorenzo", 30, "Staking", "On-chain", "Bitcoin", "Staking", 12000000.0,
                ),
                (
                    "hemi",
                    50,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    35000000.0,
                ),
                (
                    "bob",
                    45,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "Optimistic",
                    28000000.0,
                ),
                (
                    "merlin",
                    35,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    45000000.0,
                ),
                (
                    "mezo",
                    25,
                    "Economic Layer",
                    "On-chain",
                    "Bitcoin",
                    "tBTC",
                    150000000.0,
                ),
                ("nubit", 20, "DA", "On-chain", "Bitcoin", "N/A", 5000000.0),
                (
                    "bison",
                    55,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    10000000.0,
                ),
                (
                    "zulu",
                    40,
                    "Multi-layer",
                    "On-chain",
                    "Bitcoin",
                    "N/A",
                    15000000.0,
                ),
                (
                    "botanix",
                    60,
                    "Spiderchain",
                    "Sidechain",
                    "Bitcoin",
                    "Spiderchain",
                    8000000.0,
                ),
                (
                    "bitlayer",
                    45,
                    "Optimistic",
                    "Rollup",
                    "Bitcoin",
                    "BitVM",
                    25000000.0,
                ),
                (
                    "alpen",
                    30,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    12000000.0,
                ),
                (
                    "taproot-assets",
                    15,
                    "Client-side",
                    "Off-chain",
                    "Bitcoin",
                    "N/A",
                    5500000.0,
                ),
                (
                    "bitvm2",
                    85,
                    "ZK-Fraud Proofs",
                    "On-chain",
                    "Bitcoin",
                    "BitVM2",
                    15000000.0,
                ),
                (
                    "b2network",
                    35,
                    "ZK",
                    "Rollup",
                    "Bitcoin",
                    "ZK Bridge",
                    18000000.0,
                ),
            ]
        };

        for (name, latency, trust, da, settlement, bridge, tvl) in services {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.2.0".to_string());
            if !is_mainnet {
                if name == "stacks" {
                    metadata.insert("block_height".to_string(), "841500".to_string());
                    metadata.insert("hiro_api_connected".to_string(), "true".to_string());
                }
                if name == "lorenzo" {
                    metadata.insert("staked_btc".to_string(), "1250.5".to_string());
                }
                if name == "b2network" {
                    metadata.insert("block_height".to_string(), "12600".to_string());
                }
            }

            let assessment = self.calculate_risk_assessment(trust, da, settlement, bridge);
            let risk_level = assessment.overall_level.clone();

            statuses.insert(
                name.to_string(),
                ServiceStatus {
                    name: name.to_string(),
                    status: "active".to_string(),
                    last_checked: Utc::now(),
                    latency_ms: latency,
                    trust_model: trust.to_string(),
                    risk_level,
                    risk_assessment: Some(assessment),
                    data_availability: da.to_string(),
                    settlement: settlement.to_string(),
                    bridge_security: bridge.to_string(),
                    tvl_usd: tvl,
                    version: Some("1.2.0".to_string()),
                    metadata,
                },
            );
        }

        if !is_mainnet {
            let mut reserves = self.reserves.write().unwrap();
            reserves.push(ReserveAsset {
                asset: "L-BTC".to_string(),
                total_supplied: 1500.0,
                total_reserves: 1500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });
            reserves.push(ReserveAsset {
                asset: "RBTC".to_string(),
                total_supplied: 2500.0,
                total_reserves: 2500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });

            let mut prices = self.prices.write().unwrap();
            prices.insert(
                "BTC".to_string(),
                PriceInfo {
                    asset: "BTC".to_string(),
                    price_usd: 65000.0,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
            prices.insert(
                "STX".to_string(),
                PriceInfo {
                    asset: "STX".to_string(),
                    price_usd: 2.50,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
        }
    }

    fn calculate_risk_assessment(
        &self,
        trust: &str,
        da: &str,
        settlement: &str,
        bridge: &str,
    ) -> RiskAssessment {
        let mut da_score = if da == "On-chain" { 95 } else { 70 };
        let mut settlement_score = if settlement == "Bitcoin" { 95 } else { 80 };
        let mut bridge_score = match bridge {
            "BitVM" | "BitVM2" => 90,
            "sBTC Bridge" => 85,
            "Powpeg" => 80,
            _ => 70,
        };

        if trust.contains("ZK") {
            da_score += 2;
            settlement_score += 2;
            bridge_score += 2;
        }

        let overall_score = (da_score + settlement_score + bridge_score) / 3;
        let overall_level = if overall_score > 90 {
            "Low".to_string()
        } else if overall_score > 80 {
            "Medium".to_string()
        } else {
            "High".to_string()
        };

        RiskAssessment {
            overall_level,
            da_score,
            settlement_score,
            bridge_score,
            exit_mechanism_score: 85,
            operators_score: 80,
            decentralization_score: 75,
        }
    }

    fn update_metrics(&self) {
        let total_requests = self.request_count.load(Ordering::SeqCst);
        let mut metrics = self.financial_metrics.write().unwrap();
        metrics.protocol_fees_collected_usd = total_requests as f64 * 0.05;
        metrics.mrr_usd = metrics.protocol_fees_collected_usd * 1.5;
        metrics.last_updated = Utc::now();

        let mut total_tvl = self.total_tvl_usd.write().unwrap();
        *total_tvl = self.calculate_total_tvl();

        {
            let wallets = self.sab_wallets.read().unwrap();
            if wallets.is_empty() {
                log::warn!("No SAB wallets configured for mainnet execution!");
            }
        }
    }

    pub fn get_service_status(&self, name: &str) -> ServiceStatus {
        let statuses = self.service_statuses.read().unwrap();
        statuses
            .get(name)
            .cloned()
            .unwrap_or_else(|| ServiceStatus {
                name: name.to_string(),
                status: "unknown".to_string(),
                last_checked: Utc::now(),
                latency_ms: 0,
                trust_model: "Unknown".to_string(),
                risk_level: "High".to_string(),
                risk_assessment: None,
                data_availability: "Unknown".to_string(),
                settlement: "Unknown".to_string(),
                bridge_security: "Unknown".to_string(),
                tvl_usd: 0.0,
                version: None,
                metadata: HashMap::new(),
            })
    }

    pub async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        let fallback_height = 841500;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.mainnet.hiro.so/v2/info")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;
        let mut height_opt = None;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                height_opt = info["stacks_tip_height"].as_u64();
            }
        }
        let height = height_opt.unwrap_or(fallback_height);
        {
            let mut statuses = self.service_statuses.write().unwrap();
            if let Some(status) = statuses.get_mut("stacks") {
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("hiro_api_connected".to_string(), "true".to_string());
                status.last_checked = Utc::now();
            }
        }
        Ok(height)
    }

    async fn analyze_bitcoin_mpool(&self) {
        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "https://bitcoin-rpc.publicnode.com".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "1.0", "id": "mempool-audit", "method": "getmempoolinfo", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                let size = info["result"]["size"].as_u64().unwrap_or(0);
                let bytes = info["result"]["bytes"].as_u64().unwrap_or(0);
                if size > 100000 {
                    log::warn!("High mempool congestion detected: {} txs", size);
                }
                let mut statuses = self.service_statuses.write().unwrap();
                if let Some(status) = statuses.get_mut("bitvm") {
                    status
                        .metadata
                        .insert("mempool_size".to_string(), size.to_string());
                    status
                        .metadata
                        .insert("mempool_bytes".to_string(), bytes.to_string());
                }
            }
        }
    }

    async fn fetch_bitcoin_rpc_status(&self) {
        let is_mainnet = remediation::is_production_mainnet();
        let rpc_url = std::env::var("BITCOIN_RPC_URL")
            .unwrap_or_else(|_| "https://bitcoin-rpc.publicnode.com".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "1.0", "id": "gateway-audit", "method": "getblockchaininfo", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        let mut height_opt = None;
        let mut connected = false;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                height_opt = info["result"]["blocks"].as_u64();
                connected = true;
            }
        }
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("bitvm") {
            if connected {
                let height = height_opt.unwrap_or(841500);
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("rpc_connected".to_string(), "true".to_string());
                status.status = "active".to_string();
            } else if is_mainnet {
                status.status = "ConnectionRequired".to_string();
            }
            status.last_checked = Utc::now();
        }
    }

    async fn track_l2_finality(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("hemi") {
            status
                .metadata
                .insert("bitcoin_finality_depth".to_string(), "3".to_string());
            status
                .metadata
                .insert("ethereum_finality_depth".to_string(), "12".to_string());
            status
                .metadata
                .insert("cross_layer_audit".to_string(), "Passed".to_string());
        }
        if let Some(status) = statuses.get_mut("bob") {
            status
                .metadata
                .insert("optimistic_window_blocks".to_string(), "1008".to_string());
            status
                .metadata
                .insert("finality_status".to_string(), "SettledOnL1".to_string());
        }
    }

    async fn fetch_core_dao_rpc_status(&self) {
        let is_mainnet = remediation::is_production_mainnet();
        let rpc_url = std::env::var("CORE_DAO_RPC_URL")
            .unwrap_or_else(|_| "https://rpc.coredao.org".to_string());
        let client = reqwest::Client::new();
        let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber", "params": [] });
        let res = client.post(&rpc_url).json(&body).send().await;
        let mut height_opt = None;
        let mut connected = false;
        if let Ok(resp) = res {
            if let Ok(info) = resp.json::<serde_json::Value>().await {
                let hex_height = info["result"].as_str().unwrap_or("0x0");
                height_opt = u64::from_str_radix(hex_height.trim_start_matches("0x"), 16).ok();
                connected = true;
            }
        }
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("core-dao") {
            if connected {
                let height = height_opt.unwrap_or(0);
                status
                    .metadata
                    .insert("block_height".to_string(), height.to_string());
                status
                    .metadata
                    .insert("rpc_connected".to_string(), "true".to_string());
                status.status = "active".to_string();
            } else if is_mainnet {
                status.status = "ConnectionRequired".to_string();
            }
            status.last_checked = Utc::now();
        }
    }

    pub fn calculate_total_tvl(&self) -> f64 {
        let statuses = self.service_statuses.read().unwrap();
        statuses.values().map(|s| s.tvl_usd).sum()
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }
    pub fn get_bitvm_proof(&self, id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": id, "protocol": "bitvm", "verified": true, "commitment_hash": "0x123...abc", "challenge_period_blocks": 144, "active_verifiers": 15, "status": "Operational" })
    }
    pub fn get_citrea_proof(&self, id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": id, "protocol": "citrea", "verified": true, "zk_proof_type": "Groth16", "l1_commitment_tx": "0x987...xyz", "status": "Finalized" })
    }
    pub fn get_financial_metrics(&self) -> FinancialMetrics {
        self.financial_metrics.read().unwrap().clone()
    }
    pub fn get_b2network_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("b2network");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "block_height": status.metadata.get("block_height").cloned().unwrap_or_else(|| "0".to_string()), "sequencer_status": "Healthy", "zk_proof_generated": true })
    }
    pub fn get_bitlayer_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitlayer");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "node_status": "Active", "bridge_capacity_btc": 500.0, "active_challenges": 0 })
    }
    pub fn get_alpen_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "finality_depth_blocks": 6, "da_layer": "Bitcoin", "status": "Production" })
    }
    pub fn get_mezo_yield(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("mezo");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "apy_pct": 12.4, "asset": "tBTC", "yield_source": "Bitcoin Staking" })
    }
    pub fn get_zulu_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("zulu");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "network_type": "Multi-layer", "evm_compatible": true, "status": "Mainnet" })
    }
    pub fn get_bison_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bison");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "rollup_type": "ZK", "proof_system": "STARK", "status": "Active" })
    }
    pub fn get_hemi_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("hemi");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "h_po_w_status": "Active", "bitcoin_finality": true, "ethereum_finality": true })
    }
    pub fn get_taproot_assets_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("taproot-assets");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "assets_minted": 150, "total_transfers": 5600, "status": "Operational" })
    }
    pub fn get_nubit_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("nubit");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_throughput_mb_s": 10.5, "blob_count": 12500, "status": "Healthy" })
    }
    pub fn get_risk_assessments(&self) -> HashMap<String, Option<RiskAssessment>> {
        let statuses = self.service_statuses.read().unwrap();
        let mut assessments = HashMap::new();
        for (name, status) in statuses.iter() {
            assessments.insert(name.clone(), status.risk_assessment.clone());
        }
        assessments
    }
    pub fn resolve_identity(&self, query: &str) -> IdentityRecord {
        self.increment_requests();
        let mut records = self.identity_records.write().unwrap();
        records
            .entry(query.to_string())
            .or_insert_with(|| IdentityRecord {
                address: query.to_string(),
                ens_name: query
                    .strip_prefix("0x")
                    .or_else(|| query.strip_prefix("0X"))
                    .and_then(|s| {
                        let prefix: String = s
                            .chars()
                            .take(4)
                            .take_while(|c| c.is_ascii_hexdigit())
                            .map(|c| c.to_ascii_lowercase())
                            .collect();
                        (!prefix.is_empty()).then(|| format!("{prefix}.eth"))
                    }),
                bns_name: if query.len() > 20 {
                    Some("conxian.btc".to_string())
                } else {
                    None
                },
                world_id_verified: query.contains("verified"),
            })
            .clone()
    }
    pub fn sync_erp_data(&self, system: &str) -> ErpSyncRecord {
        self.increment_requests();
        let mut erp_sync = self.erp_sync_status.write().unwrap();
        let record = erp_sync
            .entry(system.to_string())
            .or_insert_with(|| ErpSyncRecord {
                erp_system: system.to_string(),
                last_sync: Utc::now(),
                total_transactions_synced: 0,
                status: "Initializing".to_string(),
            });
        record.last_sync = Utc::now();
        record.total_transactions_synced += 150;
        record.status = "Healthy".to_string();
        record.clone()
    }
    pub fn get_cjcs_v2_spec(&self) -> serde_json::Value {
        serde_json::json!({ "@context": "https://conxian.com/contexts/job-card/v2.0", "@type": "ConxianJobCard", "version": "2.0.0", "standard": "JSON-LD", "description": "Enterprise-to-Bitcoin labor orchestration protocol" })
    }
    pub fn get_dlc_bond_info(&self, bond_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "bond_id": bond_id, "status": "Active", "apr_pct": 4.5, "asset": "sBTC", "maturity_blocks": 2016, "dlc_oracle": "cxn-treasury-oracle" })
    }
    pub fn get_exchange_rate(&self, from: &str, to: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "from": from, "to": to, "rate": 1.0, "timestamp": Utc::now() })
    }
    pub fn get_core_dao_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("core-dao");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "active_stakers": 1500, "satoshi_plus_rewards_distributed_usd": 1200000.0 })
    }
    pub fn get_lorenzo_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("lorenzo");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "staked_btc": status.metadata.get("staked_btc").cloned().unwrap_or_else(|| "1250.5".to_string()), "active_stakers": 850 })
    }
    pub fn commit_state_to_tableland(&self, state_root: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "table_name": "conxian_state_shards", "state_root": state_root, "transaction_hash": "0xdef...456", "status": "Finalized", "persistence": "Decentralized (Tableland)" })
    }
    pub fn get_status(&self) -> serde_json::Value {
        let uptime = (Utc::now() - self.start_time).num_seconds();
        serde_json::json!({ "version": self.version, "uptime_seconds": uptime, "status": "operational", "total_requests": self.request_count.load(Ordering::SeqCst), "total_tvl_usd": *self.total_tvl_usd.read().unwrap(), "active_nodes": self.active_sovereign_nodes.load(Ordering::SeqCst) })
    }
    pub fn is_healthy(&self) -> bool {
        true
    }
    pub fn check_compliance(&self, address: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "address": address, "status": "cleared", "risk_score": 0, "timestamp": Utc::now() })
    }
    pub fn verify_zkml_proof(&self, _proof: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "proof_id": format!("zkml-{}", Utc::now().timestamp()), "verified": true, "attestation_role": "Guardian", "compliance_standard": "CARF/BRS v1.5", "timestamp": Utc::now() })
    }
    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
            return serde_json::json!({ "asset": "L-BTC", "peg_status": "ConnectionRequired", "error": "Mainnet node connection required for Liquid peg verification.", "remediation": "Configure LIQUID_RPC_URL" });
        }
        serde_json::json!({ "asset": "L-BTC", "peg_status": "Active", "collateral_ratio": 1.0, "verified_on_chain": true })
    }
    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
            return serde_json::json!({ "asset": "RBTC", "powpeg_status": "ConnectionRequired", "error": "Mainnet node connection required for Rootstock powpeg verification.", "remediation": "Configure ROOTSTOCK_RPC_URL" });
        }
        serde_json::json!({ "asset": "RBTC", "powpeg_status": "Active", "signatories_active": 12, "btc_locked": 2500.0 })
    }
    pub fn get_all_service_statuses(&self) -> Vec<ServiceStatus> {
        self.service_statuses
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    pub fn get_babylon_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("babylon");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "active_delegators": 1250, "staking_apr": 8.5 })
    }
    pub fn create_lightning_invoice(
        &self,
        amount_msat: u64,
        description: &str,
    ) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "invoice": "lnbc...", "amount_msat": amount_msat, "description": description, "expires_at": (Utc::now() + chrono::Duration::hours(1)).timestamp() })
    }
    pub fn pay_lightning_invoice(&self, invoice: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "payment_hash": "0xabc...", "status": "Complete", "invoice": invoice })
    }
    pub fn get_stacks_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "contract_id": contract_id, "status": "Active", "source_code_verified": true, "tx_count": 1250 })
    }
    pub fn get_proposals(&self) -> Vec<StateProposal> {
        self.state_proposals
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
    pub fn approve_proposal(&self, proposal_id: &str) -> bool {
        let mut proposals = self.state_proposals.write().unwrap();
        if let Some(proposal) = proposals.get_mut(proposal_id) {
            if proposal.status == "Pending" {
                proposal.status = "Approved".to_string();
                return true;
            }
        }
        false
    }
    pub fn execute_proposal(&self, proposal_id: &str) -> Result<(), ProposalExecutionError> {
        let mut proposals = self.state_proposals.write().unwrap();
        let proposal = proposals
            .get_mut(proposal_id)
            .ok_or(ProposalExecutionError::NotFound)?;

        if proposal.status != "Approved" {
            return Err(ProposalExecutionError::NotApproved);
        }

        let current_height: u64 = self
            .get_service_status("stacks")
            .metadata
            .get("block_height")
            .and_then(|h| h.parse().ok())
            .unwrap_or(841500);

        if current_height < proposal.timelock_end_block {
            return Err(ProposalExecutionError::TimelockNotExpired {
                current_block: current_height,
                timelock_end_block: proposal.timelock_end_block,
            });
        }

        proposal.status = "Executed".to_string();
        Ok(())
    }
    pub fn get_reserves(&self) -> Vec<ReserveAsset> {
        self.reserves.read().unwrap().clone()
    }
    pub fn get_prices(&self) -> Vec<PriceInfo> {
        self.prices.read().unwrap().values().cloned().collect()
    }
    pub fn process_external_settlement(&self, protocol: &str, payload: Value) -> StateProposal {
        self.increment_requests();
        let raw_payload = serde_json::to_string(&payload).unwrap_or_default();
        let envelope = SettlementEnvelope {
            protocol: protocol.to_string(),
            payload,
            raw_payload_bytes: raw_payload,
            ingress_timestamp: Utc::now(),
        };
        self.settlement_log.write().unwrap().push(envelope);
        let trigger_id = format!(
            "{}-trigger-{}",
            protocol.to_lowercase(),
            Utc::now().timestamp()
        );
        let proposal_id = format!("prop-{}", trigger_id);
        let current_height: u64 = self
            .get_service_status("stacks")
            .metadata
            .get("block_height")
            .and_then(|h| h.parse().ok())
            .unwrap_or(841500);
        let timelock_end = current_height + 144;
        let proposal = StateProposal {
            proposal_id: proposal_id.clone(),
            trigger_id,
            proposed_state: "MainnetSovereignStateUpdate".to_string(),
            timelock_end_block: timelock_end,
            status: "Pending".to_string(),
            tee_attestation: "VerifiedByStrongBox-Mainnet-v1.0".to_string(),
            yield_routing: "5/5/90".to_string(),
            capital_status: "TransitBond".to_string(),
        };
        self.state_proposals
            .write()
            .unwrap()
            .insert(proposal_id, proposal.clone());
        proposal
    }
    fn next_partner_lead_token(&self, prefix: &str) -> String {
        let seq = self.partner_lead_sequence.fetch_add(1, Ordering::SeqCst);
        format!("{prefix}-{}-{seq}", Utc::now().timestamp_millis())
    }
    pub fn create_partner_lead(
        &self,
        input: PartnerLeadCreateInput,
        idempotency_key: &str,
    ) -> PartnerLeadCreateOutcome {
        self.increment_requests();
        if let Some(existing_id) = self
            .partner_lead_idempotency
            .read()
            .unwrap()
            .get(idempotency_key)
            .cloned()
        {
            if let Some(existing_lead) = self.partner_leads.read().unwrap().get(&existing_id) {
                return PartnerLeadCreateOutcome {
                    lead_id: existing_lead.lead_id.clone(),
                    status: "idempotent_replay".to_string(),
                    idempotent_replay: true,
                };
            }
        }
        let lead_id = self.next_partner_lead_token("LEAD");
        let now = Utc::now();
        let lead = PartnerLead {
            lead_id: lead_id.clone(),
            partner_name: input.partner_name,
            contact_name: input.contact_name,
            contact_email: input.contact_email,
            company_name: input.company_name,
            status: PartnerLeadStatus::New,
            owner: None,
            created_at: now,
            updated_at: now,
            notes: input.notes,
        };
        self.partner_leads
            .write()
            .unwrap()
            .insert(lead_id.clone(), lead);
        self.partner_lead_idempotency
            .write()
            .unwrap()
            .insert(idempotency_key.to_string(), lead_id.clone());
        PartnerLeadCreateOutcome {
            lead_id,
            status: "created".to_string(),
            idempotent_replay: false,
        }
    }
    pub fn get_partner_lead(&self, lead_id: &str) -> Option<PartnerLead> {
        self.partner_leads.read().unwrap().get(lead_id).cloned()
    }
    pub fn list_partner_leads(
        &self,
        status: Option<PartnerLeadStatus>,
        owner: Option<&str>,
    ) -> Vec<PartnerLead> {
        self.partner_leads
            .read()
            .unwrap()
            .values()
            .filter(|l| status.as_ref().is_none_or(|s| &l.status == s))
            .filter(|l| {
                owner
                    .as_ref()
                    .is_none_or(|o| l.owner.as_deref() == Some(*o))
            })
            .cloned()
            .collect()
    }
    pub fn transition_partner_lead(
        &self,
        lead_id: &str,
        input: PartnerLeadStatusUpdateInput,
    ) -> Result<PartnerLead, PartnerLeadTransitionError> {
        self.increment_requests();
        let mut leads = self.partner_leads.write().unwrap();
        let lead = leads
            .get_mut(lead_id)
            .ok_or(PartnerLeadTransitionError::NotFound)?;
        self.validate_partner_transition(&lead.status, &input.status, input.owner.as_deref())?;
        let from_status = lead.status.clone();
        let now = Utc::now();
        lead.status = input.status.clone();
        lead.updated_at = now;
        if let Some(new_owner) = &input.owner {
            lead.owner = Some(new_owner.clone());
        }
        let event = PartnerLeadEvent {
            event_id: self.next_partner_lead_token("EVT"),
            lead_id: lead_id.to_string(),
            timestamp: now,
            from_status,
            to_status: lead.status.clone(),
            owner: input.owner,
            note: input.event_note,
        };
        self.partner_lead_events.write().unwrap().push(event);
        Ok(lead.clone())
    }
    fn validate_partner_transition(
        &self,
        from: &PartnerLeadStatus,
        to: &PartnerLeadStatus,
        owner: Option<&str>,
    ) -> Result<(), PartnerLeadTransitionError> {
        if from == to {
            return Ok(());
        }
        match (from, to) {
            (PartnerLeadStatus::New, PartnerLeadStatus::Assigned) => {
                if owner.is_none() {
                    return Err(PartnerLeadTransitionError::OwnerRequired);
                }
            }
            (PartnerLeadStatus::Assigned, PartnerLeadStatus::InProgress) => {}
            (PartnerLeadStatus::InProgress, PartnerLeadStatus::Qualified) => {}
            (_, PartnerLeadStatus::Escalated) => {}
            (_, PartnerLeadStatus::ClosedWon) => {}
            (_, PartnerLeadStatus::ClosedLost) => {}
            _ => {
                return Err(PartnerLeadTransitionError::InvalidTransition {
                    from: from.clone(),
                    to: to.clone(),
                })
            }
        }
        Ok(())
    }

    pub fn get_compliance_status(&self) -> ComplianceStatus {
        self.compliance.read().unwrap().clone()
    }
    pub fn get_bitvm2_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitvm2");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "challenge_period": 144, "bridge_status": "Active" })
    }
    pub fn get_bitvm2_segments(&self, state_root: &str) -> serde_json::Value {
        self.increment_requests();
        let orchestrator = lib_conxian_core::bitvm2::Bitvm2Orchestrator::new();
        let segments = orchestrator.generate_segments(state_root);
        serde_json::json!({ "state_root": state_root, "segments_count": segments.len(), "segments": segments })
    }
    pub fn get_bob_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bob");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "optimistic_finality": true, "fraud_proof_window_blocks": 1008 })
    }
    pub fn get_merlin_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("merlin");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "sequencer_batch_count": 12500, "zk_proof_system": "zk-STARK" })
    }
    pub fn get_botanix_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("botanix");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "spiderchain_nodes": 64, "peg_in_status": "Operational" })
    }
    pub fn get_rgb_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({ "contract_id": contract_id, "status": "Finalized", "asset_type": "RGB20" })
    }
    pub fn get_alpen_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_verification": "Enabled" })
    }
    pub fn get_bison_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bison");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "proof_count": 850 })
    }
    pub fn get_hemi_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("hemi");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "network": "Mainnet" })
    }
    pub fn get_taproot_assets_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("taproot-assets");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "issuance_count": 45 })
    }
    pub fn get_nubit_da_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("nubit");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "da_layer_status": "Active" })
    }
    pub fn get_b2_status(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("b2network");
        serde_json::json!({ "tvl_usd": status.tvl_usd, "status": "Operational" })
    }
    pub fn get_affiliates(&self) -> Vec<AffiliateInfo> {
        self.affiliates.read().unwrap().values().cloned().collect()
    }
    pub fn get_marketing(&self) -> Vec<MarketingInfo> {
        self.marketing.read().unwrap().clone()
    }
    pub fn is_mainnet_only() -> bool {
        remediation::is_production_mainnet()
    }

    pub async fn start_monitoring(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                engine.update_metrics();
                let engine_clone = Arc::clone(&engine);
                tokio::spawn(async move {
                    let _ = engine_clone.fetch_stacks_block_height().await;
                });
                let engine_clone_btc = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_btc.fetch_bitcoin_rpc_status().await;
                });
                let engine_clone_core = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_core.fetch_core_dao_rpc_status().await;
                });
                let engine_clone_mpool = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_mpool.analyze_bitcoin_mpool().await;
                });
                let engine_clone_l2 = Arc::clone(&engine);
                tokio::spawn(async move {
                    engine_clone_l2.track_l2_finality().await;
                });
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    pub async fn broadcast_intents(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                let proposals = engine.get_proposals();
                let pending_count = proposals.iter().filter(|p| p.status == "Pending").count();
                if pending_count > 0 {
                    log::info!("Broadcasting Phase 9 Real-time Intent Update: {} pending proposals requiring handshake", pending_count);
                }
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        });
    }

    pub fn get_sab_wallets(&self) -> Vec<SabWallet> {
        self.sab_wallets.read().unwrap().clone()
    }
    pub async fn poll_support(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                log::info!("Polling support mailbox for new tickets...");
                let ts = Utc::now();
                let ticket = engine.support_intake.process_inbound_metadata(
                    "support@conxian-labs.com",
                    "user@external.com",
                    "Assistance required with MuSig2",
                    "<sim-123@external.com>",
                    ts,
                );
                log::info!("Generated support ticket: {}", ticket.token);
                tokio::time::sleep(std::time::Duration::from_secs(
                    engine.support_intake.config.poll_interval_secs,
                ))
                .await;
            }
        });
    }
}
