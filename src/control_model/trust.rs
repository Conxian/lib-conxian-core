use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Chain {
    Bitcoin,
    Stacks,
    Liquid,
    Lightning,
    Babylon,
    Bob,
    Mezo,
    Citrea,
    Botanix,
    Ethereum,
    Base,
    Arbitrum,
    Optimism,
    Polygon,
    CosmosHub,
    Osmosis,
    Celestia,
    Solana,
    Eclipse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    BitcoinUtxo,
    Evm,
    CosmosIbc,
    SolanaSvm,
    Move,
    Substrate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Strict,
    Managed,
    Expedient,
    ObserverOnly,
}

impl TrustTier {
    pub fn is_production_allowed(&self) -> bool {
        matches!(
            self,
            TrustTier::Strict | TrustTier::Managed | TrustTier::Expedient
        )
    }

    pub fn requires_light_client(&self) -> bool {
        matches!(self, TrustTier::Strict)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationClass {
    LightClient,
    ExternalQuorum,
    AppDefinedMultiVerifier,
    SharedPos,
    NativeObservation,
    ZkVerified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalityClass {
    Economic,
    Probabilistic,
    Deterministic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Degraded,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSystem {
    Ibc,
    WormholeNtt,
    Hyperlane,
    LayerZeroV2,
    Axelar,
    ChainlinkCcip,
    NearChainSignatures,
    CircleCctp,
    NexusZkVM,
    Bitvm2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEnvelope {
    pub system: BridgeSystem,
    pub system_version: String,
    pub trust_tier: TrustTier,
    pub verification_class: VerificationClass,
    pub source_chain_id: String,
    pub destination_chain_id: String,
    pub finality_class: FinalityClass,
    pub min_confirmations: u32,
    pub observed_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub proof_ref: String,
    pub evidence_hash: String,
    pub evidence_uri: Option<String>,
    pub verifier_set_ref: String,
    pub security_params: serde_json::Value,
    pub verification_status: VerificationStatus,
    pub verification_reason: Option<String>,
}

pub fn validate_trust_tier_policy(
    tier: TrustTier,
    verification: VerificationClass,
) -> Result<(), String> {
    if !tier.is_production_allowed() {
        return Err(format!("Trust tier {:?} is not allowed in production", tier));
    }

    if tier.requires_light_client() && verification != VerificationClass::LightClient {
        return Err(format!(
            "Strict trust tier requires light_client verification, but found {:?}",
            verification
        ));
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RiskAssessment {
    pub overall_level: String,
    pub da_score: u32,
    pub settlement_score: u32,
    pub bridge_score: u32,
    pub exit_mechanism_score: u32,
    pub operators_score: u32,
    pub decentralization_score: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailTrustAssumptions {
    pub security_anchor: String,
    pub operator_dependency: String,
    pub liveness_assumption: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailFinalitySemantics {
    pub confirmation_model: String,
    pub settlement_layer: String,
    pub typical_finality_window: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailCustodyModel {
    pub asset_control_model: String,
    pub signer_architecture: String,
    pub redemption_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailComplianceConstraints {
    pub baseline_controls: Vec<String>,
    pub jurisdictional_scope: String,
    pub monitoring_requirements: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailOperationalCapabilities {
    pub supported_flows: Vec<String>,
    pub integration_modes: Vec<String>,
    pub resilience_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RailMetadata {
    pub rail_family: ChainFamily,
    pub trust_assumptions: RailTrustAssumptions,
    pub finality_semantics: RailFinalitySemantics,
    pub custody_model: RailCustodyModel,
    pub compliance_constraints: RailComplianceConstraints,
    pub operational_capabilities: RailOperationalCapabilities,
}
