//! Versioned, canonical static risk-profile contracts.
//!
//! This module owns protocol metadata only. It deliberately does not acquire live observations,
//! calculate market risk, or select runtime routes. Nexus and Gateway may consume these types as
//! inputs to their own observation and policy workflows, but those workflows remain outside core.

use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;

use chrono::NaiveDate;
use semver::Version;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use super::{
    chain_family_for, validate_trust_tier_policy, Chain, ChainFamily, FinalityClass, RailMetadata,
    TrustTier, VerificationClass,
};

/// The only risk-profile schema currently accepted by this crate.
pub const RISK_PROFILE_SCHEMA_VERSION: u16 = 1;

/// The initial canonical profile-set version.
pub const CANONICAL_RISK_PROFILE_SET_VERSION: &str = "1.0.0";

/// All chain families currently represented by the core taxonomy.
pub const ALL_CHAIN_FAMILIES: &[ChainFamily] = &[
    ChainFamily::BitcoinUtxo,
    ChainFamily::Evm,
    ChainFamily::CosmosIbc,
    ChainFamily::SolanaSvm,
    ChainFamily::Move,
    ChainFamily::Substrate,
];

/// All chain variants currently represented by the core taxonomy.
pub const ALL_CHAINS: &[Chain] = &[
    Chain::Bitcoin,
    Chain::Stacks,
    Chain::Liquid,
    Chain::Lightning,
    Chain::Babylon,
    Chain::Bob,
    Chain::Mezo,
    Chain::Citrea,
    Chain::Botanix,
    Chain::Ethereum,
    Chain::Base,
    Chain::Arbitrum,
    Chain::Optimism,
    Chain::Polygon,
    Chain::CosmosHub,
    Chain::Osmosis,
    Chain::Celestia,
    Chain::Solana,
    Chain::Eclipse,
    Chain::Aptos,
    Chain::Sui,
    Chain::Polkadot,
    Chain::Kusama,
];

/// Errors returned when a canonical risk profile or profile set fails validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskProfileError {
    /// A profile or set could not satisfy a protocol invariant.
    Validation(String),
    /// The checked-in canonical artifact could not be decoded.
    Json(String),
}

impl RiskProfileError {
    fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }
}

impl fmt::Display for RiskProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) => {
                write!(formatter, "risk profile validation failed: {message}")
            }
            Self::Json(message) => {
                write!(formatter, "canonical risk profile JSON failed: {message}")
            }
        }
    }
}

impl std::error::Error for RiskProfileError {}

/// A score in the inclusive range 0..=100.
///
/// Scores are unitless protocol metadata points. They are not percentages, probabilities, or
/// live observations. A score of zero is an assessed minimum; unknown and not-assessed values are
/// represented by [`RiskMetricValue::Unknown`] and [`RiskMetricValue::NotAssessed`] respectively.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct RiskScore(u8);

impl RiskScore {
    /// Lowest valid score.
    pub const MIN: u8 = 0;

    /// Highest valid score.
    pub const MAX: u8 = 100;

    /// Construct a score from an integer that may exceed the valid range.
    pub fn from_u16(value: u16) -> Result<Self, RiskProfileError> {
        if value > u16::from(Self::MAX) {
            return Err(RiskProfileError::validation(format!(
                "risk score {value} is outside the inclusive range 0..=100"
            )));
        }
        Ok(Self(value as u8))
    }

    /// Return the score's numeric value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u16> for RiskScore {
    type Error = RiskProfileError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_u16(value)
    }
}

impl From<RiskScore> for u8 {
    fn from(score: RiskScore) -> Self {
        score.0
    }
}

impl<'de> Deserialize<'de> for RiskScore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u16::deserialize(deserializer)?;
        Self::from_u16(value).map_err(D::Error::custom)
    }
}

/// A governance-defined overall band label.
///
/// Core treats the label as opaque: no approved band vocabulary or threshold mapping is invented
/// here. Governance may define and review labels in a later profile revision.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct RiskBand(String);

impl RiskBand {
    /// Construct a non-empty governance-defined band label.
    pub fn new(value: impl Into<String>) -> Result<Self, RiskProfileError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(RiskProfileError::validation(
                "overall risk band must not be empty",
            ));
        }
        Ok(Self(value))
    }

    /// Return the opaque governance-defined label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RiskBand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

/// Explicit state of one risk dimension.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RiskMetricValue {
    /// The dimension was assessed and has a score, including a valid score of zero.
    Assessed { score: RiskScore },
    /// The dimension is relevant but its value is currently unknown.
    Unknown { reason: String },
    /// No approved assessment has been made for the dimension.
    NotAssessed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiskMetricValueWire {
    state: String,
    #[serde(default)]
    score: Option<Option<RiskScore>>,
    #[serde(default)]
    reason: Option<Option<String>>,
}

impl<'de> Deserialize<'de> for RiskMetricValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RiskMetricValueWire::deserialize(deserializer)?;
        let value = match wire.state.as_str() {
            "assessed" => match (wire.score, wire.reason) {
                (Some(Some(score)), None) => Self::Assessed { score },
                _ => {
                    return Err(D::Error::custom(
                        "assessed risk metrics require a non-null score and no reason",
                    ));
                }
            },
            "unknown" => match (wire.score, wire.reason) {
                (None, Some(Some(reason))) if !reason.trim().is_empty() => Self::Unknown { reason },
                _ => {
                    return Err(D::Error::custom(
                        "unknown risk metrics require a non-empty reason and no score",
                    ));
                }
            },
            "not_assessed" => match (wire.score, wire.reason) {
                (None, None) => Self::NotAssessed,
                _ => {
                    return Err(D::Error::custom(
                        "not_assessed risk metrics must not include score or reason",
                    ));
                }
            },
            state => {
                return Err(D::Error::custom(format!(
                    "unsupported risk metric state {state:?}"
                )));
            }
        };

        Ok(value)
    }
}

/// State derived from a metric value without treating zero as missing.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskMetricState {
    Assessed,
    Unknown,
    NotAssessed,
}

impl RiskMetricValue {
    /// Return the explicit wire state of this metric.
    pub const fn state(&self) -> RiskMetricState {
        match self {
            Self::Assessed { .. } => RiskMetricState::Assessed,
            Self::Unknown { .. } => RiskMetricState::Unknown,
            Self::NotAssessed => RiskMetricState::NotAssessed,
        }
    }

    fn validate(&self, dimension: &'static str) -> Result<(), RiskProfileError> {
        if let Self::Unknown { reason } = self {
            if reason.trim().is_empty() {
                return Err(RiskProfileError::validation(format!(
                    "unknown risk dimension {dimension} must include a non-empty reason"
                )));
            }
        }
        Ok(())
    }
}

/// The six canonical risk dimensions.
///
/// Every dimension uses the same strength polarity: 0 is the lowest assessed strength and 100 is
/// the highest assessed strength for that dimension. In particular, `operator_dependency_score`
/// is a normalized operator-independence/resilience score: 100 means the profile has the strongest
/// assessed independence, not the greatest raw operator dependency. Raw dependency observations
/// belong to Nexus/Gateway and are not encoded by this static score.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct RiskDimensions {
    /// Data availability strength: 0 = weakest assessed availability, 100 = strongest.
    #[serde(alias = "da_score")]
    pub data_availability_score: RiskMetricValue,
    /// Settlement assurance strength: 0 = weakest assessed assurance, 100 = strongest.
    pub settlement_score: RiskMetricValue,
    /// Bridge security strength: 0 = weakest assessed bridge controls, 100 = strongest.
    pub bridge_score: RiskMetricValue,
    /// Exit mechanism strength: 0 = weakest assessed exit guarantees, 100 = strongest.
    pub exit_mechanism_score: RiskMetricValue,
    /// Operator independence/resilience strength; higher means less operator dependency.
    #[serde(alias = "operators_score")]
    pub operator_dependency_score: RiskMetricValue,
    /// Decentralization strength: 0 = weakest assessed decentralization, 100 = strongest.
    pub decentralization_score: RiskMetricValue,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiskDimensionsWire {
    #[serde(alias = "da_score")]
    data_availability_score: RiskMetricValue,
    settlement_score: RiskMetricValue,
    bridge_score: RiskMetricValue,
    exit_mechanism_score: RiskMetricValue,
    #[serde(alias = "operators_score")]
    operator_dependency_score: RiskMetricValue,
    decentralization_score: RiskMetricValue,
}

impl<'de> Deserialize<'de> for RiskDimensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RiskDimensionsWire::deserialize(deserializer)?;
        Ok(Self {
            data_availability_score: wire.data_availability_score,
            settlement_score: wire.settlement_score,
            bridge_score: wire.bridge_score,
            exit_mechanism_score: wire.exit_mechanism_score,
            operator_dependency_score: wire.operator_dependency_score,
            decentralization_score: wire.decentralization_score,
        })
    }
}

impl RiskDimensions {
    fn entries(&self) -> [(&'static str, &RiskMetricValue); 6] {
        [
            ("data_availability_score", &self.data_availability_score),
            ("settlement_score", &self.settlement_score),
            ("bridge_score", &self.bridge_score),
            ("exit_mechanism_score", &self.exit_mechanism_score),
            ("operator_dependency_score", &self.operator_dependency_score),
            ("decentralization_score", &self.decentralization_score),
        ]
    }
}

/// Explicit aggregate assessment state.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskAssessmentStatus {
    /// All six dimensions and the overall band are assessed.
    Assessed,
    /// At least one dimension is assessed and at least one is unknown or not assessed.
    PartiallyAssessed,
    /// All six dimensions are explicitly unknown.
    Unknown,
    /// No dimension has an approved assessment.
    NotAssessed,
}

/// Explicit overall status and, when approved, an opaque governance-defined band.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum OverallRiskStatus {
    /// An approved band label accompanies the overall assessment.
    Assessed { band: RiskBand },
    /// The overall band is currently unknown.
    Unknown { reason: String },
    /// No approved overall assessment has been made.
    NotAssessed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverallRiskStatusWire {
    state: String,
    #[serde(default)]
    band: Option<Option<RiskBand>>,
    #[serde(default)]
    reason: Option<Option<String>>,
}

impl<'de> Deserialize<'de> for OverallRiskStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = OverallRiskStatusWire::deserialize(deserializer)?;
        let value = match wire.state.as_str() {
            "assessed" => match (wire.band, wire.reason) {
                (Some(Some(band)), None) => Self::Assessed { band },
                _ => {
                    return Err(D::Error::custom(
                        "assessed overall status requires a non-null band and no reason",
                    ));
                }
            },
            "unknown" => match (wire.band, wire.reason) {
                (None, Some(Some(reason))) if !reason.trim().is_empty() => Self::Unknown { reason },
                _ => {
                    return Err(D::Error::custom(
                        "unknown overall status requires a non-empty reason and no band",
                    ));
                }
            },
            "not_assessed" => match (wire.band, wire.reason) {
                (None, None) => Self::NotAssessed,
                _ => {
                    return Err(D::Error::custom(
                        "not_assessed overall status must not include band or reason",
                    ));
                }
            },
            state => {
                return Err(D::Error::custom(format!(
                    "unsupported overall risk status {state:?}"
                )));
            }
        };

        Ok(value)
    }
}

impl OverallRiskStatus {
    fn validate(&self) -> Result<(), RiskProfileError> {
        match self {
            Self::Assessed { band } => {
                if band.as_str().trim().is_empty() {
                    return Err(RiskProfileError::validation(
                        "overall assessed band must not be empty",
                    ));
                }
            }
            Self::Unknown { reason } if reason.trim().is_empty() => {
                return Err(RiskProfileError::validation(
                    "unknown overall risk status must include a non-empty reason",
                ));
            }
            Self::Unknown { .. } | Self::NotAssessed => {}
        }
        Ok(())
    }
}

/// Risk dimensions plus an explicit aggregate state.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct RiskProfileAssessment {
    pub status: RiskAssessmentStatus,
    pub dimensions: RiskDimensions,
    pub overall: OverallRiskStatus,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiskProfileAssessmentWire {
    status: RiskAssessmentStatus,
    dimensions: RiskDimensions,
    overall: OverallRiskStatus,
}

impl<'de> Deserialize<'de> for RiskProfileAssessment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RiskProfileAssessmentWire::deserialize(deserializer)?;
        let assessment = Self {
            status: wire.status,
            dimensions: wire.dimensions,
            overall: wire.overall,
        };
        assessment.validate_structure().map_err(D::Error::custom)?;
        Ok(assessment)
    }
}

impl RiskProfileAssessment {
    /// Validate state consistency and the evidence requirement for assessed profiles.
    pub fn validate(&self, evidence_count: usize) -> Result<(), RiskProfileError> {
        self.validate_structure()?;

        match self.status {
            RiskAssessmentStatus::Assessed => require_evidence(evidence_count, "assessed")?,
            RiskAssessmentStatus::PartiallyAssessed => {
                require_evidence(evidence_count, "partially_assessed")?
            }
            RiskAssessmentStatus::Unknown | RiskAssessmentStatus::NotAssessed => {}
        }

        Ok(())
    }

    fn validate_structure(&self) -> Result<(), RiskProfileError> {
        for (dimension, value) in self.dimensions.entries() {
            value.validate(dimension)?;
        }
        self.overall.validate()?;

        let states = self.dimensions.entries().map(|(_, value)| value.state());
        let assessed_count = states
            .into_iter()
            .filter(|state| *state == RiskMetricState::Assessed)
            .count();
        let unknown_count = states
            .into_iter()
            .filter(|state| *state == RiskMetricState::Unknown)
            .count();
        let not_assessed_count = states
            .into_iter()
            .filter(|state| *state == RiskMetricState::NotAssessed)
            .count();

        match self.status {
            RiskAssessmentStatus::Assessed => {
                if assessed_count != 6
                    || !matches!(self.overall, OverallRiskStatus::Assessed { .. })
                {
                    return Err(RiskProfileError::validation(
                        "assessed profiles require six assessed dimensions and an assessed overall band",
                    ));
                }
            }
            RiskAssessmentStatus::PartiallyAssessed => {
                if assessed_count == 0
                    || assessed_count == 6
                    || !matches!(self.overall, OverallRiskStatus::Unknown { .. })
                {
                    return Err(RiskProfileError::validation(
                        "partially_assessed profiles require a mix of assessed and unknown/not_assessed dimensions with an unknown overall status",
                    ));
                }
            }
            RiskAssessmentStatus::Unknown => {
                if unknown_count != 6 || !matches!(self.overall, OverallRiskStatus::Unknown { .. })
                {
                    return Err(RiskProfileError::validation(
                        "unknown profiles require all six dimensions and the overall status to be unknown",
                    ));
                }
            }
            RiskAssessmentStatus::NotAssessed => {
                if not_assessed_count != 6
                    || !matches!(self.overall, OverallRiskStatus::NotAssessed)
                {
                    return Err(RiskProfileError::validation(
                        "not_assessed profiles require all six dimensions and the overall status to be not_assessed",
                    ));
                }
            }
        }

        Ok(())
    }
}

fn require_evidence(evidence_count: usize, status: &str) -> Result<(), RiskProfileError> {
    if evidence_count == 0 {
        return Err(RiskProfileError::validation(format!(
            "{status} profiles require at least one evidence reference"
        )));
    }
    Ok(())
}

/// A target identity for a family-wide or chain-specific profile.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RiskTarget {
    Family { family: ChainFamily },
    Chain { chain: Chain, family: ChainFamily },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiskTargetWire {
    kind: String,
    #[serde(default)]
    family: Option<Option<ChainFamily>>,
    #[serde(default)]
    chain: Option<Option<Chain>>,
}

impl<'de> Deserialize<'de> for RiskTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RiskTargetWire::deserialize(deserializer)?;
        let target = match wire.kind.as_str() {
            "family" => match (wire.family, wire.chain) {
                (Some(Some(family)), None) => Self::Family { family },
                _ => {
                    return Err(D::Error::custom(
                        "family risk targets require a non-null family and no chain",
                    ));
                }
            },
            "chain" => match (wire.chain, wire.family) {
                (Some(Some(chain)), Some(Some(family))) => Self::Chain { chain, family },
                _ => {
                    return Err(D::Error::custom(
                        "chain risk targets require non-null chain and family values",
                    ));
                }
            },
            kind => {
                return Err(D::Error::custom(format!(
                    "unsupported risk target kind {kind:?}"
                )));
            }
        };

        target.validate().map_err(D::Error::custom)?;
        Ok(target)
    }
}

impl RiskTarget {
    /// Return the family covered by this target.
    pub fn family(&self) -> ChainFamily {
        match self {
            Self::Family { family } | Self::Chain { family, .. } => family.clone(),
        }
    }

    /// Validate that a chain target carries the canonical family mapping.
    pub fn validate(&self) -> Result<(), RiskProfileError> {
        if let Self::Chain { chain, family } = self {
            let expected = chain_family_for(chain);
            if *family != expected {
                return Err(RiskProfileError::validation(format!(
                    "chain target {:?} carries family {:?}, expected {:?}",
                    chain, family, expected
                )));
            }
        }
        Ok(())
    }

    fn key(&self) -> String {
        match self {
            Self::Family { family } => format!("family:{}", family_name(family)),
            Self::Chain { chain, .. } => format!("chain:{}", chain_name(chain)),
        }
    }
}

/// The closed provenance class for empirical evidence supporting a profile.
///
/// Governance/change references intentionally are not an evidence kind. The profile's separate
/// `governance_ref` field carries those references; they cannot satisfy the empirical evidence
/// requirement for assessed or partially assessed profiles.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Specification,
    Audit,
    Research,
    Observation,
}

/// A typed reference to evidence supporting an assessed or partially assessed profile.
#[derive(Serialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EvidenceReference {
    pub kind: EvidenceKind,
    pub reference: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceReferenceWire {
    kind: EvidenceKind,
    reference: String,
}

impl<'de> Deserialize<'de> for EvidenceReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EvidenceReferenceWire::deserialize(deserializer)?;
        let evidence = Self {
            kind: wire.kind,
            reference: wire.reference,
        };
        evidence.validate_reference().map_err(D::Error::custom)?;
        Ok(evidence)
    }
}

impl EvidenceReference {
    fn validate_reference(&self) -> Result<(), RiskProfileError> {
        require_non_empty("evidence reference", &self.reference)
    }

    fn validate(&self, index: usize) -> Result<(), RiskProfileError> {
        if self.reference.trim().is_empty() {
            return Err(RiskProfileError::validation(format!(
                "evidence reference at index {index} must not be empty"
            )));
        }
        Ok(())
    }
}

/// Optional static policy assumptions associated with a profile.
///
/// These are protocol metadata assumptions, not live [`super::VerificationStatus`] values. No
/// universal finality combination rule is inferred here; only the existing trust-tier policy is
/// enforced.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct StaticPolicyAssumptions {
    pub trust_tier: TrustTier,
    pub verification_class: VerificationClass,
    pub finality_class: FinalityClass,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StaticPolicyAssumptionsWire {
    trust_tier: TrustTier,
    verification_class: VerificationClass,
    finality_class: FinalityClass,
}

impl<'de> Deserialize<'de> for StaticPolicyAssumptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = StaticPolicyAssumptionsWire::deserialize(deserializer)?;
        let policy = Self {
            trust_tier: wire.trust_tier,
            verification_class: wire.verification_class,
            finality_class: wire.finality_class,
        };
        policy.validate().map_err(D::Error::custom)?;
        Ok(policy)
    }
}

impl StaticPolicyAssumptions {
    /// Validate the existing trust-tier/verification invariant.
    pub fn validate(&self) -> Result<(), RiskProfileError> {
        validate_trust_tier_policy(self.trust_tier.clone(), self.verification_class.clone())
            .map_err(RiskProfileError::validation)
    }
}

/// Versioned compatibility wrapper for the legacy [`RailMetadata`] shape.
///
/// The legacy fields remain unchanged inside `metadata`; this wrapper makes use in a canonical
/// profile explicit and validates the rail family against the profile target.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct VersionedRailMetadata {
    pub schema_version: u16,
    pub metadata: RailMetadata,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedRailMetadataWire {
    schema_version: u16,
    metadata: RailMetadataWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailMetadataWire {
    rail_family: ChainFamily,
    trust_assumptions: RailTrustAssumptionsWire,
    finality_semantics: RailFinalitySemanticsWire,
    custody_model: RailCustodyModelWire,
    compliance_constraints: RailComplianceConstraintsWire,
    operational_capabilities: RailOperationalCapabilitiesWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailTrustAssumptionsWire {
    security_anchor: String,
    operator_dependency: String,
    liveness_assumption: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailFinalitySemanticsWire {
    confirmation_model: String,
    settlement_layer: String,
    typical_finality_window: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailCustodyModelWire {
    asset_control_model: String,
    signer_architecture: String,
    redemption_path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailComplianceConstraintsWire {
    baseline_controls: Vec<String>,
    jurisdictional_scope: String,
    monitoring_requirements: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RailOperationalCapabilitiesWire {
    supported_flows: Vec<String>,
    integration_modes: Vec<String>,
    resilience_features: Vec<String>,
}

impl<'de> Deserialize<'de> for VersionedRailMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = VersionedRailMetadataWire::deserialize(deserializer)?;
        let metadata = Self {
            schema_version: wire.schema_version,
            metadata: RailMetadata {
                rail_family: wire.metadata.rail_family,
                trust_assumptions: super::RailTrustAssumptions {
                    security_anchor: wire.metadata.trust_assumptions.security_anchor,
                    operator_dependency: wire.metadata.trust_assumptions.operator_dependency,
                    liveness_assumption: wire.metadata.trust_assumptions.liveness_assumption,
                },
                finality_semantics: super::RailFinalitySemantics {
                    confirmation_model: wire.metadata.finality_semantics.confirmation_model,
                    settlement_layer: wire.metadata.finality_semantics.settlement_layer,
                    typical_finality_window: wire
                        .metadata
                        .finality_semantics
                        .typical_finality_window,
                },
                custody_model: super::RailCustodyModel {
                    asset_control_model: wire.metadata.custody_model.asset_control_model,
                    signer_architecture: wire.metadata.custody_model.signer_architecture,
                    redemption_path: wire.metadata.custody_model.redemption_path,
                },
                compliance_constraints: super::RailComplianceConstraints {
                    baseline_controls: wire.metadata.compliance_constraints.baseline_controls,
                    jurisdictional_scope: wire.metadata.compliance_constraints.jurisdictional_scope,
                    monitoring_requirements: wire
                        .metadata
                        .compliance_constraints
                        .monitoring_requirements,
                },
                operational_capabilities: super::RailOperationalCapabilities {
                    supported_flows: wire.metadata.operational_capabilities.supported_flows,
                    integration_modes: wire.metadata.operational_capabilities.integration_modes,
                    resilience_features: wire.metadata.operational_capabilities.resilience_features,
                },
            },
        };
        metadata
            .validate_schema_version()
            .map_err(D::Error::custom)?;
        Ok(metadata)
    }
}

impl VersionedRailMetadata {
    fn validate_schema_version(&self) -> Result<(), RiskProfileError> {
        if self.schema_version != RISK_PROFILE_SCHEMA_VERSION {
            return Err(RiskProfileError::validation(format!(
                "unsupported rail metadata schema version {}; expected {RISK_PROFILE_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        Ok(())
    }

    /// Validate the wrapper version and target-family reconciliation.
    pub fn validate(&self, expected_family: &ChainFamily) -> Result<(), RiskProfileError> {
        self.validate_schema_version()?;
        if &self.metadata.rail_family != expected_family {
            return Err(RiskProfileError::validation(format!(
                "rail metadata family {:?} does not match target family {:?}",
                self.metadata.rail_family, expected_family
            )));
        }
        Ok(())
    }
}

/// One schema-v1 canonical risk profile.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct RiskProfile {
    pub schema_version: u16,
    pub profile_set_version: String,
    pub target: RiskTarget,
    pub profile_revision: u32,
    pub effective_date: NaiveDate,
    pub governance_ref: String,
    pub evidence_refs: Vec<EvidenceReference>,
    pub rationale: String,
    pub assessment: RiskProfileAssessment,
    pub static_policy: Option<StaticPolicyAssumptions>,
    pub rail_metadata: Option<VersionedRailMetadata>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiskProfileWire {
    schema_version: u16,
    profile_set_version: String,
    target: RiskTarget,
    profile_revision: u32,
    effective_date: NaiveDate,
    governance_ref: String,
    evidence_refs: Vec<EvidenceReference>,
    rationale: String,
    assessment: RiskProfileAssessment,
    static_policy: Option<StaticPolicyAssumptions>,
    rail_metadata: Option<VersionedRailMetadata>,
}

impl<'de> Deserialize<'de> for RiskProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RiskProfileWire::deserialize(deserializer)?;
        let profile = Self {
            schema_version: wire.schema_version,
            profile_set_version: wire.profile_set_version,
            target: wire.target,
            profile_revision: wire.profile_revision,
            effective_date: wire.effective_date,
            governance_ref: wire.governance_ref,
            evidence_refs: wire.evidence_refs,
            rationale: wire.rationale,
            assessment: wire.assessment,
            static_policy: wire.static_policy,
            rail_metadata: wire.rail_metadata,
        };
        profile.validate().map_err(D::Error::custom)?;
        Ok(profile)
    }
}

impl RiskProfile {
    /// Validate schema, provenance, target identity, assessment consistency, and optional policy.
    pub fn validate(&self) -> Result<(), RiskProfileError> {
        if self.schema_version != RISK_PROFILE_SCHEMA_VERSION {
            return Err(RiskProfileError::validation(format!(
                "unsupported risk profile schema version {}; expected {RISK_PROFILE_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        validate_profile_set_version(&self.profile_set_version)?;
        if self.profile_revision == 0 {
            return Err(RiskProfileError::validation(
                "profile_revision must be greater than zero",
            ));
        }
        self.target.validate()?;
        require_non_empty("governance_ref", &self.governance_ref)?;
        require_non_empty("rationale", &self.rationale)?;
        let mut evidence_seen = HashSet::with_capacity(self.evidence_refs.len());
        for (index, evidence) in self.evidence_refs.iter().enumerate() {
            evidence.validate(index)?;
            if !evidence_seen.insert(evidence.clone()) {
                return Err(RiskProfileError::validation(format!(
                    "duplicate evidence reference at index {index}"
                )));
            }
        }
        self.assessment.validate(self.evidence_refs.len())?;
        if let Some(static_policy) = &self.static_policy {
            static_policy.validate()?;
        }
        if let Some(rail_metadata) = &self.rail_metadata {
            rail_metadata.validate(&self.target.family())?;
        }
        Ok(())
    }
}

/// The checked-in, versioned set of canonical risk profiles.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRiskProfileSet {
    pub schema_version: u16,
    pub profile_set_version: String,
    pub profiles: Vec<RiskProfile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalRiskProfileSetWire {
    schema_version: u16,
    profile_set_version: String,
    profiles: Vec<RiskProfile>,
}

impl<'de> Deserialize<'de> for CanonicalRiskProfileSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CanonicalRiskProfileSetWire::deserialize(deserializer)?;
        let set = Self {
            schema_version: wire.schema_version,
            profile_set_version: wire.profile_set_version,
            profiles: wire.profiles,
        };
        set.validate().map_err(D::Error::custom)?;
        Ok(set)
    }
}

impl CanonicalRiskProfileSet {
    /// Validate every profile and prove exact family/chain coverage without duplicates.
    pub fn validate(&self) -> Result<(), RiskProfileError> {
        if self.schema_version != RISK_PROFILE_SCHEMA_VERSION {
            return Err(RiskProfileError::validation(format!(
                "unsupported canonical profile-set schema version {}; expected {RISK_PROFILE_SCHEMA_VERSION}",
                self.schema_version
            )));
        }
        validate_profile_set_version(&self.profile_set_version)?;

        let expected: HashSet<String> = expected_target_keys();
        let mut seen = HashSet::with_capacity(self.profiles.len());

        for profile in &self.profiles {
            profile.validate()?;
            if profile.profile_set_version != self.profile_set_version {
                return Err(RiskProfileError::validation(format!(
                    "profile {} uses profile_set_version {}, expected {}",
                    profile.target.key(),
                    profile.profile_set_version,
                    self.profile_set_version
                )));
            }
            let key = profile.target.key();
            if !seen.insert(key.clone()) {
                return Err(RiskProfileError::validation(format!(
                    "duplicate canonical risk-profile target {key}"
                )));
            }
            if !expected.contains(&key) {
                return Err(RiskProfileError::validation(format!(
                    "unexpected canonical risk-profile target {key}"
                )));
            }
        }

        if let Some(key) = expected.difference(&seen).next() {
            return Err(RiskProfileError::validation(format!(
                "missing canonical risk-profile target {key}"
            )));
        }

        Ok(())
    }

    /// Find a profile by its exact target identity.
    pub fn profile_for_target(&self, target: &RiskTarget) -> Option<&RiskProfile> {
        if target.validate().is_err() {
            return None;
        }
        self.profiles
            .iter()
            .find(|profile| &profile.target == target)
    }
}

fn require_non_empty(field: &str, value: &str) -> Result<(), RiskProfileError> {
    if value.trim().is_empty() {
        return Err(RiskProfileError::validation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn validate_profile_set_version(version: &str) -> Result<(), RiskProfileError> {
    let parsed = Version::parse(version).map_err(|error| {
        RiskProfileError::validation(format!(
            "profile_set_version {version:?} must be valid SemVer: {error}"
        ))
    })?;
    if parsed.major != 1 {
        return Err(RiskProfileError::validation(format!(
            "unsupported profile_set_version major {}; schema version {RISK_PROFILE_SCHEMA_VERSION} accepts only major 1",
            parsed.major
        )));
    }
    Ok(())
}

fn expected_target_keys() -> HashSet<String> {
    let mut expected = HashSet::with_capacity(ALL_CHAIN_FAMILIES.len() + ALL_CHAINS.len());
    expected.extend(
        ALL_CHAIN_FAMILIES
            .iter()
            .map(|family| format!("family:{}", family_name(family))),
    );
    expected.extend(
        ALL_CHAINS
            .iter()
            .map(|chain| format!("chain:{}", chain_name(chain))),
    );
    expected
}

fn family_name(family: &ChainFamily) -> &'static str {
    match family {
        ChainFamily::BitcoinUtxo => "bitcoin_utxo",
        ChainFamily::Evm => "evm",
        ChainFamily::CosmosIbc => "cosmos_ibc",
        ChainFamily::SolanaSvm => "solana_svm",
        ChainFamily::Move => "move",
        ChainFamily::Substrate => "substrate",
    }
}

fn chain_name(chain: &Chain) -> &'static str {
    match chain {
        Chain::Bitcoin => "bitcoin",
        Chain::Stacks => "stacks",
        Chain::Liquid => "liquid",
        Chain::Lightning => "lightning",
        Chain::Babylon => "babylon",
        Chain::Bob => "bob",
        Chain::Mezo => "mezo",
        Chain::Citrea => "citrea",
        Chain::Botanix => "botanix",
        Chain::Ethereum => "ethereum",
        Chain::Base => "base",
        Chain::Arbitrum => "arbitrum",
        Chain::Optimism => "optimism",
        Chain::Polygon => "polygon",
        Chain::CosmosHub => "cosmos_hub",
        Chain::Osmosis => "osmosis",
        Chain::Celestia => "celestia",
        Chain::Solana => "solana",
        Chain::Eclipse => "eclipse",
        Chain::Aptos => "aptos",
        Chain::Sui => "sui",
        Chain::Polkadot => "polkadot",
        Chain::Kusama => "kusama",
    }
}

const CANONICAL_PROFILE_SET_JSON: &str = include_str!("../../data/risk_profiles/v1.json");

static CANONICAL_PROFILE_SET: OnceLock<Result<CanonicalRiskProfileSet, RiskProfileError>> =
    OnceLock::new();

/// Return the checked-in canonical profile-set JSON source.
pub fn canonical_risk_profile_set_json() -> &'static str {
    CANONICAL_PROFILE_SET_JSON
}

/// Parse and validate the checked-in canonical profile set once, then return the cached result.
pub fn canonical_risk_profile_set() -> Result<&'static CanonicalRiskProfileSet, RiskProfileError> {
    match CANONICAL_PROFILE_SET.get_or_init(|| {
        let set: CanonicalRiskProfileSet = serde_json::from_str(CANONICAL_PROFILE_SET_JSON)
            .map_err(|error| RiskProfileError::Json(error.to_string()))?;
        set.validate()?;
        if set.profile_set_version != CANONICAL_RISK_PROFILE_SET_VERSION {
            return Err(RiskProfileError::validation(format!(
                "embedded canonical profile set version {} does not match {CANONICAL_RISK_PROFILE_SET_VERSION}",
                set.profile_set_version
            )));
        }
        Ok(set)
    }) {
        Ok(set) => Ok(set),
        Err(error) => Err(error.clone()),
    }
}

/// Alias with an explicit loader name for downstream consumers.
pub fn load_canonical_risk_profile_set(
) -> Result<&'static CanonicalRiskProfileSet, RiskProfileError> {
    canonical_risk_profile_set()
}
