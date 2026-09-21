use conxius_enclave_sdk::enclave::attestation::{
    AttestationLevel, AttestationReportType, DeviceIntegrityReport,
};
use conxius_enclave_sdk::enclave::{
    SignRequest as SdkSignRequest, SigningAlgorithm as SdkSigningAlgorithm,
};
use conxius_enclave_sdk::protocol::bitcoin::{
    FeeBumpStrategy as SdkFeeBumpStrategy, MempoolPolicy as SdkMempoolPolicy,
};
use conxius_enclave_sdk::protocol::rails::TrustTier as SdkTrustTier;
use lib_conxian_core::control_model::{
    validate_trust_tier_policy, Bip110Compliance, Bip110Limits, Bip110PreflightRequest,
    TrustTier as CoreTrustTier, VerificationClass,
};
use lib_conxian_core::signing::{
    SignRequest as CoreSignRequest, SignResponse as CoreSignResponse, SigningOperation,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoundaryError {
    UnknownSdkTrustTier(String),
    UnsupportedBip110Configuration(&'static str),
    NonCanonicalBip110Limits(Bip110Limits),
    InvalidSdkMempoolPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SdkMempoolPolicyEvidence {
    min_relay_fee: u64,
    target_blocks: u32,
    fee_bump_strategy: SdkFeeBumpStrategy,
}

impl From<&SdkMempoolPolicy> for SdkMempoolPolicyEvidence {
    fn from(policy: &SdkMempoolPolicy) -> Self {
        Self {
            min_relay_fee: policy.min_relay_fee,
            target_blocks: policy.target_blocks,
            fee_bump_strategy: policy.fee_bump_strategy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BoundedPolicyEvidence {
    core_bip110_enabled: bool,
    core_limits: Bip110Limits,
    sdk_mempool_policy: SdkMempoolPolicyEvidence,
}

fn core_trust_tier_to_sdk(tier: &CoreTrustTier) -> SdkTrustTier {
    match tier {
        CoreTrustTier::Strict => SdkTrustTier::T1,
        CoreTrustTier::Managed => SdkTrustTier::T2,
        CoreTrustTier::Expedient => SdkTrustTier::T3,
        CoreTrustTier::ObserverOnly => SdkTrustTier::T4,
    }
}

fn sdk_trust_tier_to_core(tier: SdkTrustTier) -> CoreTrustTier {
    match tier {
        SdkTrustTier::T1 => CoreTrustTier::Strict,
        SdkTrustTier::T2 => CoreTrustTier::Managed,
        SdkTrustTier::T3 => CoreTrustTier::Expedient,
        SdkTrustTier::T4 => CoreTrustTier::ObserverOnly,
    }
}

fn parse_sdk_trust_tier(value: &str) -> Result<SdkTrustTier, BoundaryError> {
    match value {
        "T1" => Ok(SdkTrustTier::T1),
        "T2" => Ok(SdkTrustTier::T2),
        "T3" => Ok(SdkTrustTier::T3),
        "T4" => Ok(SdkTrustTier::T4),
        other => Err(BoundaryError::UnknownSdkTrustTier(other.to_string())),
    }
}

/// The SDK release has no BIP-110 type or validator. This local evidence
/// adapter therefore preserves Core's canonical enabled policy and records the
/// SDK's independent mempool configuration without claiming semantic
/// enforcement by the SDK. Unsupported Core configurations fail closed.
fn adapt_bip110_policy(
    core_compliance: &Bip110Compliance,
    sdk_policy: &SdkMempoolPolicy,
) -> Result<BoundedPolicyEvidence, BoundaryError> {
    if !core_compliance.is_enabled() {
        return Err(BoundaryError::UnsupportedBip110Configuration(
            "disabled_core_policy",
        ));
    }

    let limits = *core_compliance.limits();
    if limits != Bip110Limits::canonical() {
        return Err(BoundaryError::NonCanonicalBip110Limits(limits));
    }

    Ok(BoundedPolicyEvidence {
        core_bip110_enabled: true,
        core_limits: limits,
        sdk_mempool_policy: SdkMempoolPolicyEvidence::from(sdk_policy),
    })
}

fn decode_sdk_mempool_policy(value: Value) -> Result<SdkMempoolPolicy, BoundaryError> {
    serde_json::from_value(value).map_err(|_| BoundaryError::InvalidSdkMempoolPolicy)
}

fn fixture(name: &str) -> Value {
    let contents = match name {
        "signing_boundary.json" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/signing_boundary.json"
        )),
        "bip110_preflight.json" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/bip110_preflight.json"
        )),
        other => panic!("unknown Core fixture {other}"),
    };

    serde_json::from_str(contents).expect("Core fixture must be valid JSON")
}

#[test]
fn trust_tier_mapping_is_explicit_and_serde_round_trips() {
    let cases = [
        (CoreTrustTier::Strict, SdkTrustTier::T1),
        (CoreTrustTier::Managed, SdkTrustTier::T2),
        (CoreTrustTier::Expedient, SdkTrustTier::T3),
        (CoreTrustTier::ObserverOnly, SdkTrustTier::T4),
    ];

    for (core_tier, sdk_tier) in cases {
        assert_eq!(core_trust_tier_to_sdk(&core_tier), sdk_tier);
        assert_eq!(sdk_trust_tier_to_core(sdk_tier), core_tier);

        let core_json = serde_json::to_value(&core_tier).expect("Core trust tier serializes");
        let decoded_core: CoreTrustTier =
            serde_json::from_value(core_json).expect("Core trust tier deserializes");
        assert_eq!(decoded_core, core_tier);

        let sdk_json = serde_json::to_value(sdk_tier).expect("SDK trust tier serializes");
        let decoded_sdk: SdkTrustTier =
            serde_json::from_value(sdk_json).expect("SDK trust tier deserializes");
        assert_eq!(decoded_sdk, sdk_tier);
    }

    // The adapter maps representation only. Core's production policy still
    // rejects ObserverOnly independently of the SDK's T4 representation.
    assert!(validate_trust_tier_policy(
        CoreTrustTier::ObserverOnly,
        VerificationClass::LightClient,
    )
    .is_err());
}

#[test]
fn unknown_sdk_trust_tiers_fail_closed() {
    assert!(matches!(
        parse_sdk_trust_tier("T5"),
        Err(BoundaryError::UnknownSdkTrustTier(value)) if value == "T5"
    ));
    assert!(matches!(
        parse_sdk_trust_tier("observer_only"),
        Err(BoundaryError::UnknownSdkTrustTier(value)) if value == "observer_only"
    ));
    assert!(serde_json::from_str::<SdkTrustTier>(r#""T5""#).is_err());
}

#[test]
fn canonical_bip110_policy_is_preserved_with_independent_sdk_policy() {
    let core_compliance = Bip110Compliance::new();
    let sdk_policy = SdkMempoolPolicy::default_sovereign();
    let evidence = adapt_bip110_policy(&core_compliance, &sdk_policy)
        .expect("canonical Core policy should produce evidence");

    assert!(evidence.core_bip110_enabled);
    assert_eq!(evidence.core_limits, Bip110Limits::canonical());
    assert_eq!(
        evidence.sdk_mempool_policy.min_relay_fee,
        sdk_policy.min_relay_fee
    );
    assert_eq!(
        evidence.sdk_mempool_policy.target_blocks,
        sdk_policy.target_blocks
    );
    assert_eq!(
        evidence.sdk_mempool_policy.fee_bump_strategy,
        sdk_policy.fee_bump_strategy
    );

    let encoded = serde_json::to_value(&evidence).expect("policy evidence serializes");
    let decoded: BoundedPolicyEvidence =
        serde_json::from_value(encoded).expect("policy evidence deserializes");
    assert_eq!(decoded, evidence);

    let sdk_encoded = serde_json::to_value(&sdk_policy).expect("SDK policy serializes");
    let sdk_decoded: SdkMempoolPolicy =
        serde_json::from_value(sdk_encoded).expect("SDK policy deserializes");
    assert_eq!(sdk_decoded.min_relay_fee, sdk_policy.min_relay_fee);
    assert_eq!(sdk_decoded.target_blocks, sdk_policy.target_blocks);
    assert_eq!(sdk_decoded.fee_bump_strategy, sdk_policy.fee_bump_strategy);
}

#[test]
fn unsupported_bip110_and_sdk_policy_values_fail_closed() {
    let sdk_policy = SdkMempoolPolicy::default_sovereign();
    let disabled = Bip110Compliance::disabled();
    assert!(matches!(
        adapt_bip110_policy(&disabled, &sdk_policy),
        Err(BoundaryError::UnsupportedBip110Configuration(
            "disabled_core_policy"
        ))
    ));

    let non_canonical = Bip110Compliance::with_limits(Bip110Limits {
        max_pushdata_bytes: 255,
        ..Bip110Limits::canonical()
    });
    assert!(matches!(
        adapt_bip110_policy(&non_canonical, &sdk_policy),
        Err(BoundaryError::NonCanonicalBip110Limits(_))
    ));

    let invalid_sdk_policy = json!({
        "min_relay_fee": 1000,
        "target_blocks": 3,
        "fee_bump_strategy": "unsupported"
    });
    assert!(matches!(
        decode_sdk_mempool_policy(invalid_sdk_policy),
        Err(BoundaryError::InvalidSdkMempoolPolicy)
    ));
}

#[test]
fn existing_core_signing_and_bip110_fixtures_remain_compatible() {
    let signing = fixture("signing_boundary.json");
    assert_eq!(signing["api_version"], 1);

    let request: CoreSignRequest =
        serde_json::from_value(signing["request"].clone()).expect("Core request fixture");
    let response: CoreSignResponse =
        serde_json::from_value(signing["response"].clone()).expect("Core response fixture");
    assert_eq!(request.operation(), SigningOperation::SignMessage);
    assert_eq!(
        serde_json::to_value(&request).expect("Core request re-serializes"),
        signing["request"]
    );
    assert_eq!(
        serde_json::to_value(&response).expect("Core response re-serializes"),
        signing["response"]
    );

    let bip110 = fixture("bip110_preflight.json");
    let success: Bip110PreflightRequest =
        serde_json::from_value(bip110["success"]["request"].clone())
            .expect("successful BIP-110 fixture request");
    assert!(success.validate().is_compliant);

    let failure: Bip110PreflightRequest =
        serde_json::from_value(bip110["failure"]["request"].clone())
            .expect("failing BIP-110 fixture request");
    let failure_result = failure.validate();
    assert!(!failure_result.is_compliant);
    assert_eq!(failure_result.findings.len(), 5);

    let unsupported_version: Bip110PreflightRequest =
        serde_json::from_value(bip110["unsupported_version"]["request"].clone())
            .expect("unsupported-version BIP-110 fixture request");
    assert!(!unsupported_version.validate().is_compliant);
}

#[test]
fn sdk_signing_and_attestation_dtos_round_trip_without_invoking_hardware() {
    let sdk_request = SdkSignRequest {
        algorithm: SdkSigningAlgorithm::EcdsaSecp256k1,
        message_hash: vec![0x11; 32],
        derivation_path: "m/84'/0'/0'/0/0".to_string(),
        key_id: "fixture-key".to_string(),
        taproot_tweak: None,
    };
    let encoded_request = serde_json::to_value(&sdk_request).expect("SDK request serializes");
    let decoded_request: SdkSignRequest =
        serde_json::from_value(encoded_request).expect("SDK request deserializes");
    assert_eq!(decoded_request.message_hash, sdk_request.message_hash);
    assert_eq!(decoded_request.derivation_path, sdk_request.derivation_path);
    assert_eq!(decoded_request.key_id, sdk_request.key_id);

    let report = DeviceIntegrityReport {
        report_version: 1,
        report_type: AttestationReportType::DeviceIntegrity,
        level: AttestationLevel::TEE,
        challenge_nonce: vec![1, 2, 3, 4],
        signature: vec![0; 64],
        attested_operation_public_key: vec![0xAA; 32],
        signer_key_binding: None,
        certificate_chain: vec![
            "fixture-public-key".to_string(),
            "CONCLAVE_ROOT_CA_V1".to_string(),
        ],
        timestamp: 1_000,
        extension_data: "fixture".to_string(),
        extensions: vec![],
    };
    let encoded_report = serde_json::to_value(&report).expect("SDK attestation serializes");
    let decoded_report: DeviceIntegrityReport =
        serde_json::from_value(encoded_report).expect("SDK attestation deserializes");
    assert_eq!(decoded_report.challenge_nonce, report.challenge_nonce);
    assert_eq!(decoded_report.signature, report.signature);
    assert_eq!(
        decoded_report.get_device_fingerprint(),
        report.get_device_fingerprint()
    );
    assert!(!decoded_report.verify(&[9, 9, 9]));
}
