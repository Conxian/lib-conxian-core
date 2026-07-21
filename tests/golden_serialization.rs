mod support;

use lib_conxian_core::signing::{
    AddressDerivationRequest, SignRequest, SignResponse, UniversalChainSigner, VerificationRequest,
    VerificationResult, UNIVERSAL_CHAIN_SIGNER_API_VERSION,
};
use lib_conxian_core::verifier::PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use support::{
    assert_json_round_trip, load_fixture, load_fixture_value, load_manifest, DeterministicSigner,
    SignerResponseMode,
};

const LEGACY_BOUNDARY_FIXTURES: [&str; 4] = [
    "adapter_contracts.json",
    "bip110_preflight.json",
    "signing_boundary.json",
    "verifier_boundary.json",
];

#[derive(Debug, Deserialize)]
struct SigningSuccessFixture {
    schema_version: u16,
    contract: String,
    api_version: u16,
    cases: Vec<SigningSuccessCase>,
}

#[derive(Debug, Deserialize)]
struct SigningSuccessCase {
    id: String,
    request: SignRequest,
    response: SignResponse,
    verification_request: VerificationRequest,
    verification_result: VerificationResult,
}

fn normalized_manifest_outcome(file: &str, fixture: &Value, case: &Value) -> &'static str {
    let case_id = case["id"].as_str().unwrap_or("<missing case id>");

    match file {
        "signing_success.json" => {
            assert!(
                case["response"].is_object() && case["verification_result"].is_object(),
                "fixture case {case_id} in {file} must record response and verification_result"
            );
            "success"
        }
        "signing_failures.json" | "verifier_failures.json" => {
            assert!(
                case["expected_error"]
                    .as_str()
                    .is_some_and(|error| !error.is_empty()),
                "fixture case {case_id} in {file} must record a non-empty expected_error"
            );
            "failure"
        }
        "verifier_success.json" => {
            let result_key = match case["kind"].as_str() {
                Some("proof") => "state_result",
                Some("finality") => "finality_result",
                Some(kind) => panic!(
                    "fixture case {case_id} in {file} has unknown success result kind {kind}"
                ),
                None => panic!("fixture case {case_id} in {file} is missing kind"),
            };
            assert!(
                fixture[result_key].is_object(),
                "fixture case {case_id} in {file} must map to top-level {result_key}"
            );
            "success"
        }
        "bip110_cases.json" => match case["expected_compliant"].as_bool() {
            Some(true) => "success",
            Some(false) => "failure",
            None => {
                panic!("fixture case {case_id} in {file} is missing boolean expected_compliant")
            }
        },
        "adapter_cases.json" => match case["kind"].as_str() {
            Some("bitcoin_tx_params" | "babylon_staking_intent") => {
                assert!(
                    case["expected"].is_object(),
                    "fixture case {case_id} in {file} must record expected adapter metadata"
                );
                "success"
            }
            Some("liquid_proof") => {
                assert!(
                    case["proof"].as_str().is_some() && case["malformed_proof"].as_str().is_some(),
                    "fixture case {case_id} in {file} must record both proof and malformed_proof"
                );
                "mixed"
            }
            Some("stacks_sbtc_intent") => {
                assert!(
                    case["expected"].is_object(),
                    "fixture case {case_id} in {file} must record expected sBTC outcomes"
                );
                "mixed"
            }
            Some("rgb_rollout") => {
                let rollout_cases = case["cases"].as_array().unwrap_or_else(|| {
                    panic!("fixture case {case_id} in {file} is missing rollout cases")
                });
                assert!(
                    !rollout_cases.is_empty()
                        && rollout_cases
                            .iter()
                            .all(|rollout_case| rollout_case["expected"].is_string()),
                    "fixture case {case_id} in {file} must record expected outcomes for every rollout case"
                );
                "mixed"
            }
            Some("dlc_intent") => {
                assert!(
                    case["expected_attestation"].is_boolean(),
                    "fixture case {case_id} in {file} must record expected_attestation"
                );
                "mixed"
            }
            Some(kind) => {
                panic!("fixture case {case_id} in {file} has unknown adapter kind {kind}")
            }
            None => panic!("fixture case {case_id} in {file} is missing kind"),
        },
        other => panic!("manifest references unsupported cases fixture {other}"),
    }
}

#[test]
fn manifest_and_every_golden_file_are_versioned_and_accounted_for() {
    let manifest = load_manifest();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.package_name, "lib-conxian-core");
    assert_eq!(manifest.package_version, "0.3.0");
    assert_eq!(
        manifest
            .contract_versions
            .universal_chain_signer_api_version,
        UNIVERSAL_CHAIN_SIGNER_API_VERSION
    );
    assert_eq!(manifest.contract_versions.bip110_preflight_api_version, 1);
    assert_eq!(
        manifest
            .contract_versions
            .protocol_verifier_evidence_binding_version,
        PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION
    );
    assert_eq!(
        manifest.evidence_binding_domain,
        "lib-conxian-core/protocol-verifier/evidence-binding"
    );

    let mut declared_ids = BTreeSet::new();
    let mut files = BTreeSet::new();
    for fixture in &manifest.fixtures {
        assert!(
            fixture.id.contains('.'),
            "fixture id is not namespaced: {}",
            fixture.id
        );
        assert!(matches!(
            fixture.outcome.as_str(),
            "success" | "failure" | "mixed"
        ));
        assert!(
            declared_ids.insert(fixture.id.clone()),
            "duplicate fixture id {}",
            fixture.id
        );
        files.insert(fixture.file.clone());
    }

    let manifest_files = files.clone();
    let mut file_cases = BTreeMap::new();
    for file in files {
        let value = load_fixture_value(&file);
        assert_eq!(value["schema_version"], Value::from(1));
        let cases = value["cases"]
            .as_array()
            .unwrap_or_else(|| panic!("fixture {file} is missing a cases array"));
        let mut cases_by_id = BTreeMap::new();
        for case in cases {
            let id = case["id"]
                .as_str()
                .unwrap_or_else(|| panic!("fixture {file} contains a case without an id"));
            assert!(
                cases_by_id.insert(id.to_string(), case.clone()).is_none(),
                "duplicate case id {id} in {file}"
            );
        }
        file_cases.insert(file, cases_by_id);
    }

    for row in &manifest.fixtures {
        let cases = file_cases.get(&row.file).unwrap_or_else(|| {
            panic!(
                "manifest row {} names missing fixture file {}",
                row.id, row.file
            )
        });
        let case = cases.get(&row.id).unwrap_or_else(|| {
            panic!(
                "manifest row {} names file {} but that file has no case with id {}",
                row.id, row.file, row.id
            )
        });
        let normalized =
            normalized_manifest_outcome(&row.file, &load_fixture_value(&row.file), case);
        assert_eq!(
            row.outcome, normalized,
            "manifest row {} in {} classifies case {} as {}, but its normalized fixture outcome is {}",
            row.id, row.file, row.id, row.outcome, normalized
        );
    }

    let disk_files: BTreeSet<_> = fs::read_dir(support::fixtures_dir())
        .expect("fixture directory exists")
        .map(|entry| {
            let entry = entry.expect("fixture directory entry");
            assert!(entry.file_type().expect("fixture entry type").is_file());
            entry.file_name().to_string_lossy().into_owned()
        })
        .filter(|file| file != "manifest.json")
        .filter(|file| !LEGACY_BOUNDARY_FIXTURES.contains(&file.as_str()))
        .collect();
    assert_eq!(
        disk_files, manifest_files,
        "every fixture file must be listed exactly once in the manifest"
    );

    let loaded_ids: BTreeSet<_> = file_cases
        .values()
        .flat_map(|cases| cases.keys().cloned())
        .collect();
    assert_eq!(loaded_ids, declared_ids);
}

#[test]
fn signing_success_fixture_round_trips_and_passes_the_real_facade() {
    let fixture: SigningSuccessFixture = load_fixture("signing_success.json");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "universal_chain_signer");
    assert_eq!(fixture.api_version, UNIVERSAL_CHAIN_SIGNER_API_VERSION);
    let case = fixture
        .cases
        .into_iter()
        .find(|case| case.id == "signing.success.bitcoin_message")
        .expect("signing success case");

    assert_json_round_trip(&case.request);
    assert_json_round_trip(&case.response);
    assert_json_round_trip(&case.verification_request);
    assert_json_round_trip(&case.verification_result);

    let signer = DeterministicSigner::new(SignerResponseMode::Valid);
    assert_eq!(
        signer.capabilities().api_version,
        UNIVERSAL_CHAIN_SIGNER_API_VERSION
    );

    let response = signer
        .sign(&case.request)
        .expect("fixture signing succeeds");
    assert_eq!(response, case.response);

    let derivation_request = AddressDerivationRequest::new(
        case.request.target.clone(),
        case.request.algorithm,
        case.request.derivation.clone(),
    );
    let derived = signer
        .derive_address(&derivation_request)
        .expect("fixture address derivation succeeds");
    assert_eq!(derived.address, case.response.address);
    assert_eq!(derived.verification_key, case.response.verification_key);

    let verification = signer
        .verify_signature(&case.verification_request)
        .expect("fixture verification request succeeds");
    assert_eq!(verification, case.verification_result);

    let encoded = serde_json::to_string(&response).expect("response serializes");
    let decoded: SignResponse = serde_json::from_str(&encoded).expect("response deserializes");
    assert_eq!(decoded, case.response);

    let request_encoded = serde_json::to_string(&case.request).expect("request serializes");
    let request_decoded: SignRequest =
        serde_json::from_str(&request_encoded).expect("request deserializes");
    assert_eq!(request_decoded, case.request);
}
