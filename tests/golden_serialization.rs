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

#[test]
fn manifest_and_every_golden_file_are_versioned_and_accounted_for() {
    let manifest = load_manifest();

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.package_name, "lib-conxian-core");
    assert_eq!(manifest.package_version, "0.2.12");
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
    let mut file_case_ids = BTreeMap::new();
    for file in files {
        let value = load_fixture_value(&file);
        assert_eq!(value["schema_version"], Value::from(1));
        let cases = value["cases"]
            .as_array()
            .unwrap_or_else(|| panic!("fixture {file} is missing a cases array"));
        let mut ids = BTreeSet::new();
        for case in cases {
            let id = case["id"]
                .as_str()
                .unwrap_or_else(|| panic!("fixture {file} contains a case without an id"));
            assert!(
                ids.insert(id.to_string()),
                "duplicate case id {id} in {file}"
            );
        }
        file_case_ids.insert(file, ids);
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

    let loaded_ids: BTreeSet<_> = file_case_ids
        .values()
        .flat_map(|ids| ids.iter().cloned())
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
