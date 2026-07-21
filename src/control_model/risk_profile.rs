//! Versioned, static risk-profile metadata for enumerated chain families and chains.
//!
//! This module deliberately does not contain live observations, market data, or
//! routing policy. Core owns the wire model and fail-closed invariants; Nexus
//! owns proof/finality/freshness observations; Gateway owns routing and policy
//! decisions. A profile with an unknown or not-assessed status must not be
//! treated as an approved risk score by downstream consumers.

use super::{
    validate_trust_tier_policy, Chain, ChainFamily, FinalityClass, TrustTier, VerificationClass,
};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The only schema version currently accepted by this crate.
pub const CANONICAL_RISK_PROFILE_SCHEMA_VERSION: RiskProfileSchemaVersion =
    RiskProfileSchemaVersion { major: 1, minor: 0 };

/// Stable public issue reference that records the governance gap represented by
/// the initial all-not-assessed registry.
pub const CANONICAL_RISK_PROFILE_GOVERNANCE_REFERENCE: &str = "github:Conxian/lib-conxian-core#177";

/// Stable policy identifier for the first canonical profile schema.
pub const CANONICAL_RISK_PROFILE_POLICY_REFERENCE: &str =
    "core-007-canonical-risk-profile-schema-v1";

/// The explicitly versioned wire-schema identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskProfileSchemaVersion {
    pub major: u16,
    pub minor: u16,
}

impl RiskProfileSchemaVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub const fn current() -> Self {
        CANONICAL_RISK_PROFILE_SCHEMA_VERSION
    }

    pub const fn is_supported(&self) -> bool {
        self.major == CANONICAL_RISK_PROFILE_SCHEMA_VERSION.major
            && self.minor == CANONICAL_RISK_PROFILE_SCHEMA_VERSION.minor
    }
}

/// Whether a profile contains an approved static assessment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    /// A complete static assessment is present and backed by public evidence.
    Assessed,
    /// Governance has explicitly left the subject without an assessment.
    NotAssessed,
    /// The subject or available metadata cannot currently be classified.
    Unknown,
}

/// The subject identity for a profile. A missing `chain` is a family baseline;
/// a present `chain` is a chain-specific override.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskProfileSubject {
    pub family: ChainFamily,
    pub chain: Option<Chain>,
}

impl RiskProfileSubject {
    pub fn family_baseline(family: ChainFamily) -> Self {
        Self {
            family,
            chain: None,
        }
    }

    pub fn chain(chain: Chain) -> Self {
        Self {
            family: chain.family(),
            chain: Some(chain),
        }
    }

    pub fn is_family_baseline(&self) -> bool {
        self.chain.is_none()
    }

    pub fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if let Some(chain) = &self.chain {
            let expected_family = chain.family();
            if expected_family != self.family {
                return Err(RiskProfileValidationError::SubjectFamilyMismatch {
                    chain: chain.clone(),
                    expected_family,
                    provided_family: self.family.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Risk dimensions retained from the existing `RiskAssessment` vocabulary.
/// The enum is a schema vocabulary only; it does not assign scores.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskDimension {
    DataAvailability,
    Settlement,
    Bridge,
    ExitMechanism,
    Operators,
    Decentralization,
}

impl RiskDimension {
    pub const fn all() -> &'static [Self] {
        &[
            Self::DataAvailability,
            Self::Settlement,
            Self::Bridge,
            Self::ExitMechanism,
            Self::Operators,
            Self::Decentralization,
        ]
    }

    fn order(&self) -> usize {
        match self {
            Self::DataAvailability => 0,
            Self::Settlement => 1,
            Self::Bridge => 2,
            Self::ExitMechanism => 3,
            Self::Operators => 4,
            Self::Decentralization => 5,
        }
    }
}

/// The transport unit for a future approved score. `NormalizedPoints` always
/// uses the exact inclusive `0..=100` scale. It is not a probability, market
/// score, or routing recommendation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskScoreUnit {
    NormalizedPoints,
}

/// Bounds for every score in a profile. `NormalizedPoints` is intentionally
/// fixed to the neutral `0..=100` transport scale without assigning any
/// values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskScoreScale {
    pub unit: RiskScoreUnit,
    pub lower_bound: u16,
    pub upper_bound: u16,
}

impl RiskScoreScale {
    pub const fn normalized_points() -> Self {
        Self {
            unit: RiskScoreUnit::NormalizedPoints,
            lower_bound: 0,
            upper_bound: 100,
        }
    }

    fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if self.lower_bound > self.upper_bound {
            return Err(RiskProfileValidationError::InvalidScoreBounds {
                lower_bound: self.lower_bound,
                upper_bound: self.upper_bound,
            });
        }

        match &self.unit {
            RiskScoreUnit::NormalizedPoints if self.lower_bound != 0 || self.upper_bound != 100 => {
                return Err(RiskProfileValidationError::InvalidNormalizedPointsBounds {
                    lower_bound: self.lower_bound,
                    upper_bound: self.upper_bound,
                });
            }
            RiskScoreUnit::NormalizedPoints => {}
        }

        Ok(())
    }
}

/// A score for one dimension. Values are optional at the profile level by
/// leaving the vector empty for `unknown` and `not_assessed` profiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskScore {
    pub dimension: RiskDimension,
    pub value: u16,
}

impl RiskScore {
    pub const fn new(dimension: RiskDimension, value: u16) -> Self {
        Self { dimension, value }
    }
}

/// Static posture associated with an assessed profile. This reuses the
/// existing CORE-006 trust/verification/finality taxonomies instead of
/// defining a second policy matrix.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskProfilePosture {
    pub trust_tier: TrustTier,
    pub verification_class: VerificationClass,
    pub finality_class: FinalityClass,
}

impl RiskProfilePosture {
    fn validate(&self) -> Result<(), RiskProfileValidationError> {
        validate_trust_tier_policy(self.trust_tier.clone(), self.verification_class.clone())
            .map_err(RiskProfileValidationError::InvalidPosture)
    }
}

/// Public evidence reference. References must point to public, reviewable
/// material; credentials, tokens, private endpoints, and raw sensitive data do
/// not belong in this type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskEvidenceKind {
    PublicSpecification,
    IndependentReview,
    GovernanceRecord,
    PublicObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskEvidence {
    pub kind: RiskEvidenceKind,
    pub reference: String,
    pub digest: Option<String>,
}

impl RiskEvidence {
    fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if self.reference.trim().is_empty() {
            return Err(RiskProfileValidationError::EmptyReference {
                field: "evidence.reference",
            });
        }
        if self
            .digest
            .as_ref()
            .is_some_and(|digest| digest.trim().is_empty())
        {
            return Err(RiskProfileValidationError::EmptyReference {
                field: "evidence.digest",
            });
        }
        Ok(())
    }
}

/// Public governance references for a profile decision and its governing
/// schema/policy. These are references, not embedded approval credentials.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceReference {
    pub decision_ref: String,
    pub policy_ref: String,
}

impl GovernanceReference {
    fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if self.decision_ref.trim().is_empty() {
            return Err(RiskProfileValidationError::EmptyReference {
                field: "governance.decision_ref",
            });
        }
        if self.policy_ref.trim().is_empty() {
            return Err(RiskProfileValidationError::EmptyReference {
                field: "governance.policy_ref",
            });
        }
        Ok(())
    }
}

/// Reference to the prior profile revision replaced by this profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskProfileSupersession {
    pub revision: u64,
    pub effective_from: DateTime<Utc>,
}

/// A versioned static profile. `profile_revision` and `schema_version` are
/// intentionally separate: the former changes when approved data changes,
/// while the latter changes only when the wire contract changes.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CanonicalRiskProfile {
    pub schema_version: RiskProfileSchemaVersion,
    pub profile_revision: u64,
    pub subject: RiskProfileSubject,
    pub status: AssessmentStatus,
    pub effective_from: DateTime<Utc>,
    pub supersedes: Option<RiskProfileSupersession>,
    pub score_scale: RiskScoreScale,
    pub scores: Vec<RiskScore>,
    pub posture: Option<RiskProfilePosture>,
    pub evidence: Vec<RiskEvidence>,
    pub governance: GovernanceReference,
}

impl CanonicalRiskProfile {
    pub fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if !self.schema_version.is_supported() {
            return Err(RiskProfileValidationError::UnsupportedSchemaVersion {
                version: self.schema_version.clone(),
            });
        }
        if self.profile_revision == 0 {
            return Err(RiskProfileValidationError::InvalidProfileRevision);
        }
        self.subject.validate()?;
        self.score_scale.validate()?;
        self.governance.validate()?;

        if let Some(supersedes) = &self.supersedes {
            if supersedes.revision == 0 || supersedes.revision >= self.profile_revision {
                return Err(RiskProfileValidationError::InvalidSupersessionRevision {
                    profile_revision: self.profile_revision,
                    superseded_revision: supersedes.revision,
                });
            }
            if supersedes.effective_from >= self.effective_from {
                return Err(RiskProfileValidationError::InvalidSupersessionOrdering {
                    superseded_effective_from: supersedes.effective_from,
                    effective_from: self.effective_from,
                });
            }
        }

        for evidence in &self.evidence {
            evidence.validate()?;
        }

        match self.status {
            AssessmentStatus::Assessed => {
                if self.scores.len() != RiskDimension::all().len() {
                    return Err(
                        RiskProfileValidationError::AssessedProfileRequiresAllScores {
                            expected: RiskDimension::all().len(),
                            actual: self.scores.len(),
                        },
                    );
                }
                if self.evidence.is_empty() {
                    return Err(RiskProfileValidationError::AssessedProfileRequiresEvidence);
                }
                let posture = self
                    .posture
                    .as_ref()
                    .ok_or(RiskProfileValidationError::AssessedProfileRequiresPosture)?;
                posture.validate()?;
                self.validate_scores()?;
            }
            AssessmentStatus::NotAssessed | AssessmentStatus::Unknown => {
                if !self.scores.is_empty() {
                    return Err(RiskProfileValidationError::UnassessedProfileHasScores {
                        status: self.status.clone(),
                    });
                }
                if self.posture.is_some() {
                    return Err(RiskProfileValidationError::UnassessedProfileHasPosture {
                        status: self.status.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_scores(&self) -> Result<(), RiskProfileValidationError> {
        let mut previous_order = None;
        for score in &self.scores {
            let order = score.dimension.order();
            if let Some(previous_order) = previous_order {
                if order <= previous_order {
                    return Err(RiskProfileValidationError::ScoresNotDeterministicallyOrdered);
                }
            }
            previous_order = Some(order);
            if score.value < self.score_scale.lower_bound
                || score.value > self.score_scale.upper_bound
            {
                return Err(RiskProfileValidationError::ScoreOutOfBounds {
                    dimension: score.dimension.clone(),
                    value: score.value,
                    lower_bound: self.score_scale.lower_bound,
                    upper_bound: self.score_scale.upper_bound,
                });
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for CanonicalRiskProfile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawCanonicalRiskProfile {
            schema_version: RiskProfileSchemaVersion,
            profile_revision: u64,
            subject: RiskProfileSubject,
            status: AssessmentStatus,
            effective_from: DateTime<Utc>,
            supersedes: Option<RiskProfileSupersession>,
            score_scale: RiskScoreScale,
            scores: Vec<RiskScore>,
            posture: Option<RiskProfilePosture>,
            evidence: Vec<RiskEvidence>,
            governance: GovernanceReference,
        }

        let raw = RawCanonicalRiskProfile::deserialize(deserializer)?;
        let profile = Self {
            schema_version: raw.schema_version,
            profile_revision: raw.profile_revision,
            subject: raw.subject,
            status: raw.status,
            effective_from: raw.effective_from,
            supersedes: raw.supersedes,
            score_scale: raw.score_scale,
            scores: raw.scores,
            posture: raw.posture,
            evidence: raw.evidence,
            governance: raw.governance,
        };
        profile.validate().map_err(serde::de::Error::custom)?;
        Ok(profile)
    }
}

/// A deterministic registry of one current profile per family baseline or
/// enumerated chain-specific subject.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RiskProfileRegistry {
    pub schema_version: RiskProfileSchemaVersion,
    pub profiles: Vec<CanonicalRiskProfile>,
}

impl<'de> Deserialize<'de> for RiskProfileRegistry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawRiskProfileRegistry {
            schema_version: RiskProfileSchemaVersion,
            profiles: Vec<CanonicalRiskProfile>,
        }

        let raw = RawRiskProfileRegistry::deserialize(deserializer)?;
        let registry = Self {
            schema_version: raw.schema_version,
            profiles: raw.profiles,
        };
        registry.validate().map_err(serde::de::Error::custom)?;
        Ok(registry)
    }
}

impl RiskProfileRegistry {
    pub fn new(
        schema_version: RiskProfileSchemaVersion,
        profiles: Vec<CanonicalRiskProfile>,
    ) -> Self {
        Self {
            schema_version,
            profiles,
        }
    }

    /// Returns the explicit all-not-assessed registry for every currently
    /// enumerated family and chain. This is a governance-gap marker, not a set
    /// of approved scores or routing recommendations.
    pub fn canonical() -> Self {
        let mut profiles =
            Vec::with_capacity(enumerated_chain_families().len() + enumerated_chains().len());
        for family in enumerated_chain_families() {
            profiles.push(not_assessed_profile(RiskProfileSubject::family_baseline(
                family.clone(),
            )));
        }
        for chain in enumerated_chains() {
            profiles.push(not_assessed_profile(RiskProfileSubject::chain(
                chain.clone(),
            )));
        }
        Self::new(RiskProfileSchemaVersion::current(), profiles)
    }

    pub fn validate(&self) -> Result<(), RiskProfileValidationError> {
        if !self.schema_version.is_supported() {
            return Err(RiskProfileValidationError::UnsupportedSchemaVersion {
                version: self.schema_version.clone(),
            });
        }

        let mut previous_key = None;
        for profile in &self.profiles {
            if profile.schema_version != self.schema_version {
                return Err(RiskProfileValidationError::RegistrySchemaMismatch {
                    registry: self.schema_version.clone(),
                    profile: profile.schema_version.clone(),
                });
            }
            profile.validate()?;

            let key = subject_order_key(&profile.subject);
            if let Some(previous_key) = previous_key {
                if key <= previous_key {
                    return Err(RiskProfileValidationError::RegistryNotDeterministicallyOrdered);
                }
            }
            previous_key = Some(key);
        }

        for family in enumerated_chain_families() {
            let subject = RiskProfileSubject::family_baseline(family.clone());
            if self.lookup(&subject).is_none() {
                return Err(RiskProfileValidationError::MissingSubject { subject });
            }
        }
        for chain in enumerated_chains() {
            let subject = RiskProfileSubject::chain(chain.clone());
            if self.lookup(&subject).is_none() {
                return Err(RiskProfileValidationError::MissingSubject { subject });
            }
        }

        Ok(())
    }

    pub fn lookup(&self, subject: &RiskProfileSubject) -> Option<&CanonicalRiskProfile> {
        self.profiles
            .iter()
            .find(|profile| &profile.subject == subject)
    }

    pub fn family_baseline(&self, family: &ChainFamily) -> Option<&CanonicalRiskProfile> {
        self.lookup(&RiskProfileSubject::family_baseline(family.clone()))
    }

    pub fn chain_profile(&self, chain: &Chain) -> Option<&CanonicalRiskProfile> {
        self.lookup(&RiskProfileSubject::chain(chain.clone()))
    }

    /// Resolves a chain-specific override when present, otherwise its family
    /// baseline. An explicit `unknown` or `not_assessed` chain entry wins over
    /// the family baseline so downstream callers cannot silently inherit stale
    /// or unapproved data.
    pub fn resolve(
        &self,
        chain: &Chain,
    ) -> Result<ResolvedRiskProfile<'_>, RiskProfileLookupError> {
        self.validate()
            .map_err(|source| RiskProfileLookupError::InvalidRegistry { source })?;

        let family = chain.family();
        let family_baseline = self.family_baseline(&family).ok_or_else(|| {
            RiskProfileLookupError::MissingFamilyBaseline {
                family: family.clone(),
            }
        })?;
        let chain_override = self.chain_profile(chain);
        let effective = chain_override.unwrap_or(family_baseline);
        Ok(ResolvedRiskProfile {
            family_baseline,
            chain_override,
            effective,
        })
    }
}

/// Result of resolving a chain against a family baseline and optional override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRiskProfile<'a> {
    pub family_baseline: &'a CanonicalRiskProfile,
    pub chain_override: Option<&'a CanonicalRiskProfile>,
    pub effective: &'a CanonicalRiskProfile,
}

impl ResolvedRiskProfile<'_> {
    pub fn uses_chain_override(&self) -> bool {
        self.chain_override.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskProfileLookupError {
    InvalidRegistry { source: RiskProfileValidationError },
    MissingFamilyBaseline { family: ChainFamily },
}

impl fmt::Display for RiskProfileLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegistry { source } => {
                write!(f, "invalid risk-profile registry: {source}")
            }
            Self::MissingFamilyBaseline { family } => {
                write!(f, "missing risk-profile family baseline for {family:?}")
            }
        }
    }
}

impl std::error::Error for RiskProfileLookupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidRegistry { source } => Some(source),
            Self::MissingFamilyBaseline { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskProfileValidationError {
    UnsupportedSchemaVersion {
        version: RiskProfileSchemaVersion,
    },
    RegistrySchemaMismatch {
        registry: RiskProfileSchemaVersion,
        profile: RiskProfileSchemaVersion,
    },
    SubjectFamilyMismatch {
        chain: Chain,
        expected_family: ChainFamily,
        provided_family: ChainFamily,
    },
    InvalidProfileRevision,
    InvalidSupersessionRevision {
        profile_revision: u64,
        superseded_revision: u64,
    },
    InvalidSupersessionOrdering {
        superseded_effective_from: DateTime<Utc>,
        effective_from: DateTime<Utc>,
    },
    InvalidScoreBounds {
        lower_bound: u16,
        upper_bound: u16,
    },
    InvalidNormalizedPointsBounds {
        lower_bound: u16,
        upper_bound: u16,
    },
    EmptyReference {
        field: &'static str,
    },
    AssessedProfileRequiresAllScores {
        expected: usize,
        actual: usize,
    },
    AssessedProfileRequiresEvidence,
    AssessedProfileRequiresPosture,
    UnassessedProfileHasScores {
        status: AssessmentStatus,
    },
    UnassessedProfileHasPosture {
        status: AssessmentStatus,
    },
    ScoresNotDeterministicallyOrdered,
    ScoreOutOfBounds {
        dimension: RiskDimension,
        value: u16,
        lower_bound: u16,
        upper_bound: u16,
    },
    InvalidPosture(String),
    RegistryNotDeterministicallyOrdered,
    MissingSubject {
        subject: RiskProfileSubject,
    },
}

impl fmt::Display for RiskProfileValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { version } => write!(
                f,
                "unsupported canonical risk-profile schema version {}.{}",
                version.major, version.minor
            ),
            Self::RegistrySchemaMismatch { registry, profile } => write!(
                f,
                "risk-profile registry schema {}.{} does not match profile schema {}.{}",
                registry.major, registry.minor, profile.major, profile.minor
            ),
            Self::SubjectFamilyMismatch {
                chain,
                expected_family,
                provided_family,
            } => write!(
                f,
                "risk-profile chain {chain:?} belongs to {expected_family:?}, not {provided_family:?}"
            ),
            Self::InvalidProfileRevision => write!(f, "profile_revision must be greater than zero"),
            Self::InvalidSupersessionRevision {
                profile_revision,
                superseded_revision,
            } => write!(
                f,
                "superseded revision {superseded_revision} must be non-zero and less than profile revision {profile_revision}"
            ),
            Self::InvalidSupersessionOrdering {
                superseded_effective_from,
                effective_from,
            } => write!(
                f,
                "superseded profile effective date {superseded_effective_from} must precede {effective_from}"
            ),
            Self::InvalidScoreBounds {
                lower_bound,
                upper_bound,
            } => write!(
                f,
                "risk-score lower bound {lower_bound} exceeds upper bound {upper_bound}"
            ),
            Self::InvalidNormalizedPointsBounds {
                lower_bound,
                upper_bound,
            } => write!(
                f,
                "normalized_points requires exact 0..=100 bounds, found {lower_bound}..={upper_bound}"
            ),
            Self::EmptyReference { field } => write!(f, "{field} must not be empty"),
            Self::AssessedProfileRequiresAllScores { expected, actual } => write!(
                f,
                "assessed profile requires {expected} dimension scores, found {actual}"
            ),
            Self::AssessedProfileRequiresEvidence => {
                write!(f, "assessed profile requires public evidence")
            }
            Self::AssessedProfileRequiresPosture => {
                write!(f, "assessed profile requires static trust posture")
            }
            Self::UnassessedProfileHasScores { status } => {
                write!(f, "{status:?} profile must not contain scores")
            }
            Self::UnassessedProfileHasPosture { status } => {
                write!(f, "{status:?} profile must not contain static posture")
            }
            Self::ScoresNotDeterministicallyOrdered => {
                write!(f, "risk scores must be unique and ordered by dimension")
            }
            Self::ScoreOutOfBounds {
                dimension,
                value,
                lower_bound,
                upper_bound,
            } => write!(
                f,
                "score {value} for {dimension:?} is outside {lower_bound}..={upper_bound}"
            ),
            Self::InvalidPosture(reason) => write!(f, "invalid risk-profile posture: {reason}"),
            Self::RegistryNotDeterministicallyOrdered => {
                write!(f, "risk-profile registry entries must be unique and deterministically ordered")
            }
            Self::MissingSubject { subject } => {
                write!(f, "risk-profile registry is missing subject {subject:?}")
            }
        }
    }
}

impl std::error::Error for RiskProfileValidationError {}

/// All currently enumerated family values in their stable wire/registry order.
pub fn enumerated_chain_families() -> &'static [ChainFamily] {
    static FAMILIES: [ChainFamily; 6] = [
        ChainFamily::BitcoinUtxo,
        ChainFamily::Evm,
        ChainFamily::CosmosIbc,
        ChainFamily::SolanaSvm,
        ChainFamily::Move,
        ChainFamily::Substrate,
    ];
    &FAMILIES
}

/// All currently enumerated chain values in their stable wire/registry order.
pub fn enumerated_chains() -> &'static [Chain] {
    static CHAINS: [Chain; 23] = [
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
        // Keep this list tied to the exhaustive `Chain` enum. If a new variant
        // is added, update this list and the registry coverage tests together.
    ];
    &CHAINS
}

fn subject_order_key(subject: &RiskProfileSubject) -> (u8, usize) {
    if let Some(chain) = &subject.chain {
        (1, chain_order(chain))
    } else {
        (0, family_order(&subject.family))
    }
}

fn family_order(family: &ChainFamily) -> usize {
    enumerated_chain_families()
        .iter()
        .position(|candidate| candidate == family)
        .expect("all ChainFamily variants must be listed in the canonical registry")
}

fn chain_order(chain: &Chain) -> usize {
    enumerated_chains()
        .iter()
        .position(|candidate| candidate == chain)
        .expect("all Chain variants must be listed in the canonical registry")
}

fn canonical_effective_from() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 21, 0, 0, 0)
        .single()
        .expect("canonical risk-profile effective date must be valid")
}

fn not_assessed_profile(subject: RiskProfileSubject) -> CanonicalRiskProfile {
    CanonicalRiskProfile {
        schema_version: RiskProfileSchemaVersion::current(),
        profile_revision: 1,
        subject,
        status: AssessmentStatus::NotAssessed,
        effective_from: canonical_effective_from(),
        supersedes: None,
        score_scale: RiskScoreScale::normalized_points(),
        scores: Vec::new(),
        posture: None,
        evidence: Vec::new(),
        governance: GovernanceReference {
            decision_ref: CANONICAL_RISK_PROFILE_GOVERNANCE_REFERENCE.to_string(),
            policy_ref: CANONICAL_RISK_PROFILE_POLICY_REFERENCE.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_registry_covers_all_enumerated_subjects() {
        let registry = RiskProfileRegistry::canonical();

        assert_eq!(
            registry.profiles.len(),
            enumerated_chain_families().len() + enumerated_chains().len()
        );
        assert!(registry.validate().is_ok());
        assert!(registry
            .profiles
            .iter()
            .all(|profile| profile.status == AssessmentStatus::NotAssessed));
    }

    #[test]
    fn subject_validation_rejects_known_family_mismatch() {
        let subject = RiskProfileSubject {
            family: ChainFamily::Evm,
            chain: Some(Chain::Bitcoin),
        };

        assert!(matches!(
            subject.validate(),
            Err(RiskProfileValidationError::SubjectFamilyMismatch { .. })
        ));
    }

    #[test]
    fn strict_posture_reuses_existing_light_client_invariant() {
        let posture = RiskProfilePosture {
            trust_tier: TrustTier::Strict,
            verification_class: VerificationClass::ExternalQuorum,
            finality_class: FinalityClass::Probabilistic,
        };

        assert!(matches!(
            posture.validate(),
            Err(RiskProfileValidationError::InvalidPosture(_))
        ));
    }
}
