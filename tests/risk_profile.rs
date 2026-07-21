use chrono::{TimeZone, Utc};
use lib_conxian_core::control_model::{
    enumerated_chain_families, enumerated_chains, AssessmentStatus, CanonicalRiskProfile, Chain,
    ChainFamily, FinalityClass, GovernanceReference, RailComplianceConstraints, RailCustodyModel,
    RailFinalitySemantics, RailMetadata, RailOperationalCapabilities, RailTrustAssumptions,
    RiskAssessment, RiskDimension, RiskEvidence, RiskEvidenceKind, RiskProfilePosture,
    RiskProfileRegistry, RiskProfileSchemaVersion, RiskProfileSubject, RiskProfileSupersession,
    RiskProfileValidationError, RiskScore, RiskScoreScale, RiskScoreUnit, TrustTier,
    VerificationClass, CANONICAL_RISK_PROFILE_SCHEMA_VERSION,
};
use serde_json::json;

fn timestamp(day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, day, 0, 0, 0)
        .single()
        .expect("test timestamp must be valid")
}

fn governance() -> GovernanceReference {
    GovernanceReference {
        decision_ref: "github:Conxian/lib-conxian-core#177".to_string(),
        policy_ref: "core-007-canonical-risk-profile-schema-v1".to_string(),
    }
}

fn evidence() -> Vec<RiskEvidence> {
    vec![RiskEvidence {
        kind: RiskEvidenceKind::PublicSpecification,
        reference: "https://example.invalid/public-specification".to_string(),
        digest: Some("sha256:public-reference".to_string()),
    }]
}

fn assessed_profile(subject: RiskProfileSubject) -> CanonicalRiskProfile {
    let scores = RiskDimension::all()
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, dimension)| RiskScore::new(dimension, index as u16))
        .collect();

    CanonicalRiskProfile {
        schema_version: CANONICAL_RISK_PROFILE_SCHEMA_VERSION,
        profile_revision: 2,
        subject,
        status: AssessmentStatus::Assessed,
        effective_from: timestamp(2),
        supersedes: Some(RiskProfileSupersession {
            revision: 1,
            effective_from: timestamp(1),
        }),
        score_scale: RiskScoreScale::normalized_points(),
        scores,
        posture: Some(RiskProfilePosture {
            trust_tier: TrustTier::Strict,
            verification_class: VerificationClass::LightClient,
            finality_class: FinalityClass::Probabilistic,
        }),
        evidence: evidence(),
        governance: governance(),
    }
}

#[test]
fn wire_values_are_explicit_and_stable_snake_case() {
    let registry = RiskProfileRegistry::canonical();
    let profile = registry
        .lookup(&RiskProfileSubject::chain(Chain::Bitcoin))
        .expect("canonical registry must contain Bitcoin");
    let wire = serde_json::to_value(profile).expect("profile must serialize");

    assert_eq!(wire["schema_version"], json!({ "major": 1, "minor": 0 }));
    assert_eq!(wire["status"], "not_assessed");
    assert_eq!(wire["subject"]["family"], "bitcoin_utxo");
    assert_eq!(wire["subject"]["chain"], "bitcoin");
    assert_eq!(wire["score_scale"]["unit"], "normalized_points");
    assert_eq!(wire["effective_from"], "2026-07-21T00:00:00Z");
}

#[test]
fn unknown_and_not_assessed_profiles_are_explicitly_scoreless() {
    let mut profile = RiskProfileRegistry::canonical()
        .lookup(&RiskProfileSubject::chain(Chain::Stacks))
        .expect("canonical registry must contain Stacks")
        .clone();
    assert_eq!(profile.status, AssessmentStatus::NotAssessed);
    assert!(profile.validate().is_ok());

    profile.status = AssessmentStatus::Unknown;
    assert!(profile.validate().is_ok());

    profile
        .scores
        .push(RiskScore::new(RiskDimension::Settlement, 1));
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::UnassessedProfileHasScores {
            status: AssessmentStatus::Unknown
        })
    ));
}

#[test]
fn assessed_profiles_require_evidence_posture_and_all_dimensions() {
    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    assert!(profile.validate().is_ok());

    profile.evidence.clear();
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::AssessedProfileRequiresEvidence)
    ));

    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile.posture = None;
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::AssessedProfileRequiresPosture)
    ));

    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile.scores.pop();
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::AssessedProfileRequiresAllScores { .. })
    ));
}

#[test]
fn score_lower_and_upper_bounds_are_checked() {
    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile.scores[0].value = 0;
    profile.scores[1].value = 100;
    assert!(profile.validate().is_ok());

    profile.scores[1].value = 101;
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::ScoreOutOfBounds {
            dimension: RiskDimension::Settlement,
            value: 101,
            lower_bound: 0,
            upper_bound: 100,
        })
    ));

    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile.score_scale = RiskScoreScale {
        unit: RiskScoreUnit::NormalizedPoints,
        lower_bound: 80,
        upper_bound: 20,
    };
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::InvalidScoreBounds {
            lower_bound: 80,
            upper_bound: 20,
        })
    ));
}

#[test]
fn subject_family_mapping_rejects_invalid_known_pairs() {
    let profile = assessed_profile(RiskProfileSubject {
        family: ChainFamily::Evm,
        chain: Some(Chain::Bitcoin),
    });

    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::SubjectFamilyMismatch {
            chain: Chain::Bitcoin,
            expected_family: ChainFamily::BitcoinUtxo,
            provided_family: ChainFamily::Evm,
        })
    ));
}

#[test]
fn registry_is_complete_deterministic_and_has_required_explicit_entries() {
    let registry = RiskProfileRegistry::canonical();
    assert!(registry.validate().is_ok());
    assert_eq!(registry.profiles.len(), 6 + 23);
    assert_eq!(enumerated_chain_families().len(), 6);
    assert_eq!(enumerated_chains().len(), 23);

    for chain in [Chain::Bitcoin, Chain::Stacks, Chain::Babylon, Chain::Liquid] {
        let profile = registry
            .chain_profile(&chain)
            .expect("required chain entry must be explicit");
        assert_eq!(profile.status, AssessmentStatus::NotAssessed);
    }

    let first = serde_json::to_string(&registry).expect("registry must serialize");
    let second = serde_json::to_string(&RiskProfileRegistry::canonical())
        .expect("registry must serialize deterministically");
    assert_eq!(first, second);
}

#[test]
fn resolution_prefers_chain_override_and_falls_back_to_family_baseline_only_when_absent() {
    let family_profile = assessed_profile(RiskProfileSubject::family_baseline(
        ChainFamily::BitcoinUtxo,
    ));
    let family_only = RiskProfileRegistry::new(
        CANONICAL_RISK_PROFILE_SCHEMA_VERSION,
        vec![family_profile.clone()],
    );
    let resolved = family_only
        .resolve(&Chain::Bitcoin)
        .expect("family baseline should resolve");
    assert!(!resolved.uses_chain_override());
    assert_eq!(resolved.effective.subject, family_profile.subject);

    let chain_profile = RiskProfileRegistry::canonical()
        .chain_profile(&Chain::Bitcoin)
        .expect("canonical chain profile must exist")
        .clone();
    let with_override = RiskProfileRegistry::new(
        CANONICAL_RISK_PROFILE_SCHEMA_VERSION,
        vec![family_profile, chain_profile.clone()],
    );
    let resolved = with_override
        .resolve(&Chain::Bitcoin)
        .expect("chain override should resolve");
    assert!(resolved.uses_chain_override());
    assert_eq!(resolved.effective.subject, chain_profile.subject);
    assert_eq!(resolved.effective.status, AssessmentStatus::NotAssessed);
}

#[test]
fn unsupported_schema_version_fails_validation_and_wire_deserialization() {
    let registry = RiskProfileRegistry::canonical();
    let profile = registry
        .chain_profile(&Chain::Bitcoin)
        .expect("canonical profile must exist");
    let mut wire = serde_json::to_value(profile).expect("profile must serialize");
    wire["schema_version"] = json!({ "major": 9, "minor": 0 });

    let error = serde_json::from_value::<CanonicalRiskProfile>(wire)
        .expect_err("unsupported schema version must fail closed");
    assert!(error
        .to_string()
        .contains("unsupported canonical risk-profile schema"));

    let mut invalid = profile.clone();
    invalid.schema_version = RiskProfileSchemaVersion::new(9, 0);
    assert!(matches!(
        invalid.validate(),
        Err(RiskProfileValidationError::UnsupportedSchemaVersion { .. })
    ));

    let mut registry_wire =
        serde_json::to_value(RiskProfileRegistry::canonical()).expect("registry must serialize");
    registry_wire["schema_version"] = json!({ "major": 9, "minor": 0 });
    assert!(serde_json::from_value::<RiskProfileRegistry>(registry_wire).is_err());
}

#[test]
fn effective_date_and_supersession_ordering_are_checked() {
    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile
        .supersedes
        .as_mut()
        .expect("supersession exists")
        .effective_from = timestamp(2);
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::InvalidSupersessionOrdering { .. })
    ));

    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile
        .supersedes
        .as_mut()
        .expect("supersession exists")
        .revision = 2;
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::InvalidSupersessionRevision { .. })
    ));
}

#[test]
fn strict_posture_requires_light_client_without_a_second_policy_matrix() {
    let mut profile = assessed_profile(RiskProfileSubject::chain(Chain::Bitcoin));
    profile
        .posture
        .as_mut()
        .expect("assessed profile has posture")
        .verification_class = VerificationClass::ExternalQuorum;
    assert!(matches!(
        profile.validate(),
        Err(RiskProfileValidationError::InvalidPosture(_))
    ));
}

#[test]
fn legacy_risk_and_rail_models_keep_their_existing_wire_shape() {
    let assessment = RiskAssessment {
        overall_level: "unknown".to_string(),
        da_score: 0,
        settlement_score: 0,
        bridge_score: 0,
        exit_mechanism_score: 0,
        operators_score: 0,
        decentralization_score: 0,
    };
    let assessment_wire = serde_json::to_value(assessment).expect("legacy assessment serializes");
    assert_eq!(assessment_wire["da_score"], 0);
    assert_eq!(assessment_wire["settlement_score"], 0);

    let rail = RailMetadata {
        rail_family: ChainFamily::BitcoinUtxo,
        trust_assumptions: RailTrustAssumptions {
            security_anchor: "public-spec".to_string(),
            operator_dependency: "unknown".to_string(),
            liveness_assumption: "unknown".to_string(),
        },
        finality_semantics: RailFinalitySemantics {
            confirmation_model: "unknown".to_string(),
            settlement_layer: "unknown".to_string(),
            typical_finality_window: "unknown".to_string(),
        },
        custody_model: RailCustodyModel {
            asset_control_model: "unknown".to_string(),
            signer_architecture: "unknown".to_string(),
            redemption_path: "unknown".to_string(),
        },
        compliance_constraints: RailComplianceConstraints {
            baseline_controls: Vec::new(),
            jurisdictional_scope: "unknown".to_string(),
            monitoring_requirements: Vec::new(),
        },
        operational_capabilities: RailOperationalCapabilities {
            supported_flows: Vec::new(),
            integration_modes: Vec::new(),
            resilience_features: Vec::new(),
        },
    };
    let rail_wire = serde_json::to_value(rail).expect("legacy rail metadata serializes");
    assert_eq!(rail_wire["rail_family"], "bitcoin_utxo");
    assert!(rail_wire.get("risk_profile").is_none());
}
