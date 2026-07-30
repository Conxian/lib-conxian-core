use chrono::NaiveDate;
use lib_conxian_core::control_model::{
    canonical_risk_profile_set, canonical_risk_profile_set_json, chain_family_for,
    validate_trust_tier_policy, CanonicalRiskProfileSet, Chain, ChainFamily, EvidenceKind,
    EvidenceReference, FinalityClass, OverallRiskStatus, RailComplianceConstraints,
    RailCustodyModel, RailFinalitySemantics, RailMetadata, RailOperationalCapabilities,
    RailTrustAssumptions, RiskAssessment, RiskAssessmentStatus, RiskBand, RiskDimensions,
    RiskMetricState, RiskMetricValue, RiskProfile, RiskProfileAssessment, RiskScore, RiskTarget,
    StaticPolicyAssumptions, TrustTier, VerificationClass, VersionedRailMetadata, ALL_CHAINS,
    ALL_CHAIN_FAMILIES, CANONICAL_RISK_PROFILE_SET_VERSION, RISK_PROFILE_SCHEMA_VERSION,
};
use serde_json::{json, Value};

fn not_assessed_dimensions() -> RiskDimensions {
    RiskDimensions {
        data_availability_score: RiskMetricValue::NotAssessed,
        settlement_score: RiskMetricValue::NotAssessed,
        bridge_score: RiskMetricValue::NotAssessed,
        exit_mechanism_score: RiskMetricValue::NotAssessed,
        operator_dependency_score: RiskMetricValue::NotAssessed,
        decentralization_score: RiskMetricValue::NotAssessed,
    }
}

fn assessed_dimensions(score: RiskScore) -> RiskDimensions {
    RiskDimensions {
        data_availability_score: RiskMetricValue::Assessed { score },
        settlement_score: RiskMetricValue::Assessed { score },
        bridge_score: RiskMetricValue::Assessed { score },
        exit_mechanism_score: RiskMetricValue::Assessed { score },
        operator_dependency_score: RiskMetricValue::Assessed { score },
        decentralization_score: RiskMetricValue::Assessed { score },
    }
}

fn not_assessed_assessment() -> RiskProfileAssessment {
    RiskProfileAssessment {
        status: RiskAssessmentStatus::NotAssessed,
        dimensions: not_assessed_dimensions(),
        overall: OverallRiskStatus::NotAssessed,
    }
}

fn assessed_profile() -> RiskProfile {
    RiskProfile {
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        profile_set_version: "1.0.0".to_string(),
        target: RiskTarget::Family {
            family: ChainFamily::BitcoinUtxo,
        },
        profile_revision: 1,
        effective_date: NaiveDate::from_ymd_opt(2026, 7, 21).unwrap(),
        governance_ref: "github:Conxian/lib-conxian-core#177".to_string(),
        evidence_refs: vec![EvidenceReference {
            kind: EvidenceKind::Research,
            reference: "research:approved-evidence-1".to_string(),
        }],
        rationale: "Approved evidence supports this test profile.".to_string(),
        assessment: RiskProfileAssessment {
            status: RiskAssessmentStatus::Assessed,
            dimensions: assessed_dimensions(RiskScore::from_u16(0).unwrap()),
            overall: OverallRiskStatus::Assessed {
                band: RiskBand::new("governance_defined_band").unwrap(),
            },
        },
        static_policy: None,
        rail_metadata: None,
    }
}

fn unknown_dimensions(reason: &str) -> RiskDimensions {
    let metric = || RiskMetricValue::Unknown {
        reason: reason.to_string(),
    };
    RiskDimensions {
        data_availability_score: metric(),
        settlement_score: metric(),
        bridge_score: metric(),
        exit_mechanism_score: metric(),
        operator_dependency_score: metric(),
        decentralization_score: metric(),
    }
}

fn unknown_profile() -> RiskProfile {
    let mut profile = assessed_profile();
    profile.evidence_refs.clear();
    profile.assessment = RiskProfileAssessment {
        status: RiskAssessmentStatus::Unknown,
        dimensions: unknown_dimensions("the current evidence set is inconclusive"),
        overall: OverallRiskStatus::Unknown {
            reason: "the aggregate band is not observable".to_string(),
        },
    };
    profile
}

fn rail_metadata(family: ChainFamily) -> RailMetadata {
    RailMetadata {
        rail_family: family,
        trust_assumptions: RailTrustAssumptions {
            security_anchor: "test-anchor".to_string(),
            operator_dependency: "test-assumption".to_string(),
            liveness_assumption: "test-liveness".to_string(),
        },
        finality_semantics: RailFinalitySemantics {
            confirmation_model: "test-confirmations".to_string(),
            settlement_layer: "test-settlement".to_string(),
            typical_finality_window: "test-window".to_string(),
        },
        custody_model: RailCustodyModel {
            asset_control_model: "test-control".to_string(),
            signer_architecture: "test-signers".to_string(),
            redemption_path: "test-redemption".to_string(),
        },
        compliance_constraints: RailComplianceConstraints {
            baseline_controls: vec!["test-control".to_string()],
            jurisdictional_scope: "test-scope".to_string(),
            monitoring_requirements: vec!["test-monitoring".to_string()],
        },
        operational_capabilities: RailOperationalCapabilities {
            supported_flows: vec!["test-flow".to_string()],
            integration_modes: vec!["test-mode".to_string()],
            resilience_features: vec!["test-resilience".to_string()],
        },
    }
}

#[test]
fn canonical_artifact_is_a_valid_json_golden_with_exact_coverage() {
    let source: Value = serde_json::from_str(canonical_risk_profile_set_json()).unwrap();
    let set: CanonicalRiskProfileSet = serde_json::from_value(source.clone()).unwrap();
    set.validate().unwrap();

    let roundtrip: Value = serde_json::to_value(&set).unwrap();
    assert_eq!(roundtrip, source);
    assert_eq!(set.schema_version, RISK_PROFILE_SCHEMA_VERSION);
    assert_eq!(set.profile_set_version, CANONICAL_RISK_PROFILE_SET_VERSION);
    assert_eq!(set.profiles.len(), 17 + 48);
    assert_eq!(
        set.profiles
            .iter()
            .filter(|profile| matches!(profile.target, RiskTarget::Family { .. }))
            .count(),
        17
    );
    assert_eq!(
        set.profiles
            .iter()
            .filter(|profile| matches!(profile.target, RiskTarget::Chain { .. }))
            .count(),
        48
    );
    assert!(set.profiles.iter().all(|profile| {
        profile.assessment.status == RiskAssessmentStatus::NotAssessed
            && profile.evidence_refs.is_empty()
    }));
}

#[test]
fn canonical_loader_validates_and_exposes_each_target() {
    let set = canonical_risk_profile_set().unwrap();

    for family in ALL_CHAIN_FAMILIES {
        let target = RiskTarget::Family {
            family: family.clone(),
        };
        assert!(set.profile_for_target(&target).is_some());
    }

    for chain in ALL_CHAINS {
        let target = RiskTarget::Chain {
            family: chain_family_for(chain),
            chain: chain.clone(),
        };
        assert!(set.profile_for_target(&target).is_some());
    }
}

#[test]
fn score_bounds_accept_zero_and_one_hundred_but_reject_one_hundred_one() {
    assert_eq!(RiskScore::from_u16(0).unwrap().value(), 0);
    assert_eq!(RiskScore::from_u16(100).unwrap().value(), 100);
    assert!(RiskScore::from_u16(101).is_err());

    let zero: RiskScore = serde_json::from_str("0").unwrap();
    let hundred: RiskScore = serde_json::from_str("100").unwrap();
    assert_eq!(zero.value(), 0);
    assert_eq!(hundred.value(), 100);
    assert!(serde_json::from_str::<RiskScore>("101").is_err());
    assert!(serde_json::from_str::<RiskScore>("-1").is_err());
}

#[test]
fn missing_metric_is_rejected_instead_of_defaulted() {
    let mut value = serde_json::to_value(assessed_profile()).unwrap();
    value["assessment"]["dimensions"]
        .as_object_mut()
        .unwrap()
        .remove("bridge_score");

    let error = serde_json::from_value::<RiskProfile>(value).unwrap_err();
    assert!(error.to_string().contains("bridge_score"));
}

#[test]
fn direct_assessment_decode_rejects_inconsistent_status_and_metrics() {
    let mut value = serde_json::to_value(assessed_profile().assessment).unwrap();
    value["status"] = json!("not_assessed");

    let error = serde_json::from_value::<RiskProfileAssessment>(value).unwrap_err();
    assert!(error.to_string().contains("not_assessed profiles"));
}

#[test]
fn direct_target_decode_rejects_chain_family_mismatch() {
    let value = json!({
        "kind": "chain",
        "chain": "bitcoin",
        "family": "evm"
    });

    assert!(serde_json::from_value::<RiskTarget>(value).is_err());
}

#[test]
fn direct_metric_decode_rejects_blank_unknown_reason_and_unknown_fields() {
    let blank_reason = json!({
        "state": "unknown",
        "reason": "   "
    });
    assert!(serde_json::from_value::<RiskMetricValue>(blank_reason).is_err());

    let typo = json!({
        "state": "not_assessed",
        "reasno": "typo"
    });
    assert!(serde_json::from_value::<RiskMetricValue>(typo).is_err());
}

#[test]
fn zero_assessed_is_distinct_from_unknown_and_not_assessed() {
    let zero = RiskMetricValue::Assessed {
        score: RiskScore::from_u16(0).unwrap(),
    };
    let unknown = RiskMetricValue::Unknown {
        reason: "evidence is inconclusive".to_string(),
    };

    assert_eq!(zero.state(), RiskMetricState::Assessed);
    assert_eq!(unknown.state(), RiskMetricState::Unknown);
    assert_eq!(
        RiskMetricValue::NotAssessed.state(),
        RiskMetricState::NotAssessed
    );
    assert_ne!(
        serde_json::to_value(&zero).unwrap(),
        serde_json::to_value(&unknown).unwrap()
    );
    assert_ne!(
        serde_json::to_value(&unknown).unwrap(),
        serde_json::to_value(&RiskMetricValue::NotAssessed).unwrap()
    );
}

#[test]
fn unknown_and_partial_states_are_explicit_and_consistent() {
    let unknown = RiskProfileAssessment {
        status: RiskAssessmentStatus::Unknown,
        dimensions: RiskDimensions {
            data_availability_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
            settlement_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
            bridge_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
            exit_mechanism_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
            operator_dependency_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
            decentralization_score: RiskMetricValue::Unknown {
                reason: "not observable in the current evidence set".to_string(),
            },
        },
        overall: OverallRiskStatus::Unknown {
            reason: "the aggregate band is not observable".to_string(),
        },
    };
    unknown.validate(0).unwrap();

    let mut partial = assessed_profile();
    partial.assessment.status = RiskAssessmentStatus::PartiallyAssessed;
    partial.assessment.dimensions.bridge_score = RiskMetricValue::Unknown {
        reason: "bridge evidence is pending".to_string(),
    };
    partial.assessment.overall = OverallRiskStatus::Unknown {
        reason: "the aggregate band is withheld for partial evidence".to_string(),
    };
    partial.validate().unwrap();
}

#[test]
fn direct_profile_decode_rejects_semantic_states_without_explicit_validate() {
    let mut mismatch = serde_json::to_value(assessed_profile()).unwrap();
    mismatch["target"] = json!({
        "kind": "chain",
        "chain": "bitcoin",
        "family": "evm"
    });
    assert!(serde_json::from_value::<RiskProfile>(mismatch).is_err());

    let mut inconsistent_status = serde_json::to_value(assessed_profile()).unwrap();
    inconsistent_status["assessment"]["dimensions"]["bridge_score"] =
        json!({"state": "not_assessed"});
    assert!(serde_json::from_value::<RiskProfile>(inconsistent_status).is_err());

    let mut blank_unknown = serde_json::to_value(unknown_profile()).unwrap();
    blank_unknown["assessment"]["dimensions"]["bridge_score"]["reason"] = json!(" ");
    assert!(serde_json::from_value::<RiskProfile>(blank_unknown).is_err());

    let mut no_evidence = serde_json::to_value(assessed_profile()).unwrap();
    no_evidence["evidence_refs"] = json!([]);
    assert!(serde_json::from_value::<RiskProfile>(no_evidence).is_err());
}

#[test]
fn direct_profile_decode_rejects_invalid_static_policy_and_rail_metadata() {
    let invalid_policy = StaticPolicyAssumptions {
        trust_tier: TrustTier::Strict,
        verification_class: VerificationClass::ExternalQuorum,
        finality_class: FinalityClass::Probabilistic,
    };
    assert!(serde_json::from_value::<StaticPolicyAssumptions>(
        serde_json::to_value(invalid_policy.clone()).unwrap()
    )
    .is_err());

    let mut policy_profile = serde_json::to_value(assessed_profile()).unwrap();
    policy_profile["static_policy"] = serde_json::to_value(invalid_policy).unwrap();
    assert!(serde_json::from_value::<RiskProfile>(policy_profile).is_err());

    let invalid_schema = VersionedRailMetadata {
        schema_version: RISK_PROFILE_SCHEMA_VERSION + 1,
        metadata: rail_metadata(ChainFamily::BitcoinUtxo),
    };
    assert!(serde_json::from_value::<VersionedRailMetadata>(
        serde_json::to_value(invalid_schema).unwrap()
    )
    .is_err());

    let mismatched = VersionedRailMetadata {
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        metadata: rail_metadata(ChainFamily::Evm),
    };
    let mut rail_profile = serde_json::to_value(assessed_profile()).unwrap();
    rail_profile["rail_metadata"] = serde_json::to_value(mismatched).unwrap();
    assert!(serde_json::from_value::<RiskProfile>(rail_profile).is_err());
}

#[test]
fn strict_v1_profile_decode_rejects_unknown_or_typo_fields() {
    let mut profile = serde_json::to_value(assessed_profile()).unwrap();
    profile["governance_reff"] = json!("typo");
    assert!(serde_json::from_value::<RiskProfile>(profile).is_err());

    let mut set: Value = serde_json::from_str(canonical_risk_profile_set_json()).unwrap();
    set["profiles"][0]["assessment"]["typo_field"] = json!(true);
    assert!(serde_json::from_value::<CanonicalRiskProfileSet>(set).is_err());

    let mut rail = serde_json::to_value(assessed_profile()).unwrap();
    rail["rail_metadata"] = serde_json::to_value(VersionedRailMetadata {
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        metadata: rail_metadata(ChainFamily::BitcoinUtxo),
    })
    .unwrap();
    rail["rail_metadata"]["metadata"]["trust_assumptions"]["security_anchro"] = json!("typo");
    assert!(serde_json::from_value::<RiskProfile>(rail).is_err());
}

#[test]
fn unsupported_schema_version_fails_closed_during_decode() {
    let mut profile = serde_json::to_value(assessed_profile()).unwrap();
    profile["schema_version"] = json!(RISK_PROFILE_SCHEMA_VERSION + 1);
    assert!(serde_json::from_value::<RiskProfile>(profile).is_err());

    let mut set: Value = serde_json::from_str(canonical_risk_profile_set_json()).unwrap();
    set["schema_version"] = json!(RISK_PROFILE_SCHEMA_VERSION + 1);
    assert!(serde_json::from_value::<CanonicalRiskProfileSet>(set).is_err());
}

#[test]
fn profile_revision_version_provenance_and_date_are_validated() {
    let mut revision = assessed_profile();
    revision.profile_revision = 0;
    assert!(revision.validate().is_err());

    let mut version = assessed_profile();
    version.profile_set_version = "v1".to_string();
    assert!(version.validate().is_err());

    let mut governance = assessed_profile();
    governance.governance_ref.clear();
    assert!(governance.validate().is_err());

    let mut rationale = assessed_profile();
    rationale.rationale = "   ".to_string();
    assert!(rationale.validate().is_err());

    let mut invalid_date: Value = serde_json::to_value(assessed_profile()).unwrap();
    invalid_date["effective_date"] = json!("2026-02-30");
    assert!(serde_json::from_value::<RiskProfile>(invalid_date).is_err());
}

#[test]
fn assessed_and_partial_profiles_require_evidence_but_not_assessed_does_not() {
    let mut assessed = assessed_profile();
    assessed.evidence_refs.clear();
    assert!(assessed.validate().is_err());

    let mut partial = assessed_profile();
    partial.assessment.status = RiskAssessmentStatus::PartiallyAssessed;
    partial.assessment.dimensions.bridge_score = RiskMetricValue::NotAssessed;
    partial.assessment.overall = OverallRiskStatus::Unknown {
        reason: "aggregate withheld".to_string(),
    };
    partial.evidence_refs.clear();
    assert!(partial.validate().is_err());

    let not_assessed = RiskProfile {
        target: RiskTarget::Family {
            family: ChainFamily::BitcoinUtxo,
        },
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        profile_set_version: "1.0.0".to_string(),
        profile_revision: 1,
        effective_date: NaiveDate::from_ymd_opt(2026, 7, 21).unwrap(),
        governance_ref: "github:Conxian/lib-conxian-core#177".to_string(),
        evidence_refs: vec![],
        rationale: "Awaiting approved evidence.".to_string(),
        assessment: not_assessed_assessment(),
        static_policy: None,
        rail_metadata: None,
    };
    not_assessed.validate().unwrap();
}

#[test]
fn canonical_set_rejects_duplicate_and_missing_targets() {
    let canonical = canonical_risk_profile_set().unwrap();

    let mut duplicate = (*canonical).clone();
    duplicate.profiles.push(duplicate.profiles[0].clone());
    assert!(duplicate
        .validate()
        .unwrap_err()
        .to_string()
        .contains("duplicate"));
    assert!(serde_json::from_value::<CanonicalRiskProfileSet>(
        serde_json::to_value(duplicate).unwrap()
    )
    .is_err());

    let mut missing = (*canonical).clone();
    missing.profiles.pop();
    assert!(missing
        .validate()
        .unwrap_err()
        .to_string()
        .contains("missing"));
    assert!(serde_json::from_value::<CanonicalRiskProfileSet>(
        serde_json::to_value(missing).unwrap()
    )
    .is_err());
}

#[test]
fn profile_set_version_accepts_semver_major_one_and_rejects_other_majors() {
    let canonical = canonical_risk_profile_set().unwrap();
    let mut compatible = (*canonical).clone();
    compatible.profile_set_version = "1.1.0".to_string();
    for profile in &mut compatible.profiles {
        profile.profile_set_version = "1.1.0".to_string();
    }
    let decoded: CanonicalRiskProfileSet =
        serde_json::from_value(serde_json::to_value(compatible).unwrap()).unwrap();
    assert_eq!(decoded.profile_set_version, "1.1.0");

    let mut mismatch = (*canonical).clone();
    mismatch.profile_set_version = "1.1.0".to_string();
    assert!(serde_json::from_value::<CanonicalRiskProfileSet>(
        serde_json::to_value(mismatch).unwrap()
    )
    .is_err());

    for version in ["2.0.0", "0.9.0"] {
        let mut set = (*canonical).clone();
        set.profile_set_version = version.to_string();
        for profile in &mut set.profiles {
            profile.profile_set_version = version.to_string();
        }
        assert!(serde_json::from_value::<CanonicalRiskProfileSet>(
            serde_json::to_value(set).unwrap()
        )
        .is_err());
    }
}

#[test]
fn malformed_profile_set_versions_fail_at_profile_decode_boundary() {
    for version in ["1.0", "1.01.0", "1.0.0.0", "v1.0.0", ""] {
        let mut profile = serde_json::to_value(assessed_profile()).unwrap();
        profile["profile_set_version"] = json!(version);
        assert!(
            serde_json::from_value::<RiskProfile>(profile).is_err(),
            "{version}"
        );
    }
}

#[test]
fn every_chain_target_matches_the_canonical_chain_family_mapping() {
    let set = canonical_risk_profile_set().unwrap();
    for profile in &set.profiles {
        if let RiskTarget::Chain { chain, family } = &profile.target {
            assert_eq!(family, &chain_family_for(chain));
        }
    }
}

#[test]
fn all_current_chain_family_mappings_are_explicitly_checked() {
    let expected = [
        // ── Bitcoin L1 ──
        (Chain::Bitcoin, ChainFamily::BitcoinUtxo),
        // ── Bitcoin Native ──
        (Chain::Lightning, ChainFamily::BitcoinUtxo),
        (Chain::Spark, ChainFamily::Statechain),
        (Chain::MercuryLayer, ChainFamily::Statechain),
        (Chain::Second, ChainFamily::Ark),
        (Chain::Arkade, ChainFamily::Ark),
        // ── Sidesystems ──
        (Chain::Stacks, ChainFamily::Anchor),
        (Chain::Liquid, ChainFamily::Federation),
        (Chain::Rootstock, ChainFamily::MergeMined),
        (Chain::Botanix, ChainFamily::Federation),
        (Chain::Citrea, ChainFamily::Rollup),
        (Chain::Alpen, ChainFamily::Rollup),
        (Chain::Arch, ChainFamily::BPoS),
        (Chain::Midl, ChainFamily::BPoS),
        (Chain::Nomic, ChainFamily::BPoS),
        (Chain::SideProtocol, ChainFamily::BPoS),
        // ── Other Bitcoin-adjacent ──
        (Chain::Babylon, ChainFamily::BPoS),
        (Chain::Bob, ChainFamily::AltRollup),
        (Chain::Mezo, ChainFamily::Federation),
        (Chain::Alkanes, ChainFamily::Rollup),
        (Chain::Bevm, ChainFamily::AltLayer1),
        (Chain::Bitlayer, ChainFamily::Federation),
        (Chain::Bsquared, ChainFamily::AltRollup),
        (Chain::Core, ChainFamily::BPoS),
        (Chain::Corn, ChainFamily::AltRollup),
        (Chain::Flashnet, ChainFamily::Hybrid),
        (Chain::Fractal, ChainFamily::MergeMined),
        (Chain::Goat, ChainFamily::AltLayer1),
        (Chain::Hemi, ChainFamily::AltRollup),
        (Chain::InternetComputer, ChainFamily::Hybrid),
        (Chain::Merlin, ChainFamily::AltRollup),
        (Chain::Rgb, ChainFamily::Csv),
        (Chain::Rollux, ChainFamily::AltRollup),
        (Chain::Starknet, ChainFamily::AltRollup),
        // ── Cross-ecosystem ──
        (Chain::Ethereum, ChainFamily::Evm),
        (Chain::Base, ChainFamily::Evm),
        (Chain::Arbitrum, ChainFamily::Evm),
        (Chain::Optimism, ChainFamily::Evm),
        (Chain::Polygon, ChainFamily::Evm),
        (Chain::CosmosHub, ChainFamily::CosmosIbc),
        (Chain::Osmosis, ChainFamily::CosmosIbc),
        (Chain::Celestia, ChainFamily::CosmosIbc),
        (Chain::Solana, ChainFamily::SolanaSvm),
        (Chain::Eclipse, ChainFamily::SolanaSvm),
        (Chain::Aptos, ChainFamily::Move),
        (Chain::Sui, ChainFamily::Move),
        (Chain::Polkadot, ChainFamily::Substrate),
        (Chain::Kusama, ChainFamily::Substrate),
    ];

    assert_eq!(expected.len(), 48);
    for (chain, family) in expected {
        assert_eq!(chain_family_for(&chain), family);
        assert!(canonical_risk_profile_set()
            .unwrap()
            .profile_for_target(&RiskTarget::Chain { chain, family })
            .is_some());
    }
}

#[test]
fn chain_family_mismatches_are_rejected() {
    let profile = RiskProfile {
        target: RiskTarget::Chain {
            chain: Chain::Bitcoin,
            family: ChainFamily::Evm,
        },
        ..assessed_profile()
    };
    assert!(profile.validate().is_err());
}

#[test]
fn profile_lookup_rejects_invalid_input_and_matches_full_target_identity() {
    let set = canonical_risk_profile_set().unwrap();
    let invalid = RiskTarget::Chain {
        chain: Chain::Bitcoin,
        family: ChainFamily::Evm,
    };
    assert!(set.profile_for_target(&invalid).is_none());

    let valid = RiskTarget::Chain {
        chain: Chain::Bitcoin,
        family: ChainFamily::BitcoinUtxo,
    };
    assert!(set.profile_for_target(&valid).is_some());
}

#[test]
fn evidence_provenance_is_typed_closed_and_exact_duplicates_are_rejected() {
    for kind in [
        EvidenceKind::Specification,
        EvidenceKind::Audit,
        EvidenceKind::Research,
        EvidenceKind::Observation,
    ] {
        let evidence = EvidenceReference {
            kind,
            reference: "ref:example".to_string(),
        };
        let encoded = serde_json::to_value(&evidence).unwrap();
        assert_eq!(
            serde_json::from_value::<EvidenceReference>(encoded).unwrap(),
            evidence
        );
    }

    assert!(serde_json::from_value::<EvidenceReference>(json!({
        "kind": "governance",
        "reference": "github:Conxian/lib-conxian-core#177"
    }))
    .is_err());

    let mut blank = serde_json::to_value(assessed_profile()).unwrap();
    blank["evidence_refs"][0]["reference"] = json!(" ");
    assert!(serde_json::from_value::<RiskProfile>(blank).is_err());

    let mut duplicate = serde_json::to_value(assessed_profile()).unwrap();
    duplicate["evidence_refs"] = json!([
        {"kind": "research", "reference": "research:duplicate"},
        {"kind": "research", "reference": "research:duplicate"}
    ]);
    assert!(serde_json::from_value::<RiskProfile>(duplicate).is_err());
}

#[test]
fn static_policy_reuses_existing_trust_tier_validation_without_finality_invention() {
    let strict_light_client = StaticPolicyAssumptions {
        trust_tier: TrustTier::Strict,
        verification_class: VerificationClass::LightClient,
        finality_class: FinalityClass::Probabilistic,
    };
    strict_light_client.validate().unwrap();

    let strict_external_quorum = StaticPolicyAssumptions {
        trust_tier: TrustTier::Strict,
        verification_class: VerificationClass::ExternalQuorum,
        finality_class: FinalityClass::Probabilistic,
    };
    assert!(strict_external_quorum.validate().is_err());

    let managed_external_quorum = StaticPolicyAssumptions {
        trust_tier: TrustTier::Managed,
        verification_class: VerificationClass::ExternalQuorum,
        finality_class: FinalityClass::Deterministic,
    };
    managed_external_quorum.validate().unwrap();

    let expedient_external_quorum = StaticPolicyAssumptions {
        trust_tier: TrustTier::Expedient,
        verification_class: VerificationClass::ExternalQuorum,
        finality_class: FinalityClass::Probabilistic,
    };
    expedient_external_quorum.validate().unwrap();
    assert!(serde_json::from_value::<StaticPolicyAssumptions>(
        serde_json::to_value(expedient_external_quorum).unwrap()
    )
    .is_ok());

    let observer_only = StaticPolicyAssumptions {
        trust_tier: TrustTier::ObserverOnly,
        verification_class: VerificationClass::NativeObservation,
        finality_class: FinalityClass::Probabilistic,
    };
    assert!(observer_only.validate().is_err());
    assert!(
        validate_trust_tier_policy(TrustTier::Managed, VerificationClass::ExternalQuorum).is_ok()
    );
}

#[test]
fn versioned_rail_metadata_requires_matching_target_family() {
    let valid = VersionedRailMetadata {
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        metadata: rail_metadata(ChainFamily::BitcoinUtxo),
    };
    valid.validate(&ChainFamily::BitcoinUtxo).unwrap();

    let mismatch = VersionedRailMetadata {
        schema_version: RISK_PROFILE_SCHEMA_VERSION,
        metadata: rail_metadata(ChainFamily::Evm),
    };
    assert!(mismatch.validate(&ChainFamily::BitcoinUtxo).is_err());

    let mut profile = assessed_profile();
    profile.rail_metadata = Some(valid);
    profile.validate().unwrap();
}

#[test]
fn legacy_risk_and_rail_json_shapes_remain_compatible() {
    let risk_json = r#"{
        "overall_level": "legacy_low",
        "da_score": 0,
        "settlement_score": 100,
        "bridge_score": 50,
        "exit_mechanism_score": 75,
        "operators_score": 25,
        "decentralization_score": 100
    }"#;
    let risk: RiskAssessment = serde_json::from_str(risk_json).unwrap();
    assert_eq!(risk.da_score, 0);
    assert_eq!(risk.operators_score, 25);
    assert_eq!(
        serde_json::to_value(risk).unwrap()["overall_level"],
        "legacy_low"
    );

    let rail = rail_metadata(ChainFamily::BitcoinUtxo);
    let encoded = serde_json::to_value(&rail).unwrap();
    let decoded: RailMetadata = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, rail);
    assert_eq!(encoded["rail_family"], "bitcoin_utxo");
}
