use chrono::{DateTime, Utc};
use lib_conxian_core::adapters::{
    BitcoinAdapter, CosmosAdapter, EvmAdapter, TxParams, UniversalChainAdapter,
};
use lib_conxian_core::control_model::{
    Bip110PreflightRequest, Bip110PreflightResult, Chain, ChainFamily, TrustTier,
    VerificationStatus, BIP110_PREFLIGHT_API_VERSION,
};
use lib_conxian_core::signing::{
    SignRequest, SignResponse, SignerCapabilities, SigningError, UniversalChainSigner,
    UNIVERSAL_CHAIN_SIGNER_API_VERSION,
};
use lib_conxian_core::verifier::{
    DynProtocolVerifier, LatestVerifiedBlock, ProofVerificationRequest, ProofVerificationResult,
    ProtocolVerifier, ProtocolVerifierBackend, ProtocolVerifierError, TransactionFinalityRequest,
    TransactionFinalityResult, VerifierCapabilities, PROTOCOL_VERIFIER_EVIDENCE_BINDING_DOMAIN,
    PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn fixture_contents(name: &str) -> &'static str {
    match name {
        "signing_boundary.json" => include_str!("fixtures/signing_boundary.json"),
        "verifier_boundary.json" => include_str!("fixtures/verifier_boundary.json"),
        "bip110_preflight.json" => include_str!("fixtures/bip110_preflight.json"),
        "adapter_contracts.json" => include_str!("fixtures/adapter_contracts.json"),
        unknown => panic!("unknown deterministic integration fixture: {unknown}"),
    }
}

fn fixture<T: DeserializeOwned>(name: &str) -> T {
    let contents = fixture_contents(name);

    serde_json::from_str(contents)
        .unwrap_or_else(|error| panic!("fixture {name} must deserialize: {error}"))
}

fn assert_semantic_fixture_round_trip<T: Serialize>(name: &str, fixture: &T) {
    let source: serde_json::Value = serde_json::from_str(fixture_contents(name))
        .unwrap_or_else(|error| panic!("fixture {name} must parse as JSON: {error}"));
    let encoded = serde_json::to_value(fixture)
        .unwrap_or_else(|error| panic!("fixture {name} must serialize as JSON: {error}"));
    assert_eq!(
        encoded, source,
        "fixture {name} changed during JSON round trip"
    );
}

#[derive(Debug, Deserialize)]
struct SigningFixture {
    api_version: u16,
    capabilities: SignerCapabilities,
    request: SignRequest,
    response: SignResponse,
    unsupported_cases: Vec<SigningNegativeCase>,
}

#[derive(Debug, Deserialize)]
struct SigningNegativeCase {
    name: String,
    request: SignRequest,
    expected_error: SigningError,
}

#[derive(Debug)]
struct DeterministicFixtureSigner {
    capabilities: SignerCapabilities,
    response: SignResponse,
}

impl UniversalChainSigner for DeterministicFixtureSigner {
    fn capabilities(&self) -> &SignerCapabilities {
        &self.capabilities
    }

    fn sign_impl(&self, _request: &SignRequest) -> Result<SignResponse, SigningError> {
        // This is a structural contract double. It returns synthetic public
        // metadata from the fixture and does not claim cryptographic signing.
        Ok(self.response.clone())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct VerifierFixture {
    fixture_scope: String,
    evidence_binding_version: u8,
    evidence_binding_domain: String,
    validation_time: DateTime<Utc>,
    capabilities: VerifierCapabilities,
    unsupported_capability_capabilities: VerifierCapabilities,
    state_request: ProofVerificationRequest,
    state_result: ProofVerificationResult,
    finality_request: TransactionFinalityRequest,
    finality_result: TransactionFinalityResult,
    malformed_proof: VerifierRequestErrorCase<ProofVerificationRequest>,
    unsupported_capability: VerifierRequestErrorCase<TransactionFinalityRequest>,
    stale_evidence: VerifierErrorCase,
    policy_rejection: PolicyRejectionCase,
}

#[derive(Debug, Deserialize, Serialize)]
struct VerifierRequestErrorCase<T> {
    request: T,
    expected_error: ProtocolVerifierError,
}

#[derive(Debug, Deserialize, Serialize)]
struct VerifierErrorCase {
    expected_error: ProtocolVerifierError,
}

#[derive(Debug, Deserialize, Serialize)]
struct PolicyRejectionCase {
    result: ProofVerificationResult,
    expected_error: ProtocolVerifierError,
}

fn unexpected_backend_call<T>() -> Result<T, ProtocolVerifierError> {
    Err(ProtocolVerifierError::UnsupportedVerifier {
        verifier_id: "integration-fixture".to_string(),
        reason: "backend hook was called after a structural rejection".to_string(),
    })
}

#[derive(Clone, Debug)]
struct SuccessfulVerifierDouble {
    capabilities: VerifierCapabilities,
    state_result: ProofVerificationResult,
    finality_result: TransactionFinalityResult,
    calls: Arc<AtomicUsize>,
}

impl SuccessfulVerifierDouble {
    fn new(
        capabilities: VerifierCapabilities,
        state_result: ProofVerificationResult,
        finality_result: TransactionFinalityResult,
    ) -> Self {
        Self {
            capabilities,
            state_result,
            finality_result,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for SuccessfulVerifierDouble {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.state_result.clone())
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.state_result.verified_block.clone())
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.finality_result.clone())
    }
}

#[derive(Clone, Debug)]
struct UnsupportedCapabilityVerifierDouble {
    capabilities: VerifierCapabilities,
    calls: Arc<AtomicUsize>,
}

impl UnsupportedCapabilityVerifierDouble {
    fn new(capabilities: VerifierCapabilities) -> Self {
        Self {
            capabilities,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for UnsupportedCapabilityVerifierDouble {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }
}

#[derive(Clone, Debug)]
struct MalformedProofVerifierDouble {
    capabilities: VerifierCapabilities,
    calls: Arc<AtomicUsize>,
}

impl MalformedProofVerifierDouble {
    fn new(capabilities: VerifierCapabilities) -> Self {
        Self {
            capabilities,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for MalformedProofVerifierDouble {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }
}

#[derive(Clone, Debug)]
struct StaleEvidenceVerifierDouble {
    capabilities: VerifierCapabilities,
    error: ProtocolVerifierError,
    calls: Arc<AtomicUsize>,
}

impl StaleEvidenceVerifierDouble {
    fn new(capabilities: VerifierCapabilities, error: ProtocolVerifierError) -> Self {
        Self {
            capabilities,
            error,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for StaleEvidenceVerifierDouble {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(self.error.clone())
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }
}

#[derive(Clone, Debug)]
struct PolicyRejectingVerifierDouble {
    capabilities: VerifierCapabilities,
    result: ProofVerificationResult,
    calls: Arc<AtomicUsize>,
}

impl PolicyRejectingVerifierDouble {
    fn new(capabilities: VerifierCapabilities, result: ProofVerificationResult) -> Self {
        Self {
            capabilities,
            result,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for PolicyRejectingVerifierDouble {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.result.clone())
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &lib_conxian_core::verifier::ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.result.verified_block.clone())
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        unexpected_backend_call()
    }
}

#[derive(Debug, Deserialize)]
struct Bip110Fixture {
    api_version: u16,
    success: Bip110Case,
    failure: Bip110Case,
    unsupported_version: Bip110Case,
}

#[derive(Debug, Deserialize)]
struct Bip110Case {
    request: Bip110PreflightRequest,
    expected: Bip110PreflightResult,
}

#[derive(Debug, Deserialize, Serialize)]
struct AdapterFixture {
    fixture_scope: String,
    schema_version: u16,
    verification_scope: String,
    contracts: Vec<AdapterContractFixture>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AdapterContractFixture {
    adapter: String,
    chain: Chain,
    family: ChainFamily,
    trust_tier: TrustTier,
    transaction: TxParams,
    expected_fee: u64,
}

fn adapter_for(name: &str, chain: Chain) -> Box<dyn UniversalChainAdapter> {
    match name {
        "bitcoin" => Box::new(BitcoinAdapter),
        "evm" => Box::new(EvmAdapter { chain }),
        "cosmos" => Box::new(CosmosAdapter { chain }),
        unknown => panic!("unknown adapter fixture: {unknown}"),
    }
}

#[test]
fn signing_fixture_round_trips_and_fail_closed_capabilities_are_deterministic() {
    let fixture: SigningFixture = fixture("signing_boundary.json");
    assert_eq!(fixture.api_version, UNIVERSAL_CHAIN_SIGNER_API_VERSION);
    assert_eq!(
        fixture.capabilities.api_version,
        UNIVERSAL_CHAIN_SIGNER_API_VERSION
    );

    let request_json = serde_json::to_string(&fixture.request).expect("request serializes");
    let decoded_request: SignRequest =
        serde_json::from_str(&request_json).expect("request deserializes");
    assert_eq!(decoded_request, fixture.request);
    assert_eq!(
        serde_json::to_string(&decoded_request).expect("round-tripped request serializes"),
        request_json
    );

    let signer = DeterministicFixtureSigner {
        capabilities: fixture.capabilities,
        response: fixture.response.clone(),
    };
    let first = signer
        .sign(&fixture.request)
        .expect("fixture sign succeeds");
    let second = signer
        .sign(&fixture.request)
        .expect("repeated fixture sign succeeds");
    assert_eq!(first, fixture.response);
    assert_eq!(first, second);

    for case in fixture.unsupported_cases {
        let error = signer
            .sign(&case.request)
            .expect_err("unsupported fixture request must fail closed");
        assert_eq!(error, case.expected_error, "case {}", case.name);
    }
}

#[test]
fn verifier_fixtures_cover_success_finality_and_structural_rejection_paths() {
    let fixture: VerifierFixture = fixture("verifier_boundary.json");
    assert_eq!(fixture.fixture_scope, "synthetic-structural-only");
    assert_semantic_fixture_round_trip("verifier_boundary.json", &fixture);
    assert_eq!(
        fixture.evidence_binding_version,
        PROTOCOL_VERIFIER_EVIDENCE_BINDING_VERSION
    );
    assert_eq!(
        fixture.evidence_binding_domain,
        String::from_utf8_lossy(PROTOCOL_VERIFIER_EVIDENCE_BINDING_DOMAIN)
    );
    assert_eq!(fixture.capabilities.version, "1");
    fixture
        .capabilities
        .validate()
        .expect("fixture capabilities are internally valid");

    let state_request_json =
        serde_json::to_string(&fixture.state_request).expect("state request serializes");
    let decoded_state_request: ProofVerificationRequest =
        serde_json::from_str(&state_request_json).expect("state request deserializes");
    assert_eq!(
        serde_json::to_value(&decoded_state_request).expect("decoded state request is JSON"),
        serde_json::to_value(&fixture.state_request).expect("fixture state request is JSON")
    );

    let finality_request_json =
        serde_json::to_string(&fixture.finality_request).expect("finality request serializes");
    let decoded_finality_request: TransactionFinalityRequest =
        serde_json::from_str(&finality_request_json).expect("finality request deserializes");
    assert_eq!(decoded_finality_request, fixture.finality_request);

    let backend = SuccessfulVerifierDouble::new(
        fixture.capabilities.clone(),
        fixture.state_result.clone(),
        fixture.finality_result.clone(),
    );
    let verifier = ProtocolVerifier::try_new(backend.clone()).expect("valid verifier fixture");
    let state = verifier
        .verify_chain_state_at(&fixture.state_request, fixture.validation_time)
        .expect("fixture state proof succeeds structurally");
    assert_eq!(state, fixture.state_result);
    assert_eq!(
        state.verified_block.verification_status,
        VerificationStatus::Verified
    );
    let finality = verifier
        .verify_transaction_finality_at(&fixture.finality_request, fixture.validation_time)
        .expect("fixture finality succeeds structurally");
    assert_eq!(finality, fixture.finality_result);
    assert_eq!(finality.verification_status, VerificationStatus::Verified);
    assert!(finality.is_final());
    assert_eq!(backend.calls(), 2);

    let encoded_result = serde_json::to_string(&state).expect("state result serializes");
    let decoded_result: ProofVerificationResult =
        serde_json::from_str(&encoded_result).expect("state result deserializes");
    assert_eq!(decoded_result, state);

    let malformed_backend = MalformedProofVerifierDouble::new(fixture.capabilities.clone());
    let malformed_verifier = ProtocolVerifier::try_new(malformed_backend.clone())
        .expect("malformed-proof fixture backend advertisement is valid");
    let malformed_error = malformed_verifier
        .verify_chain_state_at(&fixture.malformed_proof.request, fixture.validation_time)
        .expect_err("empty proof bytes must fail closed before backend access");
    assert_eq!(malformed_error, fixture.malformed_proof.expected_error);
    assert_eq!(malformed_backend.calls(), 0);

    let unsupported_backend = UnsupportedCapabilityVerifierDouble::new(
        fixture.unsupported_capability_capabilities.clone(),
    );
    let unsupported_verifier = ProtocolVerifier::try_new(unsupported_backend.clone())
        .expect("unsupported-capability fixture advertisement is valid");
    let unsupported_error = unsupported_verifier
        .verify_transaction_finality_at(
            &fixture.unsupported_capability.request,
            fixture.validation_time,
        )
        .expect_err("unadvertised finality must fail closed before backend access");
    assert_eq!(
        unsupported_error,
        fixture.unsupported_capability.expected_error
    );
    assert_eq!(unsupported_backend.calls(), 0);

    let stale_backend = StaleEvidenceVerifierDouble::new(
        fixture.capabilities.clone(),
        fixture.stale_evidence.expected_error.clone(),
    );
    let stale_verifier = ProtocolVerifier::try_new(stale_backend.clone())
        .expect("stale-evidence fixture advertisement is valid");
    let stale_error = stale_verifier
        .verify_chain_state_at(&fixture.state_request, fixture.validation_time)
        .expect_err("stale evidence double must return its typed error");
    assert_eq!(stale_error, fixture.stale_evidence.expected_error);
    assert_eq!(stale_backend.calls(), 1);

    let policy_backend = PolicyRejectingVerifierDouble::new(
        fixture.capabilities.clone(),
        fixture.policy_rejection.result.clone(),
    );
    let policy_verifier = ProtocolVerifier::try_new(policy_backend.clone())
        .expect("policy-rejection fixture advertisement is valid");
    let policy_error = policy_verifier
        .verify_chain_state_at(&fixture.state_request, fixture.validation_time)
        .expect_err("degraded evidence must be policy blocked");
    assert_eq!(policy_error, fixture.policy_rejection.expected_error);
    assert_eq!(policy_backend.calls(), 1);

    let dynamic_backend = SuccessfulVerifierDouble::new(
        fixture.capabilities,
        fixture.state_result,
        fixture.finality_result,
    );
    let dynamic: DynProtocolVerifier = ProtocolVerifier::new(Box::new(dynamic_backend));
    assert!(dynamic
        .verify_chain_state_at(&fixture.state_request, fixture.validation_time)
        .is_ok());
}

#[test]
fn bip110_fixture_covers_success_failure_and_version_compatibility() {
    let fixture: Bip110Fixture = fixture("bip110_preflight.json");
    assert_eq!(fixture.api_version, BIP110_PREFLIGHT_API_VERSION);

    for case in [
        &fixture.success,
        &fixture.failure,
        &fixture.unsupported_version,
    ] {
        let actual = case.request.validate();
        assert_eq!(actual, case.expected);

        let request_json = serde_json::to_string(&case.request).expect("preflight serializes");
        let decoded_request: Bip110PreflightRequest =
            serde_json::from_str(&request_json).expect("preflight deserializes");
        assert_eq!(decoded_request, case.request);

        let result_json = serde_json::to_string(&actual).expect("preflight result serializes");
        let decoded_result: Bip110PreflightResult =
            serde_json::from_str(&result_json).expect("preflight result deserializes");
        assert_eq!(decoded_result, actual);
    }

    assert!(fixture.success.expected.is_compliant);
    assert!(!fixture.failure.expected.is_compliant);
    assert_eq!(
        fixture
            .failure
            .expected
            .violations()
            .map(|violation| violation.code())
            .collect::<Vec<_>>(),
        vec![
            "pushdata_exceeds_limit",
            "op_return_exceeds_limit",
            "script_pubkey_exceeds_limit",
            "witness_element_exceeds_limit",
            "taproot_control_block_exceeds_limit",
        ]
    );
    assert!(!fixture.unsupported_version.expected.is_compliant);
    assert_eq!(
        fixture
            .unsupported_version
            .expected
            .errors()
            .next()
            .unwrap()
            .code(),
        "unsupported_api_version"
    );
}

#[test]
fn adapter_fixtures_validate_chain_metadata_and_request_shapes_only() {
    let fixture: AdapterFixture = fixture("adapter_contracts.json");
    assert_eq!(fixture.fixture_scope, "synthetic-structural-only");
    assert_semantic_fixture_round_trip("adapter_contracts.json", &fixture);
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.verification_scope, "structural_contract_only");

    for contract in fixture.contracts {
        let adapter = adapter_for(&contract.adapter, contract.chain.clone());
        assert_eq!(adapter.chain(), contract.chain);
        assert_eq!(adapter.family(), contract.family);
        assert_eq!(adapter.trust_tier(), contract.trust_tier);
        assert!(adapter
            .validate_address(&contract.transaction.destination)
            .is_ok());
        assert_eq!(
            adapter.estimate_fee(&contract.transaction).unwrap(),
            contract.expected_fee
        );
        assert_eq!(contract.chain.family(), contract.family);

        let transaction_json =
            serde_json::to_string(&contract.transaction).expect("adapter request serializes");
        let decoded_transaction: TxParams =
            serde_json::from_str(&transaction_json).expect("adapter request deserializes");
        assert_eq!(
            decoded_transaction.amount_sats,
            contract.transaction.amount_sats
        );
        assert_eq!(
            decoded_transaction.destination,
            contract.transaction.destination
        );
        assert_eq!(decoded_transaction.data, contract.transaction.data);
    }

    // Adapter `verify_state_proof` methods are intentionally not exercised here:
    // this fixture layer does not claim authoritative cryptographic verification.
}
