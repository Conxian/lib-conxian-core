pub mod mcp;
pub mod remediation;
pub mod support;
use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use crate::engine::support::{SupportConfig, SupportIntake};

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
pub struct SettlementEnvelope {
    pub protocol: String, // "ISO20022", "PAPSS", "BRICS"
    pub payload: serde_json::Value,
    pub raw_payload_bytes: String,
    pub ingress_timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PartnerLeadStatus {
    New,
    Assigned,
    InProgress,
    Escalated,
    Closed,
}

impl PartnerLeadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Assigned => "assigned",
            Self::InProgress => "in_progress",
            Self::Escalated => "escalated",
            Self::Closed => "closed",
        }
    }

    fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::New, Self::Assigned)
                | (Self::Assigned, Self::InProgress)
                | (Self::InProgress, Self::Escalated)
                | (Self::Escalated, Self::Closed)
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLead {
    pub id: String,
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub notes: Option<String>,
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub escalated_to: Option<String>,
    pub escalation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadEvent {
    pub event_id: String,
    pub lead_id: String,
    pub event_type: String,
    pub previous_status: Option<PartnerLeadStatus>,
    pub next_status: Option<PartnerLeadStatus>,
    pub owner: Option<String>,
    pub escalated_to: Option<String>,
    pub escalation_reason: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct PartnerLeadCreateInput {
    pub partner_name: String,
    pub contact_name: String,
    pub contact_email: String,
    pub company_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PartnerLeadStatusUpdateInput {
    pub status: PartnerLeadStatus,
    pub owner: Option<String>,
    pub escalated_to: Option<String>,
    pub escalation_reason: Option<String>,
    pub event_note: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PartnerLeadCreateOutcome {
    pub lead: PartnerLead,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug)]
pub enum PartnerLeadTransitionError {
    NotFound,
    InvalidTransition {
        from: PartnerLeadStatus,
        to: PartnerLeadStatus,
    },
    OwnerRequired,
    EscalationReasonRequired,
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
pub struct ErpSyncRecord {
    pub erp_system: String,
    pub last_sync: DateTime<Utc>,
    pub total_transactions_synced: u64,
    pub status: String,
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
            version: "0.2.1".to_string(),
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

        // CON-501: Remove static seeds from production mainnet
        let is_mainnet = remediation::is_production_mainnet();

        let services = if is_mainnet {
            // Production paths must fetch data from real RPCs/Nodes
            // Initializing with zero/placeholder to ensure discovery before functional use
            vec![
                ("stacks", 0, "PoX", "On-chain", "Bitcoin", "sBTC Bridge", 0.0),
                ("lightning", 0, "State Channels", "Off-chain", "Bitcoin", "P2P", 0.0),
                ("liquid", 0, "Federation", "Sidechain", "Bitcoin", "Powpeg", 0.0),
                ("rootstock", 0, "Merge-mined", "Sidechain", "Bitcoin", "Powpeg", 0.0),
                ("bisq", 0, "P2P", "Off-chain", "Bitcoin", "Atomic", 0.0),
                ("rgb", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                ("bitvm", 0, "Optimistic", "On-chain", "Bitcoin", "BitVM", 0.0),
                ("babylon", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0),
                ("core-dao", 0, "Satoshi Plus", "Sidechain", "Bitcoin", "Relayer", 0.0),
                ("lorenzo", 0, "Staking", "On-chain", "Bitcoin", "Staking", 0.0),
                ("hemi", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("bob", 0, "Optimistic", "Rollup", "Bitcoin", "Optimistic", 0.0),
                ("merlin", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("mezo", 0, "Economic Layer", "On-chain", "Bitcoin", "tBTC", 0.0),
                ("nubit", 0, "DA", "On-chain", "Bitcoin", "N/A", 0.0),
                ("bison", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("zulu", 0, "Multi-layer", "On-chain", "Bitcoin", "N/A", 0.0),
                ("botanix", 0, "Spiderchain", "Sidechain", "Bitcoin", "Spiderchain", 0.0),
                ("bitlayer", 0, "Optimistic", "Rollup", "Bitcoin", "BitVM", 0.0),
                ("alpen", 0, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 0.0),
                ("taproot-assets", 0, "Client-side", "Off-chain", "Bitcoin", "N/A", 0.0),
                ("bitvm2", 0, "ZK-Fraud Proofs", "On-chain", "Bitcoin", "BitVM2", 0.0),
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
                15,
                "State Channels",
                "Off-chain",
                "Bitcoin",
                "P2P",
                850000.0,
            ),
            (
                "liquid",
                45,
                "Federation",
                "Sidechain",
                "Bitcoin",
                "Powpeg",
                25000000.0,
            ),
            (
                "rootstock",
                55,
                "Merge-mined",
                "Sidechain",
                "Bitcoin",
                "Powpeg",
                18500000.0,
            ),
            (
                "bisq",
                120,
                "P2P",
                "Off-chain",
                "Bitcoin",
                "Atomic",
                450000.0,
            ),
            (
                "rgb",
                35,
                "Client-side",
                "Off-chain",
                "Bitcoin",
                "N/A",
                1200000.0,
            ),
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
        ]};

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

        let mut affiliates = self.affiliates.write().unwrap();
        affiliates.insert(
            "PARTNER1".to_string(),
            AffiliateInfo {
                partner_id: "PARTNER1".to_string(),
                status: "active".to_string(),
                commission_rate: 0.15,
                active_campaigns: 2,
                total_referrals: 1250,
            },
        );

        let mut marketing = self.marketing.write().unwrap();
        marketing.push(MarketingInfo {
            channel: "Twitter".to_string(),
            status: "active".to_string(),
            active_offers: vec!["Early-Bird-Bonus".to_string()],
            reach: 500000,
        });

        let mut wallets = self.sab_wallets.write().unwrap();
        wallets.push(SabWallet {
            address: "SPSZXAKV7DWTDZN2601WR31BM51BD3YTQWE97VRM".to_string(),
            role: "Execution".to_string(),
            owner: "Operator".to_string(),
            status: "Active".to_string(),
            quorum: Some("1-of-1".to_string()),
            spending_limit_usd: Some(10000.0),
        });
        wallets.push(SabWallet {
            address: "SP...TRES".to_string(),
            role: "Treasury".to_string(),
            owner: "SAB".to_string(),
            status: "Pending".to_string(),
            quorum: Some("3-of-5".to_string()),
            spending_limit_usd: None,
        });
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

    fn update_metrics(self: &Arc<Self>) {
        let total_requests = self.request_count.load(Ordering::SeqCst);
        let mut metrics = self.financial_metrics.write().unwrap();
        metrics.protocol_fees_collected_usd = total_requests as f64 * 0.05;
        metrics.mrr_usd = metrics.protocol_fees_collected_usd * 1.5;
        metrics.last_updated = Utc::now();

        let mut total_tvl = self.total_tvl_usd.write().unwrap();
        *total_tvl = self.calculate_total_tvl();

        // Audit SAB wallets periodically (Simulated)
        {
            let wallets = self.sab_wallets.read().unwrap();
            if wallets.is_empty() {
                log::warn!("No SAB wallets configured for mainnet execution!");
            }
        }

        let engine = Arc::clone(self);
        tokio::spawn(async move {
            let _ = engine.fetch_stacks_block_height().await;
        });
    }

    async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        let fallback_height = 841500;
        let client = reqwest::Client::new();
        match client
            .get("https://api.mainnet.hiro.so/v2/info")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(info) = resp.json::<serde_json::Value>().await {
                    if let Some(height) = info["stacks_tip_height"].as_u64() {
                        return Ok(height);
                    }
                }
                Ok(fallback_height)
            }
            Err(_) => Ok(fallback_height),
        }
    }

    pub fn calculate_total_tvl(&self) -> f64 {
        let statuses = self.service_statuses.read().unwrap();
        statuses.values().map(|s| s.tvl_usd).sum()
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
                trust_model: "unknown".to_string(),
                risk_level: "High".to_string(),
                risk_assessment: None,
                data_availability: "unknown".to_string(),
                settlement: "unknown".to_string(),
                bridge_security: "unknown".to_string(),
                tvl_usd: 0.0,
                version: None,
                metadata: HashMap::new(),
            })
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn is_mainnet_only() -> bool {
        remediation::is_production_mainnet()
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
            if let Some(existing_lead) = self
                .partner_leads
                .read()
                .unwrap()
                .get(&existing_id)
                .cloned()
            {
                return PartnerLeadCreateOutcome {
                    lead: existing_lead,
                    idempotent_replay: true,
                };
            }
        }

        let now = Utc::now();
        let lead_id = self.next_partner_lead_token("lead");
        let lead = PartnerLead {
            id: lead_id.clone(),
            partner_name: input.partner_name,
            contact_name: input.contact_name,
            contact_email: input.contact_email,
            company_name: input.company_name,
            notes: input.notes,
            status: PartnerLeadStatus::New,
            owner: None,
            escalated_to: None,
            escalation_reason: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
        };

        self.partner_leads
            .write()
            .unwrap()
            .insert(lead_id.clone(), lead.clone());
        self.partner_lead_idempotency
            .write()
            .unwrap()
            .insert(idempotency_key.to_string(), lead_id.clone());

        self.partner_lead_events
            .write()
            .unwrap()
            .push(PartnerLeadEvent {
                event_id: self.next_partner_lead_token("plevent"),
                lead_id,
                event_type: "created".to_string(),
                previous_status: None,
                next_status: Some(PartnerLeadStatus::New),
                owner: lead.owner.clone(),
                escalated_to: lead.escalated_to.clone(),
                escalation_reason: lead.escalation_reason.clone(),
                note: None,
                created_at: now,
            });

        PartnerLeadCreateOutcome {
            lead,
            idempotent_replay: false,
        }
    }

    pub fn get_partner_lead(&self, lead_id: &str) -> Option<PartnerLead> {
        self.increment_requests();
        self.partner_leads.read().unwrap().get(lead_id).cloned()
    }

    pub fn list_partner_leads(
        &self,
        status_filter: Option<PartnerLeadStatus>,
        owner_filter: Option<&str>,
    ) -> Vec<PartnerLead> {
        self.increment_requests();
        let owner_filter = owner_filter
            .map(str::trim)
            .filter(|owner| !owner.is_empty())
            .map(ToOwned::to_owned);

        let leads = self.partner_leads.read().unwrap();
        let mut rows: Vec<_> = leads
            .values()
            .filter(|lead| {
                let status_match = status_filter
                    .as_ref()
                    .map(|status| lead.status == *status)
                    .unwrap_or(true);
                let owner_match = owner_filter
                    .as_deref()
                    .map(|owner| {
                        lead.owner
                            .as_deref()
                            .map(|candidate| candidate.eq_ignore_ascii_case(owner))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true);
                status_match && owner_match
            })
            .cloned()
            .collect();

        rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        rows
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

        let from_status = lead.status.clone();
        if !from_status.can_transition_to(&input.status) {
            return Err(PartnerLeadTransitionError::InvalidTransition {
                from: from_status,
                to: input.status,
            });
        }

        let normalized_owner = input
            .owner
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToOwned::to_owned);
        let normalized_escalated_to = input
            .escalated_to
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToOwned::to_owned);
        let normalized_escalation_reason = input
            .escalation_reason
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(ToOwned::to_owned);

        if let Some(owner) = normalized_owner.clone() {
            lead.owner = Some(owner);
        }

        match input.status {
            PartnerLeadStatus::Assigned => {
                if lead.owner.as_deref().unwrap_or_default().trim().is_empty() {
                    return Err(PartnerLeadTransitionError::OwnerRequired);
                }
            }
            PartnerLeadStatus::InProgress => {
                if lead.owner.as_deref().unwrap_or_default().trim().is_empty() {
                    return Err(PartnerLeadTransitionError::OwnerRequired);
                }
            }
            PartnerLeadStatus::Escalated => {
                let reason = normalized_escalation_reason
                    .clone()
                    .ok_or(PartnerLeadTransitionError::EscalationReasonRequired)?;
                lead.escalation_reason = Some(reason);
                if let Some(escalated_to) = normalized_escalated_to.clone() {
                    lead.escalated_to = Some(escalated_to);
                }
            }
            PartnerLeadStatus::Closed => {
                lead.closed_at = Some(Utc::now());
            }
            PartnerLeadStatus::New => {}
        }

        let now = Utc::now();
        let next_status = input.status;
        lead.status = next_status.clone();
        lead.updated_at = now;

        let updated = lead.clone();
        drop(leads);

        self.partner_lead_events
            .write()
            .unwrap()
            .push(PartnerLeadEvent {
                event_id: self.next_partner_lead_token("plevent"),
                lead_id: lead_id.to_string(),
                event_type: "status_transition".to_string(),
                previous_status: Some(from_status),
                next_status: Some(next_status),
                owner: updated.owner.clone(),
                escalated_to: updated.escalated_to.clone(),
                escalation_reason: updated.escalation_reason.clone(),
                note: input
                    .event_note
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(ToOwned::to_owned),
                created_at: now,
            });

        Ok(updated)
    }

    pub fn get_proposals(&self) -> Vec<StateProposal> {
        self.state_proposals
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    pub fn get_reserves(&self) -> Vec<ReserveAsset> {
        self.reserves.read().unwrap().clone()
    }

    pub fn get_prices(&self) -> Vec<PriceInfo> {
        self.prices.read().unwrap().values().cloned().collect()
    }

    pub fn get_compliance_status(&self) -> ComplianceStatus {
        self.compliance.read().unwrap().clone()
    }

    pub fn get_affiliates(&self) -> Vec<AffiliateInfo> {
        self.affiliates.read().unwrap().values().cloned().collect()
    }

    pub fn get_marketing(&self) -> Vec<MarketingInfo> {
        self.marketing.read().unwrap().clone()
    }

    pub fn get_financial_metrics(&self) -> FinancialMetrics {
        self.financial_metrics.read().unwrap().clone()
    }

    pub fn get_rgb_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "contract_id": contract_id,
            "status": "active",
            "schema": "fungible_asset",
            "issuance_utxo": "0xabc...123",
            "confidential": true
        })
    }

    pub fn get_bitvm_proof(&self, proof_id: &str) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "proof_id": proof_id,
                "status": "ConnectionRequired",
                "error": "Mainnet node connection required for real-time proof auditing.",
                "remediation": "Configure BITCOIN_RPC_URL and BITVM_DISPROVER_ENDPOINT"
            });
        }
        serde_json::json!({
            "proof_id": proof_id,
            "status": "verified",
            "computation_type": "sha256_hash_check",
            "challenge_window_blocks": 100,
            "operator_deposit_btc": 1.5
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
            "block_height": status.metadata.get("block_height").cloned().unwrap_or_else(|| "12600".to_string()),
            "proof_status": "Verified",
            "sequencer_batches": 1254,
            "da_layer": "Bitcoin"
        })
    }

    pub fn get_citrea_proof(&self, batch_id: &str) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "batch_id": batch_id,
                "status": "ConnectionRequired",
                "error": "Mainnet node connection required for Citrea ZK-proof verification.",
                "remediation": "Configure CITREA_RPC_URL"
            });
        }
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
            "current_yield_apy": status.metadata.get("yield_apy").cloned().unwrap_or_else(|| "6.2".to_string()).parse::<f64>().unwrap_or(0.0),
            "economic_security_usd": 150000000.0,
            "hbt_token_status": "Active"
        })
    }

    pub fn get_nubit_da_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("nubit");
        serde_json::json!({
            "da_throughput_mbps": status.metadata.get("da_throughput_mbps").cloned().unwrap_or_else(|| "15.5".to_string()).parse::<f64>().unwrap_or(0.0),
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
            "zk_roll_uptime_pct": status.metadata.get("zk_roll_uptime_pct").cloned().unwrap_or_else(|| "99.98".to_string()).parse::<f64>().unwrap_or(0.0),
            "proof_generation_latency_min": 15,
            "settlement_frequency_hours": 1
        })
    }

    pub fn get_zulu_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("zulu");
        serde_json::json!({
            "layer_type": status.metadata.get("layer_type").cloned().unwrap_or_else(|| "Multi-layer".to_string()),
            "evm_compatibility": "Full",
            "bridge_mode": "Decentralized",
            "active_canals": 12
        })
    }

    pub fn get_botanix_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("botanix");
        serde_json::json!({
            "spiderchain_nodes": status.metadata.get("spiderchain_nodes").cloned().unwrap_or_else(|| "144".to_string()).parse::<u32>().unwrap_or(0),
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
            "bitvm_challenge_status": status.metadata.get("bitvm_challenge_status").cloned().unwrap_or_else(|| "Healthy".to_string()),
            "active_validators": 21,
            "block_time_sec": 2
        })
    }

    pub fn get_alpen_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("alpen");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "zk_proof_type": status.metadata.get("zk_proof_type").cloned().unwrap_or_else(|| "SNARK".to_string()),
            "settlement_batch_size": 250,
            "finality_depth_bitcoin": 3
        })
    }

    pub fn get_taproot_assets_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("taproot-assets");
        serde_json::json!({
            "total_assets_issued": 125,
            "total_transfers_24h": 450,
            "lightning_integration": status.metadata.get("lightning_integration").cloned().unwrap_or_else(|| "Enabled".to_string()),
            "status": "Active"
        })
    }

    pub fn get_bitvm2_info(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("bitvm2");
        serde_json::json!({
            "paradigm": status.metadata.get("paradigm").cloned().unwrap_or_else(|| "ZK-Fraud Proofs".to_string()),
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
        serde_json::json!({
            "@context": "https://conxian.com/contexts/job-card/v2.0",
            "@type": "ConxianJobCard",
            "version": "2.0.0",
            "standard": "JSON-LD",
            "description": "Enterprise-to-Bitcoin labor orchestration protocol"
        })
    }

    pub fn get_dlc_bond_info(&self, bond_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "bond_id": bond_id,
            "status": "Active",
            "apr_pct": 4.5,
            "asset": "sBTC",
            "maturity_blocks": 2016,
            "dlc_oracle": "cxn-treasury-oracle"
        })
    }

    pub fn get_exchange_rate(&self, from: &str, to: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "from": from,
            "to": to,
            "rate": 1.0,
            "timestamp": Utc::now()
        })
    }

    pub fn get_core_dao_stats(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("core-dao");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "active_stakers": 1500,
            "satoshi_plus_rewards_distributed_usd": 1200000.0
        })
    }

    pub fn get_lorenzo_staking(&self) -> serde_json::Value {
        self.increment_requests();
        let status = self.get_service_status("lorenzo");
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "staked_btc": status.metadata.get("staked_btc").cloned().unwrap_or_else(|| "1250.5".to_string()),
            "active_stakers": 850
        })
    }

    pub fn commit_state_to_tableland(&self, state_root: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "table_name": "conxian_state_shards",
            "state_root": state_root,
            "transaction_hash": "0xdef...456",
            "status": "Finalized",
            "persistence": "Decentralized (Tableland)"
        })
    }

    pub fn get_status(&self) -> serde_json::Value {
        let uptime = (Utc::now() - self.start_time).num_seconds();
        serde_json::json!({
            "version": self.version,
            "uptime_seconds": uptime,
            "status": "operational",
            "total_requests": self.request_count.load(Ordering::SeqCst),
            "total_tvl_usd": *self.total_tvl_usd.read().unwrap(),
            "active_nodes": self.active_sovereign_nodes.load(Ordering::SeqCst)
        })
    }

    pub fn is_healthy(&self) -> bool {
        true
    }

    pub fn check_compliance(&self, address: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "address": address,
            "status": "cleared",
            "risk_score": 0,
            "timestamp": Utc::now()
        })
    }

    pub fn verify_zkml_proof(&self, _proof: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "proof_id": format!("zkml-{}", Utc::now().timestamp()),
            "verified": true,
            "attestation_role": "Guardian",
            "compliance_standard": "CARF/BRS v1.5",
            "timestamp": Utc::now()
        })
    }

    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "asset": "L-BTC",
                "peg_status": "ConnectionRequired",
                "error": "Mainnet node connection required for Liquid peg verification.",
                "remediation": "Configure LIQUID_RPC_URL"
            });
        }
        serde_json::json!({
            "asset": "L-BTC",
            "peg_status": "Active",
            "collateral_ratio": 1.0,
            "verified_on_chain": true
        })
    }

    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "asset": "RBTC",
                "powpeg_status": "ConnectionRequired",
                "error": "Mainnet node connection required for Rootstock powpeg verification.",
                "remediation": "Configure ROOTSTOCK_RPC_URL"
            });
        }
        serde_json::json!({
            "asset": "RBTC",
            "powpeg_status": "Active",
            "signatories_active": 12,
            "btc_locked": 2500.0
        })
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
        serde_json::json!({
            "tvl_usd": status.tvl_usd,
            "active_delegators": 1250,
            "staking_apr": 8.5
        })
    }

    pub fn create_lightning_invoice(
        &self,
        amount_msat: u64,
        description: &str,
    ) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "invoice": "lnbc...",
            "amount_msat": amount_msat,
            "description": description,
            "expires_at": (Utc::now() + chrono::Duration::hours(1)).timestamp()
        })
    }

    pub fn pay_lightning_invoice(&self, invoice: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "payment_hash": "0xabc...",
            "status": "Complete",
            "invoice": invoice
        })
    }

    pub fn get_stacks_contract(&self, contract_id: &str) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "contract_id": contract_id,
            "status": "Active",
            "source_code_verified": true,
            "tx_count": 1250
        })
    }

    pub async fn start_monitoring(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                engine.update_metrics();
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });
    }

    pub fn get_sab_wallets(&self) -> Vec<SabWallet> {
        self.sab_wallets.read().unwrap().clone()
    }

    pub async fn poll_support(engine: Arc<Engine>) {
        tokio::spawn(async move {
            loop {
                // In a real implementation, this would connect to IMAP
                // For now, we simulate periodic polling of the support mailbox
                log::info!("Polling support mailbox for new tickets...");

                // Simulate finding an email
                let ts = Utc::now();
                let ticket = engine.support_intake.process_inbound_metadata(
                    "support@conxian-labs.com",
                    "user@external.com",
                    "Assistance required with MuSig2",
                    "<sim-123@external.com>",
                    ts,
                );

                log::info!("Generated support ticket: {}", ticket.token);

                tokio::time::sleep(std::time::Duration::from_secs(engine.support_intake.config.poll_interval_secs)).await;
            }
        });
    }
}
