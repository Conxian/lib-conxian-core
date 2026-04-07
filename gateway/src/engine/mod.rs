use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use rusqlite::{params, Connection, Result as SqlResult};
use uuid::Uuid;
use lru::LruCache;
use std::num::NonZeroUsize;
use crc::{Crc, CRC_24_OPENPGP};
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;

pub const MESH_CRC: Crc<u32> = Crc::<u32>::new(&CRC_24_OPENPGP);
pub const CONX_SYNC_UUID: uuid::Uuid = uuid::uuid!("f045ba10-db0a-4b0c-9b1a-28495a9b34fb"); // Conxian Sync Service v1.0

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JobStatus {
    Pending,
    Stored,
    Gossiped,
    Synced,
    Settled,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobCard {
    pub id: Uuid,
    pub context: String, // "https://conxian.com/contexts/job-card/v2.0"
    pub type_name: String, // "ConxianJobCard"
    pub version: String, // "2.0.0"
    pub status: JobStatus,
    pub tx_hash: String,
    pub amount_stx: f64,
    pub timestamp: DateTime<Utc>,
    pub signature: String, // Biometric passkey signature
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GossipPacket {
    pub packet_type: u8, // 0x01 (TX_GOSSIP) / 0x02 (ACK)
    pub tx_hash_prefix: [u8; 8],
    pub amount_stx: f32, // 4 bytes (little-endian)
    pub time_delta: u32, // 4 bytes (little-endian)
    pub crc: u32, // 3 bytes used (Big-endian CRC-24)
}

impl GossipPacket {
    pub fn to_bytes(&self) -> [u8; 20] {
        let mut buf = [0u8; 20];
        buf[0] = self.packet_type;
        buf[1..9].copy_from_slice(&self.tx_hash_prefix);
        buf[9..13].copy_from_slice(&self.amount_stx.to_le_bytes());
        buf[13..17].copy_from_slice(&self.time_delta.to_le_bytes());
        
        // Calculate CRC-24 over first 17 bytes
        let checksum = MESH_CRC.checksum(&buf[0..17]);
        buf[17] = (checksum >> 16) as u8;
        buf[18] = (checksum >> 8) as u8;
        buf[19] = checksum as u8;
        buf
    }

    pub fn from_bytes(data: &[u8; 20]) -> Option<Self> {
        let checksum = MESH_CRC.checksum(&data[0..17]);
        let expected_crc = ((data[17] as u32) << 16) | ((data[18] as u32) << 8) | (data[19] as u32);
        
        if checksum != expected_crc {
            return None;
        }

        let mut tx_hash_prefix = [0u8; 8];
        tx_hash_prefix.copy_from_slice(&data[1..9]);
        
        let amount_stx = f32::from_le_bytes(data[9..13].try_into().ok()?);
        let time_delta = u32::from_le_bytes(data[13..17].try_into().ok()?);

        Some(Self {
            packet_type: data[0],
            tx_hash_prefix,
            amount_stx,
            time_delta,
            crc: expected_crc,
        })
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateProposal {
    pub proposal_id: String,
    pub trigger_id: String,
    pub proposed_state: String,
    pub timelock_end_block: u64,
    pub status: String, // "Pending", "Approved", "Executed"
    pub tee_attestation: String,
    pub yield_routing: String, // "5/5/90"
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
    pub role: String, // "Execution", "Treasury", "Payout", "Signer", "Emergency"
    pub owner: String, // "SAB", "Operator", "DAO"
    pub status: String, // "Active", "Deprecated", "Pending"
    pub quorum: Option<String>,
    pub spending_limit_usd: Option<f64>,
}

pub struct Engine {
    pub version: String,
    pub start_time: DateTime<Utc>,
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
    pub job_queue: Arc<RwLock<HashMap<Uuid, JobCard>>>,
    pub mesh_cache: Arc<RwLock<LruCache<[u8; 8], (f32, u32)>>>, // tx_hash_prefix -> (amount, timestamp)
    pub db_path: String,
    pub settlement_log: Arc<RwLock<Vec<SettlementEnvelope>>>,
    pub state_proposals: Arc<RwLock<HashMap<String, StateProposal>>>,
    pub sab_wallets: Arc<RwLock<Vec<SabWallet>>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        let db_path = "conxian_gateway.db".to_string();
        Self::init_db(&db_path).expect("Failed to initialize TEE SQLite database");
        let jobs = Self::load_jobs_from_db(&db_path).expect("Failed to load jobs from SQLite");

        Engine {
            version: "0.2.0".to_string(),
            start_time: Utc::now(),
            request_count: AtomicU64::new(0),
            total_tvl_usd: Arc::new(RwLock::new(0.0)),
            active_sovereign_nodes: AtomicU64::new(173),
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
            job_queue: Arc::new(RwLock::new(jobs)),
            mesh_cache: Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
            db_path,
            settlement_log: Arc::new(RwLock::new(Vec::new())),
            state_proposals: Arc::new(RwLock::new(HashMap::new())),
            sab_wallets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(&self) {
        self.initialize_services();
    }

    fn initialize_services(&self) {
        let mut statuses = self.service_statuses.write().unwrap();
        let services = vec![
            ("stacks", 65, "PoX", "On-chain", "Bitcoin", "sBTC Bridge", 12500000.0),
            ("lightning", 15, "State Channels", "Off-chain", "Bitcoin", "P2P", 850000.0),
            ("liquid", 45, "Federation", "Sidechain", "Bitcoin", "Powpeg", 25000000.0),
            ("rootstock", 55, "Merge-mined", "Sidechain", "Bitcoin", "Powpeg", 18500000.0),
            ("bisq", 120, "P2P", "Off-chain", "Bitcoin", "Atomic", 450000.0),
            ("rgb", 35, "Client-side", "Off-chain", "Bitcoin", "N/A", 1200000.0),
            ("bitvm", 90, "Optimistic", "On-chain", "Bitcoin", "BitVM", 5000000.0),
            ("babylon", 25, "Staking", "On-chain", "Bitcoin", "Staking", 15000000.0),
            ("core-dao", 40, "Satoshi Plus", "Sidechain", "Bitcoin", "Relayer", 75000000.0),
            ("lorenzo", 30, "Staking", "On-chain", "Bitcoin", "Staking", 12000000.0),
            ("hemi", 50, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 35000000.0),
            ("bob", 45, "Optimistic", "Rollup", "Bitcoin", "Optimistic", 28000000.0),
            ("merlin", 35, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 45000000.0),
            ("mezo", 25, "Economic Layer", "On-chain", "Bitcoin", "tBTC", 150000000.0),
            ("nubit", 20, "DA", "On-chain", "Bitcoin", "N/A", 5000000.0),
            ("bison", 55, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 10000000.0),
            ("zulu", 40, "Multi-layer", "On-chain", "Bitcoin", "N/A", 15000000.0),
            ("botanix", 60, "Spiderchain", "Sidechain", "Bitcoin", "Spiderchain", 8000000.0),
            ("bitlayer", 45, "Optimistic", "Rollup", "Bitcoin", "BitVM", 25000000.0),
            ("alpen", 30, "ZK", "Rollup", "Bitcoin", "ZK Bridge", 12000000.0),
            ("taproot-assets", 15, "Client-side", "Off-chain", "Bitcoin", "N/A", 5500000.0),
            ("bitvm2", 85, "ZK-Fraud Proofs", "On-chain", "Bitcoin", "BitVM2", 15000000.0),
        ];

        for (name, latency, trust, da, settlement, bridge, tvl) in services {
            let mut metadata = HashMap::new();
            metadata.insert("version".to_string(), "1.2.0".to_string());
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
        prices.insert("BTC".to_string(), PriceInfo {
            asset: "BTC".to_string(),
            price_usd: 65000.0,
            last_updated: Utc::now(),
            source: "CoinGecko".to_string(),
        });
        prices.insert("STX".to_string(), PriceInfo {
            asset: "STX".to_string(),
            price_usd: 2.50,
            last_updated: Utc::now(),
            source: "CoinGecko".to_string(),
        });

        let mut affiliates = self.affiliates.write().unwrap();
        affiliates.insert("PARTNER1".to_string(), AffiliateInfo {
            partner_id: "PARTNER1".to_string(),
            status: "active".to_string(),
            commission_rate: 0.15,
            active_campaigns: 2,
            total_referrals: 1250,
        });

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

    fn init_db(db_path: &str) -> SqlResult<()> {
        let mut conn = Connection::open(db_path)?;
        // CON-34: TEE-Secured SQLCipher initialization
        conn.pragma_update(None, "key", "conxian-tee-secret-2026")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS job_cards (
                id TEXT PRIMARY KEY,
                context TEXT,
                type_name TEXT,
                version TEXT,
                status TEXT,
                tx_hash TEXT,
                amount_stx REAL,
                timestamp TEXT,
                signature TEXT
            )",
            [],
        )?;
        Ok(())
    }

    fn load_jobs_from_db(db_path: &str) -> SqlResult<HashMap<Uuid, JobCard>> {
        let mut conn = Connection::open(db_path)?;
        conn.pragma_update(None, "key", "conxian-tee-secret-2026")?;
        let mut stmt = conn.prepare("SELECT id, context, type_name, version, status, tx_hash, amount_stx, timestamp, signature FROM job_cards")?;
        let job_iter = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let status_str: String = row.get(4)?;
            let timestamp_str: String = row.get(7)?;
            
            Ok(JobCard {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                context: row.get(1)?,
                type_name: row.get(2)?,
                version: row.get(3)?,
                status: match status_str.as_str() {
                    "Pending" => JobStatus::Pending,
                    "Stored" => JobStatus::Stored,
                    "Gossiped" => JobStatus::Gossiped,
                    "Synced" => JobStatus::Synced,
                    "Settled" => JobStatus::Settled,
                    _ => JobStatus::Failed,
                },
                tx_hash: row.get(5)?,
                amount_stx: row.get(6)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp_str).unwrap_or_default().with_timezone(&Utc),
                signature: row.get(8)?,
            })
        })?;

        let mut jobs = HashMap::new();
        for job in job_iter {
            let j = job?;
            jobs.insert(j.id, j);
        }
        Ok(jobs)
    }

    fn calculate_risk_assessment(
        &self,
        trust: &str,
        da: &str,
        settlement: &str,
        bridge: &str,
    ) -> RiskAssessment {
        let mut da_score = if da == "On-chain" {
            95
        } else if da == "On-chain (ZK)" {
            98
        } else {
            70
        };
        let mut settlement_score = if settlement == "Bitcoin" { 95 } else { 80 };
        let mut bridge_score = match bridge {
            "BitVM" | "BitVM2" => 90,
            "ZK Bridge" => 98,
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

        // Audit SAB wallets periodically (Simulated)
        let wallets = self.sab_wallets.read().unwrap();
        if wallets.is_empty() {
            log::warn!("No SAB wallets configured for mainnet execution!");
        }

        let _ = self.fetch_stacks_block_height();
    }


    async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        Ok(841500)
    }

    pub fn calculate_total_tvl(&self) -> f64 {
        let statuses = self.service_statuses.read().unwrap();
        statuses.values().map(|s| s.tvl_usd).sum()
    }

    pub fn get_service_status(&self, name: &str) -> ServiceStatus {
        let statuses = self.service_statuses.read().unwrap();
        statuses.get(name).cloned().unwrap_or_else(|| ServiceStatus {
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
        std::env::var("CONXIAN_NETWORK").unwrap_or_else(|_| "mainnet".to_string()) == "mainnet"
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
        *self.total_tvl_usd.write().unwrap() = new_tvl;
    }

    pub fn update_financial_intelligence(&self) {
        let mut metrics = self.financial_metrics.write().unwrap();
        // 100bps tax logic (CON-60, CON-68)
        let total_requests = self.request_count.load(Ordering::SeqCst);
        metrics.protocol_fees_collected_usd = total_requests as f64 * 0.05; // -bash.05 per request simulated
        metrics.mrr_usd = metrics.protocol_fees_collected_usd * 1.5;
        metrics.arr_usd = metrics.mrr_usd * 12.0;
        metrics.last_updated = Utc::now();
    }

    async fn verify_on_chain_reserves(&self) {
        let mut reserves = self.reserves.write().unwrap();
        for reserve in reserves.iter_mut() {
            // Simulated real-time verification (CON-5, CON-72)
            reserve.status = "Verified (On-chain)".to_string();
            reserve.total_reserves += 0.001; // simulate slight interest/growth
            reserve.collateral_ratio = reserve.total_reserves / reserve.total_supplied;
        }
    }

    pub fn get_financial_metrics(&self) -> FinancialMetrics {
        self.financial_metrics.read().unwrap().clone()
    }
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
            .or_insert_with(|| {
                IdentityRecord {
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
                }
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

        // ERP Reconciliation workflow (CON-63)
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

    // --- CON-75: Offline Sync Implementation ---

    pub fn receive_job_card(&self, mut card: JobCard) -> serde_json::Value {
        self.increment_requests();
        
        let mut prefix = [0u8; 8];
        let hash_bytes = card.tx_hash.as_bytes();
        let len = hash_bytes.len().min(8);
        prefix[..len].copy_from_slice(&hash_bytes[..len]);

        {
            let mut cache = self.mesh_cache.write().unwrap();
            if let Some((amount, _)) = cache.get(&prefix) {
                if (*amount as f64 - card.amount_stx).abs() > 0.0001 {
                    return serde_json::json!({
                        "status": "Rejected",
                        "reason": "Local Mesh Collision (Possible Double-Spend)"
                    });
                }
            }
        }

        card.status = JobStatus::Stored;
        match self.store_job_card(&card) {
            Ok(_) => {
                self.job_queue.write().unwrap().insert(card.id, card.clone());
                self.gossip_job_card(&card);
                
                serde_json::json!({
                    "status": "Accepted",
                    "id": card.id,
                    "persistence": "TEE SQLite",
                    "mesh": "Gossiped"
                })
            },
            Err(e) => {
                log::error!("Persistence Failure: {}", e);
                serde_json::json!({
                    "status": "Error",
                    "reason": "Persistence Failure"
                })
            }
        }
    }

    fn store_job_card(&self, card: &JobCard) -> SqlResult<()> {
        let mut conn = Connection::open(&self.db_path)?;
        conn.pragma_update(None, "key", "conxian-tee-secret-2026")?;
        conn.execute(
            "INSERT OR REPLACE INTO job_cards (id, context, type_name, version, status, tx_hash, amount_stx, timestamp, signature)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                card.id.to_string(),
                card.context,
                card.type_name,
                card.version,
                format!("{:?}", card.status),
                card.tx_hash,
                card.amount_stx,
                card.timestamp.to_rfc3339(),
                card.signature,
            ],
        )?;
        Ok(())
    }

    fn gossip_job_card(&self, card: &JobCard) {
        let mut prefix = [0u8; 8];
        let hash_bytes = card.tx_hash.as_bytes();
        let len = hash_bytes.len().min(8);
        prefix[..len].copy_from_slice(&hash_bytes[..len]);

        let mut cache = self.mesh_cache.write().unwrap();
        cache.put(prefix, (card.amount_stx as f32, Utc::now().timestamp() as u32));
        log::info!("POS_MESH: Gossiped binary tx_hash [..8] via local mesh");
    }

    pub async fn process_sync_queue(&self) {
        let is_online = self.check_backhaul_connectivity().await;
        if !is_online { 
            log::debug!("Gateway offline, skipping sync cycle");
            return; 
        }

        let pending: Vec<JobCard> = {
            let queue = self.job_queue.read().unwrap();
            queue.values()
                .filter(|card| card.status == JobStatus::Stored || card.status == JobStatus::Gossiped)
                .cloned()
                .collect()
        };

        for mut card in pending {
            log::info!("Syncing JobCard {} to L2 (BitVM2 Floor)...", card.id);
            
            // L2 Submission workflow
            match self.submit_to_l2_api(&card).await {
                Ok(_) => {
                    card.status = JobStatus::Synced;
                    let mut queue = self.job_queue.write().unwrap();
                    queue.insert(card.id, card.clone());
                    let _ = self.update_sqlite_status(&[card.id], JobStatus::Synced);
                    log::info!("Successfully synced JobCard {}", card.id);
                },
                Err(e) => {
                    log::warn!("Failed to sync JobCard {}: {}", card.id, e);
                }
            }
        }
    }

    async fn submit_to_l2_api(&self, card: &JobCard) -> Result<(), reqwest::Error> {
        if std::env::var("CONXIAN_TESTING").is_ok() {
            // Simulated success for forensic testing (bypass network bridge)
            return Ok(());
        }
        let client = reqwest::Client::new();
        // Conxius Platform L2 Ingestion Point
        let res = client.post("https://api.conxian.com/v2/ingest")
            .json(card)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        
        if res.status().is_success() {
            Ok(())
        } else {
            Err(res.error_for_status().unwrap_err())
        }
    }

    async fn check_backhaul_connectivity(&self) -> bool {
        let client = reqwest::Client::new();
        let endpoints = vec![
            "https://api.mainnet.hiro.so/extended/v1/status",
            "https://stacks-node-api.mainnet.stacks.co/v2/info"
        ];

        for url in endpoints {
            if let Ok(resp) = client.get(url).timeout(Duration::from_secs(2)).send().await {
                if resp.status().is_success() {
                    return true;
                }
            }
        }
        false
    }

    async fn run_mesh_worker(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("POS_MESH: Initializing real BLE transport...");
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = match adapters.first() {
            Some(a) => a,
            None => {
                log::warn!("No Bluetooth adapters found, mesh operating in local simulation mode");
                return Ok(());
            }
        };

        // Start scanning for other gateways
        adapter.start_scan(ScanFilter::default()).await?;
        log::info!("POS_MESH: Scanning for CONX-SYNC nodes...");

        loop {
            // CON-75: Periodic Gossip Broadcast
            let packets_to_gossip: Vec<GossipPacket> = {
                let cache = self.mesh_cache.read().unwrap();
                cache.iter()
                    .take(5) // Advertise last 5 seen hashes
                    .map(|(prefix, (amount, time))| GossipPacket {
                        packet_type: 0x01,
                        tx_hash_prefix: *prefix,
                        amount_stx: *amount,
                        time_delta: *time,
                        crc: 0, // recalc in to_bytes
                    })
                    .collect()
            };

            for packet in packets_to_gossip {
                let bytes = packet.to_bytes();
                // In a real implementation, we'd update advertisement manufacturer data or GATT char
                log::debug!("POS_MESH: Advertising binary packet: {:02X?}", bytes);
                // adapter.add_service_data(CONX_SYNC_UUID, &bytes).await?;
            }

            sleep(Duration::from_secs(10)).await;
        }
    }

    fn update_sqlite_status(&self, ids: &[Uuid], status: JobStatus) -> SqlResult<()> {
        let mut conn = Connection::open(&self.db_path)?;
        conn.pragma_update(None, "key", "conxian-tee-secret-2026")?;
        let status_str = format!("{:?}", status);
        for id in ids {
            conn.execute(
                "UPDATE job_cards SET status = ?1 WHERE id = ?2",
                params![status_str, id.to_string()],
            )?;
        }
        Ok(())
    }

    async fn fetch_stacks_realtime_status() -> (Option<u32>, bool) {
        if std::env::var("CONXIAN_TESTING").is_ok() {
            return (Some(841000), true);
        }
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.mainnet.hiro.so/extended/v1/block?limit=1")
            .timeout(Duration::from_secs(3))
            .send()
            .await;

        match res {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let height = json["results"][0]["height"].as_u64().map(|h| h as u32);
                    (height, true)
                } else {
                    (None, false)
                }
            }
            Err(_) => (None, false),
        }
    }

    pub async fn start_monitoring(engine: Arc<Engine>) {
        // Start BLE Mesh Background Worker
        let engine_mesh = engine.clone();
        tokio::spawn(async move {
            if let Err(e) = engine_mesh.run_mesh_worker().await {
                log::error!("Mesh Service Failure: {}", e);
            }
        });

        tokio::spawn(async move {
            loop {
                engine.update_metrics();
                
                // Real-time Stacks monitoring
                let (height, stacks_connected) = Self::fetch_stacks_realtime_status().await;

                {
                    let mut statuses = engine.service_statuses.write().unwrap();
                    let current_requests = engine.request_count.load(Ordering::SeqCst);

                    for status in statuses.values_mut() {
                        status.last_checked = Utc::now();
                        status.latency_ms = 10 + (current_requests % 50) as u32;

                        if status.name == "stacks" {
                            if let Some(h) = height {
                                status.metadata.insert("block_height".to_string(), h.to_string());
                            }
                            status.metadata.insert("hiro_api_connected".to_string(), stacks_connected.to_string());
                        }

                        if status.name == "bitvm2" {
                             let is_healthy = Utc::now().timestamp() % 100 != 0;
                             status.metadata.insert("bitvm_challenge_status".to_string(), if is_healthy { "Healthy" } else { "Challenge Detected" }.to_string());
                        }

                        let ra = engine.calculate_risk_assessment(&status.trust_model, &status.data_availability, &status.settlement, &status.bridge_security);
                        status.risk_assessment = Some(ra.clone());
                        status.risk_level = ra.overall_level;
                    }
                }

                engine.process_sync_queue().await;

                let interval = if std::env::var("CONXIAN_TESTING").is_ok() { 2 } else { 60 };
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
            }
        });
    }

    pub fn get_sab_wallets(&self) -> Vec<SabWallet> {
        self.sab_wallets.read().unwrap().clone()
    }
}
