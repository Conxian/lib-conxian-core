mod support;

use lib_conxian_core::control_model::{
    Bip110Compliance, Bip110PreflightError, Bip110PreflightFinding, Bip110PreflightRequest,
    Bip110PreflightValidator, Bip110PreflightViolationCode, BIP110_PREFLIGHT_API_VERSION,
};
use lib_conxian_core::signing::{SigningError, UniversalChainSigner};
use lib_conxian_core::verifier::{
    ProtocolVerifier, ProtocolVerifierError, VerifierCapability,
    PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION,
};
use serde::Deserialize;
use support::{
    assert_json_round_trip, assert_json_structural_round_trip, fixed_now, load_fixture,
    DeterministicSigner, DeterministicVerifierBackend, SignerResponseMode,
};

#[derive(Debug, Deserialize)]
struct SigningFailuresFixture {
    schema_version: u16,
    contract: String,
    api_version: u16,
    cases: Vec<SigningFailureCase>,
}

#[derive(Debug, Deserialize)]
struct SigningFailureCase {
    id: String,
    request: lib_conxian_core::signing::SignRequest,
    response_mode: Option<String>,
    expected_error: String,
}

#[derive(Debug, Deserialize)]
struct VerifierSuccessFixture {
    schema_version: u16,
    contract: String,
    contract_version: u16,
    evidence_binding_version: u8,
    fixed_now: String,
    capabilities: lib_conxian_core::verifier::VerifierCapabilities,
    state_request: lib_conxian_core::verifier::ProofVerificationRequest,
    state_result: lib_conxian_core::verifier::ProofVerificationResult,
    finality_request: lib_conxian_core::verifier::TransactionFinalityRequest,
    finality_result: lib_conxian_core::verifier::TransactionFinalityResult,
}

#[derive(Debug, Deserialize)]
struct VerifierFailuresFixture {
    schema_version: u16,
    contract: String,
    contract_version: u16,
    evidence_binding_version: u8,
    fixed_now: String,
    cases: Vec<VerifierFailureCase>,
}

#[derive(Debug, Deserialize)]
struct VerifierFailureCase {
    id: String,
    kind: String,
    state_request: Option<lib_conxian_core::verifier::ProofVerificationRequest>,
    finality_request: Option<lib_conxian_core::verifier::TransactionFinalityRequest>,
    finality_result: Option<lib_conxian_core::verifier::TransactionFinalityResult>,
    expected_error: String,
}

#[derive(Debug, Deserialize)]
struct Bip110Fixture {
    schema_version: u16,
    contract: String,
    api_version: u16,
    cases: Vec<Bip110Case>,
}

#[derive(Debug, Deserialize)]
struct Bip110Case {
    id: String,
    request: Bip110PreflightRequest,
    expected_compliant: bool,
    expected_finding_codes: Vec<String>,
}

fn signing_error_code(error: &SigningError) -> &'static str {
    match error {
        SigningError::InvalidTarget { .. } => "invalid_target",
        SigningError::UnsupportedChain { .. } => "unsupported_chain",
        SigningError::UnsupportedAlgorithm { .. } => "unsupported_algorithm",
        SigningError::UnsupportedOperation { .. } => "unsupported_operation",
        SigningError::InvalidPayload(_) => "invalid_payload",
        SigningError::InvalidDerivationPath(_) => "invalid_derivation_path",
        SigningError::InvalidAddress(_) => "invalid_address",
        SigningError::InvalidRequest(_) => "invalid_request",
        SigningError::InvalidSignature => "invalid_signature",
        SigningError::InvalidVerificationKey => "invalid_verification_key",
        SigningError::InvalidResponse(_) => "invalid_response",
        SigningError::BackendFailure => "backend_failure",
    }
}

fn verifier_error_code(error: &ProtocolVerifierError) -> &'static str {
    match error {
        ProtocolVerifierError::UnsupportedCapability { .. } => "unsupported_capability",
        ProtocolVerifierError::InsufficientProofData { .. } => "insufficient_proof_data",
        ProtocolVerifierError::ExpiredEvidence { .. } => "expired_evidence",
        ProtocolVerifierError::PolicyBlocked { .. } => "policy_blocked",
        ProtocolVerifierError::NonFinalState { .. } => "non_final_state",
        other => panic!("unexpected verifier error in fixture test: {other:?}"),
    }
}

#[test]
fn signer_failures_are_rejected_by_capabilities_validation_and_response_postconditions() {
    let fixture: SigningFailuresFixture = load_fixture("signing_failures.json");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "universal_chain_signer");
    assert_eq!(
        fixture.api_version,
        lib_conxian_core::signing::UNIVERSAL_CHAIN_SIGNER_API_VERSION
    );

    for case in fixture.cases {
        let signer =
            DeterministicSigner::new(SignerResponseMode::from_wire(case.response_mode.as_deref()));
        assert_json_round_trip(&case.request);
        let error = signer
            .sign(&case.request)
            .expect_err(&format!("case {} must fail", case.id));
        assert_eq!(
            signing_error_code(&error),
            case.expected_error,
            "case {}",
            case.id
        );
    }
}

#[test]
fn verifier_success_fixtures_use_fixed_clock_facade_methods_and_round_trip_structurally() {
    let fixture: VerifierSuccessFixture = load_fixture("verifier_success.json");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "protocol_verifier");
    assert_eq!(fixture.contract_version, 1);
    assert_eq!(
        fixture.evidence_binding_version,
        PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION
    );
    let now = fixed_now();
    assert_eq!(
        fixture
            .fixed_now
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap(),
        now
    );
    assert_json_structural_round_trip(&fixture.state_request);
    assert_json_round_trip(&fixture.state_result);
    assert_json_round_trip(&fixture.finality_request);
    assert_json_round_trip(&fixture.finality_result);
    assert_json_round_trip(&fixture.capabilities);

    let backend = DeterministicVerifierBackend::new(
        fixture.capabilities.clone(),
        Some(fixture.state_result.clone()),
        Some(fixture.finality_result.clone()),
    );
    let verifier = ProtocolVerifier::new(backend.clone());

    let state = verifier
        .verify_chain_state_at(&fixture.state_request, now)
        .expect("fixture proof succeeds at fixed clock");
    assert_eq!(state, fixture.state_result);

    let latest = verifier
        .get_latest_verified_block_at(&fixture.state_request.chain, now)
        .expect("fixture latest block succeeds at fixed clock");
    assert_eq!(latest, fixture.state_result.verified_block);

    let finality = verifier
        .verify_transaction_finality_at(&fixture.finality_request, now)
        .expect("fixture finality succeeds at fixed clock");
    assert_eq!(finality, fixture.finality_result);
    assert_eq!(backend.state_calls(), 1);
    assert_eq!(backend.latest_calls(), 1);
    assert_eq!(backend.finality_calls(), 1);
}

#[test]
fn verifier_failure_fixtures_fail_closed_without_ambient_time() {
    let success: VerifierSuccessFixture = load_fixture("verifier_success.json");
    let failures: VerifierFailuresFixture = load_fixture("verifier_failures.json");
    let now = fixed_now();
    assert_eq!(failures.schema_version, 1);
    assert_eq!(failures.contract, "protocol_verifier");
    assert_eq!(failures.contract_version, 1);
    assert_eq!(
        failures.evidence_binding_version,
        PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION
    );
    assert_eq!(
        failures
            .fixed_now
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap(),
        now
    );

    for case in failures.cases {
        match case.kind.as_str() {
            "malformed_proof" | "stale_evidence" | "policy_blocked" => {
                let backend = DeterministicVerifierBackend::new(
                    success.capabilities.clone(),
                    Some(success.state_result.clone()),
                    Some(success.finality_result.clone()),
                );
                let verifier = ProtocolVerifier::new(backend.clone());
                let request = case.state_request.expect("state failure request");
                let error = verifier
                    .verify_chain_state_at(&request, now)
                    .expect_err(&format!("case {} must fail", case.id));
                assert_eq!(
                    verifier_error_code(&error),
                    case.expected_error,
                    "case {}",
                    case.id
                );
                assert_eq!(
                    backend.state_calls(),
                    0,
                    "case {} must fail before backend",
                    case.id
                );
            }
            "non_final" => {
                let backend = DeterministicVerifierBackend::new(
                    success.capabilities.clone(),
                    None,
                    case.finality_result.clone(),
                );
                let verifier = ProtocolVerifier::new(backend.clone());
                let request = case.finality_request.expect("finality failure request");
                let error = verifier
                    .verify_transaction_finality_at(&request, now)
                    .expect_err(&format!("case {} must fail", case.id));
                assert_eq!(
                    verifier_error_code(&error),
                    case.expected_error,
                    "case {}",
                    case.id
                );
                assert_eq!(backend.finality_calls(), 1);
            }
            "unsupported_capability" => {
                let mut capabilities = success.capabilities.clone();
                capabilities
                    .capabilities
                    .retain(|capability| *capability != VerifierCapability::StateProofVerification);
                let backend = DeterministicVerifierBackend::new(capabilities, None, None);
                let verifier = ProtocolVerifier::new(backend.clone());
                let request = case.state_request.expect("unsupported capability request");
                let error = verifier
                    .verify_chain_state_at(&request, now)
                    .expect_err(&format!("case {} must fail", case.id));
                assert_eq!(
                    verifier_error_code(&error),
                    case.expected_error,
                    "case {}",
                    case.id
                );
                assert_eq!(backend.state_calls(), 0);
            }
            other => panic!("unknown verifier fixture case kind {other}"),
        }
    }
}

#[test]
fn bip110_fixture_replay_preserves_versions_fail_closed_cases_and_finding_order() {
    let fixture: Bip110Fixture = load_fixture("bip110_cases.json");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "bip110_preflight");
    assert_eq!(fixture.api_version, BIP110_PREFLIGHT_API_VERSION);

    for case in fixture.cases {
        let encoded = serde_json::to_value(&case.request).expect("BIP-110 request serializes");
        let decoded: Bip110PreflightRequest =
            serde_json::from_value(encoded.clone()).expect("BIP-110 request deserializes");
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            encoded,
            "case {}",
            case.id
        );

        let result = case.request.validate();
        let finding_codes: Vec<_> = result
            .findings
            .iter()
            .map(|finding| finding.code())
            .collect();
        assert_eq!(
            result.is_compliant, case.expected_compliant,
            "case {}",
            case.id
        );
        assert_eq!(
            finding_codes, case.expected_finding_codes,
            "case {}",
            case.id
        );

        if case.id == "bip110.multiple_violations" {
            let fields: Vec<_> = result
                .findings
                .iter()
                .map(|finding| match finding {
                    Bip110PreflightFinding::Violation(violation) => violation.code,
                    Bip110PreflightFinding::Error(error) => {
                        panic!("unexpected structural finding {error:?}")
                    }
                })
                .collect();
            assert_eq!(
                fields,
                vec![
                    Bip110PreflightViolationCode::PushdataExceedsLimit,
                    Bip110PreflightViolationCode::PushdataExceedsLimit,
                    Bip110PreflightViolationCode::OpReturnExceedsLimit,
                    Bip110PreflightViolationCode::OpReturnExceedsLimit,
                    Bip110PreflightViolationCode::ScriptPubkeyExceedsLimit,
                    Bip110PreflightViolationCode::ScriptPubkeyExceedsLimit,
                    Bip110PreflightViolationCode::WitnessElementExceedsLimit,
                    Bip110PreflightViolationCode::WitnessElementExceedsLimit,
                    Bip110PreflightViolationCode::TaprootControlBlockExceedsLimit,
                    Bip110PreflightViolationCode::TaprootControlBlockExceedsLimit,
                ]
            );
        }
    }

    let disabled = Bip110PreflightValidator::with_compliance(Bip110Compliance::disabled())
        .expect_err("disabled compliance must be rejected");
    assert!(matches!(disabled, Bip110PreflightError::ComplianceDisabled));
}
