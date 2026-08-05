use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use conxius_enclave_sdk::{
    config::Network as SdkNetwork,
    enclave::{
        attestation::{
            AttestationLevel as SdkAttestationLevel, AttestationReportType, DeviceIntegrityReport,
        },
        EnclaveManager, SignRequest as SdkSignRequest, SignResponse as SdkSignResponse,
        SigningAlgorithm as SdkSigningAlgorithm,
    },
    protocol::rails::TrustTier as SdkRailTrustTier,
    ConclaveError, ConclaveResult,
};
use lib_conxian_core::{
    control_model::{
        Bip110PreflightMeasurements, Bip110PreflightPhase, Bip110PreflightRequest, Chain,
        SignedEnvelopeDescriptor, TrustTier,
    },
    signing::{
        DerivationContext, DerivationIndex, DerivationPath, DerivationPurpose, DigestAlgorithm,
        SignRequest, SignatureEncoding, SigningAlgorithm, SigningPayload, SigningTarget,
    },
};
use lib_conxian_core_enclave::{
    extract_sdk_digest, from_sdk_algorithm, render_derivation_path, replay_binding_digest,
    to_sdk_algorithm, AdapterError, AttestationLevel, EnclaveSdkAdapter, MinimumAttestation,
    NetworkPolicy, ProviderResponseField, RailTrustPolicy, RailTrustTier, ReplayBinding,
    RequestPolicyContext, TrustPolicy,
};

const TEST_DIGEST: [u8; 32] = [7; 32];

fn test_envelope() -> SignedEnvelopeDescriptor {
    SignedEnvelopeDescriptor {
        event_id: "event-1".to_owned(),
        sequence: 7,
        publisher: "publisher-1".to_owned(),
        payload_hash: "sha256:test".to_owned(),
        commitments: vec!["commitment-1".to_owned()],
    }
}

fn test_policy_context() -> RequestPolicyContext {
    policy_context_for_tier(TrustTier::Managed)
}

fn policy_context_for_tier(tier: TrustTier) -> RequestPolicyContext {
    let observed = match tier {
        TrustTier::Strict => RailTrustTier::T1,
        TrustTier::Managed => RailTrustTier::T2,
        TrustTier::Expedient => RailTrustTier::T3,
        TrustTier::ObserverOnly => RailTrustTier::T4,
    };
    RequestPolicyContext::new(
        NetworkPolicy::Testnet,
        RailTrustPolicy::new(tier, observed).unwrap(),
    )
    .unwrap()
}

fn policy_context_for_adapter(adapter: &EnclaveSdkAdapter) -> RequestPolicyContext {
    policy_context_for_tier(adapter.trust_policy().tier().clone())
}

fn bound_test_digest() -> Vec<u8> {
    replay_binding_digest(&test_envelope(), &TEST_DIGEST, &test_policy_context()).unwrap()
}

#[derive(Clone)]
enum Outcome {
    Response(SdkSignResponse),
    Failure,
}

struct RecordingManager {
    calls: AtomicUsize,
    public_key_calls: AtomicUsize,
    outcome: Mutex<Outcome>,
    last_request: Mutex<Option<SdkSignRequest>>,
}

impl RecordingManager {
    fn new(response: SdkSignResponse) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            public_key_calls: AtomicUsize::new(0),
            outcome: Mutex::new(Outcome::Response(response)),
            last_request: Mutex::new(None),
        }
    }

    fn failing() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            public_key_calls: AtomicUsize::new(0),
            outcome: Mutex::new(Outcome::Failure),
            last_request: Mutex::new(None),
        }
    }

    fn set_response(&self, response: SdkSignResponse) {
        *self.outcome.lock().unwrap() = Outcome::Response(response);
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn public_key_calls(&self) -> usize {
        self.public_key_calls.load(Ordering::SeqCst)
    }

    fn last_request(&self) -> SdkSignRequest {
        self.last_request.lock().unwrap().clone().unwrap()
    }
}

impl EnclaveManager for RecordingManager {
    fn initialize(&self) -> ConclaveResult<()> {
        Ok(())
    }

    fn generate_key(&self, _key_id: &str) -> ConclaveResult<String> {
        Ok("public-key".to_owned())
    }

    fn get_public_key(&self, _derivation_path: &str) -> ConclaveResult<String> {
        self.public_key_calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("02{}", "11".repeat(32)))
    }

    fn sign(&self, request: SdkSignRequest) -> ConclaveResult<SdkSignResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        *self.last_request.lock().unwrap() = Some(request);
        match &*self.outcome.lock().unwrap() {
            Outcome::Response(response) => Ok(response.clone()),
            Outcome::Failure => Err(ConclaveError::EnclaveFailure(
                "sensitive-provider-detail".to_owned(),
            )),
        }
    }
}

fn attestation(level: SdkAttestationLevel) -> String {
    attestation_with_nonce(level, bound_test_digest())
}

fn attestation_with_nonce(level: SdkAttestationLevel, challenge_nonce: Vec<u8>) -> String {
    serde_json::to_string(&DeviceIntegrityReport {
        report_version: 1,
        report_type: AttestationReportType::DeviceIntegrity,
        level,
        challenge_nonce,
        signature: vec![4, 5, 6],
        attested_operation_public_key: vec![7, 8, 9],
        signer_key_binding: None,
        certificate_chain: vec!["device".to_owned(), "root".to_owned()],
        timestamp: 1,
        extension_data: "deterministic-test-double".to_owned(),
        extensions: vec![],
    })
    .unwrap()
}

fn response(
    signature_bytes: usize,
    public_key_bytes: usize,
    level: SdkAttestationLevel,
) -> SdkSignResponse {
    response_with_nonce(
        signature_bytes,
        public_key_bytes,
        level,
        bound_test_digest(),
    )
}

fn response_with_nonce(
    signature_bytes: usize,
    public_key_bytes: usize,
    level: SdkAttestationLevel,
    challenge_nonce: Vec<u8>,
) -> SdkSignResponse {
    SdkSignResponse {
        signature_hex: hex::encode(vec![0x11; signature_bytes]),
        public_key_hex: hex::encode(vec![0x22; public_key_bytes]),
        device_attestation: Some(attestation_with_nonce(level, challenge_nonce)),
    }
}

fn context() -> DerivationContext {
    DerivationContext::new(
        DerivationPath::new(vec![
            DerivationIndex::new(44, true),
            DerivationIndex::new(u32::MAX, false),
        ]),
        DerivationPurpose::MessageSigning,
    )
}

fn request(chain: Chain, algorithm: SigningAlgorithm, payload: SigningPayload) -> SignRequest {
    SignRequest::new(
        SigningTarget::for_chain(chain),
        algorithm,
        payload,
        context(),
    )
}

fn digest_request(chain: Chain, algorithm: SigningAlgorithm) -> SignRequest {
    request(
        chain,
        algorithm,
        SigningPayload::digest(DigestAlgorithm::Sha256, TEST_DIGEST.to_vec()),
    )
}

fn compliant_bip110_preflight() -> Bip110PreflightRequest {
    Bip110PreflightRequest::new(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::BitcoinTransaction,
        Bip110PreflightMeasurements::new(vec![256], vec![83], vec![34], vec![256]),
    )
}

fn adapter(manager: Arc<RecordingManager>, tier: TrustTier) -> EnclaveSdkAdapter {
    EnclaveSdkAdapter::new(manager, "test-key", tier).unwrap()
}

fn build_sdk_request(
    adapter: &EnclaveSdkAdapter,
    request: &SignRequest,
) -> Result<SdkSignRequest, AdapterError> {
    let policy_context = policy_context_for_adapter(adapter);
    adapter.build_sdk_sign_request(request, &test_envelope(), &policy_context)
}

fn sign_digest(
    adapter: &EnclaveSdkAdapter,
    request: &SignRequest,
) -> Result<lib_conxian_core_enclave::EnclaveSignResponse, AdapterError> {
    let policy_context = policy_context_for_adapter(adapter);
    adapter.sign_digest(request, &test_envelope(), &policy_context)
}

fn sign_digest_with_preflight(
    adapter: &EnclaveSdkAdapter,
    request: &SignRequest,
    preflight: &Bip110PreflightRequest,
) -> Result<lib_conxian_core_enclave::EnclaveSignResponse, AdapterError> {
    let policy_context = policy_context_for_adapter(adapter);
    adapter.sign_digest_with_bip110_preflight(request, preflight, &test_envelope(), &policy_context)
}

#[test]
fn sdk_and_core_algorithm_mappings_are_explicit_and_round_trip() {
    for (core, sdk) in [
        (
            SigningAlgorithm::EcdsaSecp256k1,
            SdkSigningAlgorithm::EcdsaSecp256k1,
        ),
        (
            SigningAlgorithm::SchnorrSecp256k1,
            SdkSigningAlgorithm::SchnorrSecp256k1,
        ),
        (SigningAlgorithm::Ed25519, SdkSigningAlgorithm::Ed25519),
    ] {
        assert_eq!(to_sdk_algorithm(core), sdk);
        assert_eq!(from_sdk_algorithm(sdk), core);
    }
}

#[test]
fn derivation_paths_render_deterministically_at_boundaries() {
    let root = DerivationContext::new(DerivationPath::root(), DerivationPurpose::Payment);
    assert_eq!(render_derivation_path(&root).unwrap(), "m");
    assert_eq!(
        render_derivation_path(&context()).unwrap(),
        "m/44'/4294967295"
    );

    let maximum = DerivationContext::new(
        DerivationPath::new(vec![DerivationIndex::new(0, false); 255]),
        DerivationPurpose::Payment,
    );
    assert!(render_derivation_path(&maximum).is_ok());

    let too_deep = DerivationContext::new(
        DerivationPath::new(vec![DerivationIndex::new(0, false); 256]),
        DerivationPurpose::Payment,
    );
    assert!(matches!(
        render_derivation_path(&too_deep),
        Err(AdapterError::InvalidDerivationPath(_))
    ));
}

#[test]
fn digest_extraction_preserves_supported_bytes_and_rejects_ambiguous_payloads() {
    let bytes: Vec<u8> = (0..32).collect();
    assert_eq!(
        extract_sdk_digest(&SigningPayload::digest(
            DigestAlgorithm::Sha256,
            bytes.clone(),
        ))
        .unwrap(),
        bytes
    );
    assert!(matches!(
        extract_sdk_digest(&SigningPayload::message(b"raw message".to_vec())),
        Err(AdapterError::MessagePayloadRejected)
    ));
    assert!(matches!(
        extract_sdk_digest(&SigningPayload::digest(
            DigestAlgorithm::Sha512,
            vec![0; 64]
        )),
        Err(AdapterError::UnsupportedDigestAlgorithm(
            DigestAlgorithm::Sha512
        ))
    ));
    assert!(matches!(
        extract_sdk_digest(&SigningPayload::digest(
            DigestAlgorithm::Keccak256,
            vec![0; 32]
        )),
        Err(AdapterError::UnsupportedDigestAlgorithm(
            DigestAlgorithm::Keccak256
        ))
    ));
    assert!(matches!(
        extract_sdk_digest(&SigningPayload::digest(
            DigestAlgorithm::Blake2b256,
            vec![0; 32]
        )),
        Err(AdapterError::UnsupportedDigestAlgorithm(
            DigestAlgorithm::Blake2b256
        ))
    ));
    assert!(matches!(
        extract_sdk_digest(&SigningPayload::digest(
            DigestAlgorithm::Sha256,
            vec![0; 31]
        )),
        Err(AdapterError::InvalidDigestLength {
            expected: 32,
            actual: 31
        })
    ));
}

#[test]
fn request_builder_keeps_digest_bytes_and_derivation_metadata_separate() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1);

    let sdk_request = build_sdk_request(&adapter, &request).unwrap();
    assert_eq!(sdk_request.algorithm, SdkSigningAlgorithm::EcdsaSecp256k1);
    assert_eq!(sdk_request.message_hash, bound_test_digest());
    assert_eq!(sdk_request.derivation_path, "m/44'/4294967295");
    assert_eq!(sdk_request.key_id, "test-key");
    assert_eq!(sdk_request.taproot_tweak, None);
    assert_eq!(manager.calls(), 0);
}

#[test]
fn response_mapping_supports_exact_sdk_signature_shapes() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let compact = sign_digest(
        &adapter,
        &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
    )
    .unwrap();
    assert_eq!(compact.signature.encoding, SignatureEncoding::Compact);
    assert_eq!(compact.signature.bytes.len(), 64);
    assert_eq!(compact.verification_key.bytes.len(), 33);
    assert_eq!(
        compact.attestation.evidence.challenge_nonce,
        bound_test_digest()
    );
    assert_eq!(
        compact.attestation.evidence.raw_report,
        attestation(SdkAttestationLevel::TEE)
    );
    assert_eq!(compact.attestation.evidence.signature, vec![4, 5, 6]);
    assert_eq!(
        compact.attestation.evidence.certificate_chain,
        vec!["device".to_owned(), "root".to_owned()]
    );
    let forwarded = manager.last_request();
    assert_eq!(forwarded.algorithm, SdkSigningAlgorithm::EcdsaSecp256k1);
    assert_eq!(forwarded.message_hash, bound_test_digest());
    assert_eq!(forwarded.derivation_path, "m/44'/4294967295");
    assert_eq!(forwarded.key_id, "test-key");

    manager.set_response(response(65, 65, SdkAttestationLevel::TEE));
    let recoverable = sign_digest(
        &adapter,
        &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
    )
    .unwrap();
    assert_eq!(
        recoverable.signature.encoding,
        SignatureEncoding::Recoverable
    );
    assert_eq!(recoverable.signature.bytes.len(), 65);
    assert_eq!(recoverable.verification_key.bytes.len(), 65);

    let schnorr_manager = Arc::new(RecordingManager::new(response(
        64,
        32,
        SdkAttestationLevel::TEE,
    )));
    let schnorr_adapter =
        EnclaveSdkAdapter::new(schnorr_manager, "test-key", TrustTier::Managed).unwrap();
    let schnorr = sign_digest_with_preflight(
        &schnorr_adapter,
        &digest_request(Chain::Bitcoin, SigningAlgorithm::SchnorrSecp256k1),
        &compliant_bip110_preflight(),
    )
    .unwrap();
    assert_eq!(schnorr.signature.encoding, SignatureEncoding::Compact);
    assert_eq!(schnorr.verification_key.bytes.len(), 32);

    let malformed_schnorr_manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let malformed_schnorr_adapter = EnclaveSdkAdapter::new(
        malformed_schnorr_manager.clone(),
        "test-key",
        TrustTier::Managed,
    )
    .unwrap();
    assert_eq!(
        sign_digest_with_preflight(
            &malformed_schnorr_adapter,
            &digest_request(Chain::Bitcoin, SigningAlgorithm::SchnorrSecp256k1),
            &compliant_bip110_preflight(),
        )
        .unwrap_err(),
        AdapterError::MalformedProviderResponse(ProviderResponseField::VerificationKey)
    );
    assert_eq!(malformed_schnorr_manager.calls(), 1);

    let ed_manager = Arc::new(RecordingManager::new(response(
        64,
        32,
        SdkAttestationLevel::TEE,
    )));
    let ed_adapter = EnclaveSdkAdapter::new(ed_manager, "test-key", TrustTier::Managed).unwrap();
    let ed25519 = sign_digest(
        &ed_adapter,
        &digest_request(Chain::Solana, SigningAlgorithm::Ed25519),
    )
    .unwrap();
    assert_eq!(ed25519.signature.encoding, SignatureEncoding::Raw);
    assert_eq!(ed25519.verification_key.bytes.len(), 32);
}

#[test]
fn attestation_nonce_mismatch_rejects_the_provider_response() {
    let manager = Arc::new(RecordingManager::new(response_with_nonce(
        64,
        33,
        SdkAttestationLevel::TEE,
        vec![8; 32],
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);

    assert_eq!(
        sign_digest(
            &adapter,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        )
        .unwrap_err(),
        AdapterError::AttestationChallengeMismatch
    );
    assert_eq!(manager.calls(), 1);
}

#[test]
fn schnorr_public_key_derivation_fails_closed_before_getter_invocation() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::ObserverOnly);

    assert_eq!(
        adapter
            .derive_public_key(
                &SigningTarget::for_chain(Chain::Bitcoin),
                SigningAlgorithm::SchnorrSecp256k1,
                &context(),
            )
            .unwrap_err(),
        AdapterError::UnsupportedPublicKeyDerivation(SigningAlgorithm::SchnorrSecp256k1)
    );
    assert_eq!(manager.public_key_calls(), 0);
}

#[test]
fn ed25519_public_key_derivation_fails_closed_before_getter_invocation() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        32,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);

    assert_eq!(
        adapter
            .derive_public_key(
                &SigningTarget::for_chain(Chain::Solana),
                SigningAlgorithm::Ed25519,
                &context(),
            )
            .unwrap_err(),
        AdapterError::UnsupportedPublicKeyDerivation(SigningAlgorithm::Ed25519)
    );
    assert_eq!(manager.public_key_calls(), 0);
}

#[test]
fn invalid_chain_algorithm_pairs_are_rejected_before_provider_calls() {
    for (chain, algorithm) in [
        (Chain::Solana, SigningAlgorithm::EcdsaSecp256k1),
        (Chain::Ethereum, SigningAlgorithm::Ed25519),
        (Chain::Bitcoin, SigningAlgorithm::Ed25519),
    ] {
        let manager = Arc::new(RecordingManager::new(response(
            64,
            33,
            SdkAttestationLevel::TEE,
        )));
        let adapter = adapter(manager.clone(), TrustTier::Managed);
        assert!(matches!(
            sign_digest(&adapter, &digest_request(chain.clone(), algorithm)),
            Err(AdapterError::UnsupportedChainAlgorithm { .. })
        ));
        assert_eq!(manager.calls(), 0);
    }
}

#[test]
fn capability_allowlist_accepts_only_explicit_safe_mappings() {
    for (chain, algorithm) in [
        (Chain::Bitcoin, SigningAlgorithm::EcdsaSecp256k1),
        (Chain::Bitcoin, SigningAlgorithm::SchnorrSecp256k1),
        (Chain::Stacks, SigningAlgorithm::EcdsaSecp256k1),
        (Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        (Chain::Solana, SigningAlgorithm::Ed25519),
    ] {
        let manager = Arc::new(RecordingManager::new(response(
            64,
            33,
            SdkAttestationLevel::TEE,
        )));
        let adapter = adapter(manager, TrustTier::Managed);
        assert!(build_sdk_request(&adapter, &digest_request(chain, algorithm)).is_ok());
    }
}

#[test]
fn schnorr_signing_requires_an_x_only_verification_key() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        65,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);

    assert_eq!(
        sign_digest_with_preflight(
            &adapter,
            &digest_request(Chain::Bitcoin, SigningAlgorithm::SchnorrSecp256k1),
            &compliant_bip110_preflight(),
        )
        .unwrap_err(),
        AdapterError::MalformedProviderResponse(ProviderResponseField::VerificationKey)
    );
    assert_eq!(manager.calls(), 1);
}

#[test]
fn malformed_provider_responses_are_typed_and_secret_safe() {
    let cases = [
        (
            SdkSignResponse {
                signature_hex: "not-hex".to_owned(),
                ..response(64, 33, SdkAttestationLevel::TEE)
            },
            AdapterError::MalformedProviderResponse(ProviderResponseField::Signature),
        ),
        (
            response(64, 1, SdkAttestationLevel::TEE),
            AdapterError::MalformedProviderResponse(ProviderResponseField::VerificationKey),
        ),
        (
            response(63, 33, SdkAttestationLevel::TEE),
            AdapterError::MalformedProviderResponse(ProviderResponseField::Signature),
        ),
        (
            SdkSignResponse {
                device_attestation: Some("not-json".to_owned()),
                ..response(64, 33, SdkAttestationLevel::TEE)
            },
            AdapterError::InvalidAttestation,
        ),
        (
            SdkSignResponse {
                device_attestation: None,
                ..response(64, 33, SdkAttestationLevel::TEE)
            },
            AdapterError::MissingAttestation,
        ),
    ];

    for (provider_response, expected) in cases {
        let manager = Arc::new(RecordingManager::new(provider_response));
        let adapter = adapter(manager, TrustTier::Managed);
        let error = sign_digest(
            &adapter,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        )
        .unwrap_err();
        assert_eq!(error, expected);
        assert!(!error.to_string().contains("not-hex"));
        assert!(!error.to_string().contains("not-json"));
    }
}

#[test]
fn trust_policy_is_conservative_and_observer_only_never_invokes_provider() {
    assert_eq!(
        TrustPolicy::for_tier(TrustTier::Strict).minimum_attestation(),
        Some(MinimumAttestation::HardwareBacked)
    );
    assert_eq!(
        TrustPolicy::for_tier(TrustTier::Managed).minimum_attestation(),
        Some(MinimumAttestation::Tee)
    );
    assert_eq!(
        TrustPolicy::for_tier(TrustTier::Expedient).minimum_attestation(),
        Some(MinimumAttestation::Tee)
    );

    let observer_manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let observer = adapter(observer_manager.clone(), TrustTier::ObserverOnly);
    assert_eq!(observer.trust_policy().minimum_attestation(), None);
    assert!(matches!(
        sign_digest(
            &observer,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        ),
        Err(AdapterError::ObserverOnlyCannotSign)
    ));
    assert_eq!(observer_manager.calls(), 0);

    let strict_context = policy_context_for_tier(TrustTier::Strict);
    let strict_bound_digest =
        replay_binding_digest(&test_envelope(), &TEST_DIGEST, &strict_context).unwrap();
    let strict_tee_manager = Arc::new(RecordingManager::new(response_with_nonce(
        64,
        33,
        SdkAttestationLevel::TEE,
        strict_bound_digest.clone(),
    )));
    let strict_tee = adapter(strict_tee_manager.clone(), TrustTier::Strict);
    assert!(matches!(
        sign_digest(
            &strict_tee,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        ),
        Err(AdapterError::InsufficientAttestation {
            required: Some(_),
            observed: AttestationLevel::Tee
        })
    ));
    assert_eq!(strict_tee_manager.calls(), 1);

    let strict_hardware = Arc::new(RecordingManager::new(response_with_nonce(
        64,
        33,
        SdkAttestationLevel::StrongBox,
        strict_bound_digest,
    )));
    let strict = adapter(strict_hardware.clone(), TrustTier::Strict);
    assert_eq!(
        sign_digest(
            &strict,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        )
        .unwrap()
        .attestation
        .level,
        AttestationLevel::StrongBox
    );

    let software_manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::Software,
    )));
    let managed = adapter(software_manager.clone(), TrustTier::Managed);
    assert!(matches!(
        sign_digest(
            &managed,
            &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
        ),
        Err(AdapterError::InsufficientAttestation {
            observed: AttestationLevel::Software,
            ..
        })
    ));
}

#[test]
fn bip110_preflight_rejection_happens_before_provider_invocation() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Bitcoin, SigningAlgorithm::EcdsaSecp256k1);
    let preflight = Bip110PreflightRequest::new(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::BitcoinTransaction,
        Bip110PreflightMeasurements::new(vec![257], vec![], vec![], vec![]),
    );

    assert!(matches!(
        sign_digest_with_preflight(&adapter, &request, &preflight),
        Err(AdapterError::PreflightRejected { code }) if code == "pushdata_exceeds_limit"
    ));
    assert_eq!(manager.calls(), 0);
}

#[test]
fn malformed_or_unsupported_bip110_preflight_also_blocks_provider() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Bitcoin, SigningAlgorithm::EcdsaSecp256k1);

    let missing = Bip110PreflightRequest::without_measurements(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::BitcoinTransaction,
    );
    assert!(matches!(
        sign_digest_with_preflight(&adapter, &request, &missing),
        Err(AdapterError::PreflightRejected { code }) if code == "missing_measurement_data"
    ));
    assert_eq!(manager.calls(), 0);

    let unsupported = Bip110PreflightRequest::new(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::Taproot,
        Bip110PreflightMeasurements::new(vec![], vec![], vec![], vec![]),
    );
    assert!(matches!(
        sign_digest_with_preflight(&adapter, &request, &unsupported),
        Err(AdapterError::PreflightRejected { code }) if code == "unsupported_context"
    ));
    assert_eq!(manager.calls(), 0);
}

#[test]
fn compliant_bip110_preflight_allows_provider_invocation_at_inclusive_limits() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Bitcoin, SigningAlgorithm::EcdsaSecp256k1);
    let preflight = Bip110PreflightRequest::new(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::BitcoinTransaction,
        Bip110PreflightMeasurements::new(vec![256], vec![83], vec![34], vec![256]),
    );

    let result = sign_digest_with_preflight(&adapter, &request, &preflight).unwrap();
    assert!(result.signature.bytes.iter().all(|byte| *byte == 0x11));
    assert_eq!(manager.calls(), 1);
}

#[test]
fn bitcoin_signing_requires_preflight_and_non_bitcoin_signing_does_not() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);

    assert!(matches!(
        sign_digest(
            &adapter,
            &digest_request(Chain::Bitcoin, SigningAlgorithm::EcdsaSecp256k1),
        ),
        Err(AdapterError::PreflightRequired)
    ));
    assert_eq!(manager.calls(), 0);

    sign_digest(
        &adapter,
        &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
    )
    .unwrap();
    assert_eq!(manager.calls(), 1);
}

#[test]
fn provider_failures_are_collapsed_to_secret_safe_errors() {
    let manager = Arc::new(RecordingManager::failing());
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let error = sign_digest(
        &adapter,
        &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
    )
    .unwrap_err();
    assert_eq!(error, AdapterError::ProviderFailure);
    assert!(!error.to_string().contains("sensitive-provider-detail"));
    assert_eq!(manager.calls(), 1);
}

#[test]
fn public_key_derivation_uses_the_injected_manager_without_secret_material() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager, TrustTier::ObserverOnly);
    let key = adapter
        .derive_public_key(
            &SigningTarget::for_chain(Chain::Ethereum),
            SigningAlgorithm::EcdsaSecp256k1,
            &context(),
        )
        .unwrap();
    assert_eq!(key.algorithm, SigningAlgorithm::EcdsaSecp256k1);
    assert_eq!(key.bytes.len(), 33);
    assert_eq!(key.bytes[0], 0x02);
}

#[test]
fn deserialized_trust_policy_cannot_weaken_the_core_attestation_floor() {
    let weakened: TrustPolicy =
        serde_json::from_str(r#"{"tier":"strict","minimum_attestation":"tee"}"#).unwrap();
    assert_eq!(
        weakened.validate(),
        Err(AdapterError::InvalidTrustPolicy {
            tier: TrustTier::Strict,
            required: Some(MinimumAttestation::HardwareBacked),
            configured: Some(MinimumAttestation::Tee),
        })
    );

    let manager = Arc::new(RecordingManager::failing());
    assert!(matches!(
        EnclaveSdkAdapter::with_policy(manager, "test-key", weakened),
        Err(AdapterError::InvalidTrustPolicy {
            tier: TrustTier::Strict,
            required: Some(MinimumAttestation::HardwareBacked),
            configured: Some(MinimumAttestation::Tee),
        })
    ));

    let observer_with_signing_floor: TrustPolicy =
        serde_json::from_str(r#"{"tier":"observer_only","minimum_attestation":"tee"}"#).unwrap();
    assert!(matches!(
        observer_with_signing_floor.validate(),
        Err(AdapterError::InvalidTrustPolicy {
            tier: TrustTier::ObserverOnly,
            required: None,
            configured: Some(MinimumAttestation::Tee),
        })
    ));
}

#[test]
fn rail_policy_mapping_rejects_weaker_and_observer_signing_combinations() {
    assert_eq!(
        lib_conxian_core_enclave::core_trust_to_sdk_rail_tier(TrustTier::Strict),
        Ok(SdkRailTrustTier::T1)
    );
    assert_eq!(
        lib_conxian_core_enclave::core_trust_to_sdk_rail_tier(TrustTier::Managed),
        Ok(SdkRailTrustTier::T2)
    );
    assert_eq!(
        lib_conxian_core_enclave::core_trust_to_sdk_rail_tier(TrustTier::Expedient),
        Ok(SdkRailTrustTier::T3)
    );
    assert_eq!(
        lib_conxian_core_enclave::core_trust_to_sdk_rail_tier(TrustTier::ObserverOnly),
        Err(AdapterError::ObserverOnlyCannotSign)
    );

    assert!(
        lib_conxian_core_enclave::validate_rail_trust(TrustTier::Strict, SdkRailTrustTier::T1,)
            .is_ok()
    );
    assert!(lib_conxian_core_enclave::validate_rail_trust(
        TrustTier::Managed,
        SdkRailTrustTier::T1,
    )
    .is_ok());
    assert!(matches!(
        lib_conxian_core_enclave::validate_rail_trust(TrustTier::Strict, SdkRailTrustTier::T2),
        Err(AdapterError::RailTrustDowngrade {
            requested: TrustTier::Strict,
            observed: RailTrustTier::T2,
        })
    ));
    assert!(matches!(
        lib_conxian_core_enclave::validate_rail_trust(TrustTier::Expedient, SdkRailTrustTier::T4),
        Err(AdapterError::RailTrustDowngrade {
            requested: TrustTier::Expedient,
            observed: RailTrustTier::T4,
        })
    ));

    let observer = RailTrustPolicy::new(TrustTier::ObserverOnly, RailTrustTier::T4).unwrap();
    assert_eq!(
        observer.signing_sdk_tier(),
        Err(AdapterError::ObserverOnlyCannotSign)
    );
    assert_eq!(
        lib_conxian_core_enclave::sdk_rail_tier_to_core_observation(SdkRailTrustTier::T4),
        TrustTier::ObserverOnly
    );
    assert_eq!(
        lib_conxian_core_enclave::sdk_rail_tier_to_observation(SdkRailTrustTier::T4),
        RailTrustTier::T4
    );
    assert!(matches!(
        RailTrustTier::from_wire("T5"),
        Err(AdapterError::UnknownRailTrustTier { value }) if value == "T5"
    ));
    assert!(serde_json::from_str::<RailTrustTier>(r#""t5""#).is_err());
}

#[test]
fn network_policy_round_trips_exact_sdk_values_and_rejects_unknown_wire_values() {
    for (sdk, policy) in [
        (SdkNetwork::Mainnet, NetworkPolicy::Mainnet),
        (SdkNetwork::Testnet, NetworkPolicy::Testnet),
        (SdkNetwork::Devnet, NetworkPolicy::Devnet),
    ] {
        assert_eq!(NetworkPolicy::try_from(sdk).unwrap(), policy);
        assert_eq!(policy.to_sdk(), sdk);
        let encoded = serde_json::to_string(&policy).unwrap();
        let decoded: NetworkPolicy = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, policy);
    }

    assert!(matches!(
        NetworkPolicy::from_wire("qa"),
        Err(AdapterError::UnknownNetwork { value }) if value == "qa"
    ));
    assert!(serde_json::from_str::<NetworkPolicy>(r#""qa""#).is_err());
}

#[test]
fn replay_binding_success_commits_core_identity_to_sdk_digest() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1);
    let descriptor = test_envelope();
    let policy_context = test_policy_context();
    let expected_bound_digest =
        replay_binding_digest(&descriptor, &TEST_DIGEST, &policy_context).unwrap();

    let result = adapter
        .sign_digest(&request, &descriptor, &policy_context)
        .unwrap();
    assert_eq!(manager.calls(), 1);
    assert_eq!(manager.last_request().message_hash, expected_bound_digest);
    assert_eq!(result.replay_binding.bound_digest(), expected_bound_digest);
    assert_eq!(result.replay_binding.core_digest(), TEST_DIGEST);
    assert_eq!(result.replay_binding.publisher(), descriptor.publisher);
    assert_eq!(result.replay_binding.event_id(), descriptor.event_id);
    assert_eq!(result.replay_binding.sequence(), descriptor.sequence);
    assert_eq!(
        result.replay_binding.payload_hash(),
        descriptor.payload_hash
    );
    assert_eq!(result.replay_binding.commitments(), descriptor.commitments);
    assert_eq!(result.replay_binding.policy_context(), &policy_context);
    assert_ne!(result.replay_binding.bound_digest(), TEST_DIGEST);
}

#[test]
fn replay_binding_is_unambiguous_for_delimiters_and_descriptor_mutations() {
    let context = test_policy_context();
    let base = test_envelope();

    let delimiter_left = SignedEnvelopeDescriptor {
        publisher: "publisher:event".to_owned(),
        event_id: "sequence".to_owned(),
        ..base.clone()
    };
    let delimiter_right = SignedEnvelopeDescriptor {
        publisher: "publisher".to_owned(),
        event_id: "event:sequence".to_owned(),
        ..base.clone()
    };
    assert_eq!(
        delimiter_left.idempotency_key(),
        delimiter_right.idempotency_key()
    );
    assert_ne!(
        replay_binding_digest(&delimiter_left, &TEST_DIGEST, &context).unwrap(),
        replay_binding_digest(&delimiter_right, &TEST_DIGEST, &context).unwrap()
    );

    let mut variants = Vec::new();
    let mut publisher = base.clone();
    publisher.publisher.push_str("-changed");
    variants.push(publisher);
    let mut event = base.clone();
    event.event_id.push_str("-changed");
    variants.push(event);
    let mut sequence = base.clone();
    sequence.sequence = 0;
    variants.push(sequence);
    let mut payload_hash = base.clone();
    payload_hash.payload_hash.push_str("-changed");
    variants.push(payload_hash);
    let mut commitments = base.clone();
    commitments.commitments.push("commitment-2".to_owned());
    variants.push(commitments);

    let base_digest = replay_binding_digest(&base, &TEST_DIGEST, &context).unwrap();
    for variant in variants {
        assert_ne!(
            replay_binding_digest(&variant, &TEST_DIGEST, &context).unwrap(),
            base_digest
        );
    }
    assert!(replay_binding_digest(
        &SignedEnvelopeDescriptor {
            event_id: String::new(),
            ..base
        },
        &TEST_DIGEST,
        &context,
    )
    .is_err());
}

#[test]
fn invalid_descriptor_or_policy_context_rejects_before_provider_call() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1);
    let valid_descriptor = test_envelope();
    let valid_context = test_policy_context();

    assert_eq!(
        adapter
            .sign_digest(
                &request,
                &SignedEnvelopeDescriptor {
                    event_id: String::new(),
                    ..valid_descriptor.clone()
                },
                &valid_context,
            )
            .unwrap_err(),
        AdapterError::InvalidReplayBinding
    );
    assert_eq!(manager.calls(), 0);

    let weak_context = RequestPolicyContext {
        network: NetworkPolicy::Testnet,
        rail: RailTrustPolicy {
            requested_core_tier: TrustTier::Managed,
            observed_sdk_tier: RailTrustTier::T3,
        },
    };
    assert_eq!(
        adapter
            .sign_digest(&request, &valid_descriptor, &weak_context)
            .unwrap_err(),
        AdapterError::RailTrustDowngrade {
            requested: TrustTier::Managed,
            observed: RailTrustTier::T3,
        }
    );
    assert_eq!(manager.calls(), 0);

    let mismatched_context = RequestPolicyContext {
        network: NetworkPolicy::Testnet,
        rail: RailTrustPolicy {
            requested_core_tier: TrustTier::Strict,
            observed_sdk_tier: RailTrustTier::T1,
        },
    };
    assert_eq!(
        adapter
            .sign_digest(&request, &valid_descriptor, &mismatched_context)
            .unwrap_err(),
        AdapterError::PolicyContextTierMismatch {
            adapter: TrustTier::Managed,
            requested: TrustTier::Strict,
        }
    );
    assert_eq!(manager.calls(), 0);
}

#[test]
fn deserialized_or_forged_binding_cannot_authorize_signing() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let request = digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1);
    let descriptor = test_envelope();
    let policy_context = test_policy_context();
    let response = adapter
        .sign_digest(&request, &descriptor, &policy_context)
        .unwrap();

    let mut forged_value = serde_json::to_value(&response.replay_binding).unwrap();
    forged_value["bound_digest"][0] = serde_json::json!(0);
    let forged: ReplayBinding = serde_json::from_value(forged_value).unwrap();
    assert_ne!(
        forged.bound_digest(),
        response.replay_binding.bound_digest()
    );

    let second = adapter
        .sign_digest(&request, &descriptor, &policy_context)
        .unwrap();
    assert_eq!(manager.calls(), 2);
    assert_eq!(
        manager.last_request().message_hash,
        second.replay_binding.bound_digest()
    );
    assert_ne!(manager.last_request().message_hash, forged.bound_digest());
}

#[test]
fn adapter_dtos_and_representative_errors_serde_round_trip() {
    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager, TrustTier::Managed);
    let request = digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1);
    let response = sign_digest(&adapter, &request).unwrap();
    let encoded = serde_json::to_value(&response).unwrap();
    let decoded: lib_conxian_core_enclave::EnclaveSignResponse =
        serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, response);

    let errors = [
        AdapterError::InvalidTrustPolicy {
            tier: TrustTier::Strict,
            required: Some(MinimumAttestation::HardwareBacked),
            configured: Some(MinimumAttestation::Tee),
        },
        AdapterError::RailTrustDowngrade {
            requested: TrustTier::Managed,
            observed: RailTrustTier::T3,
        },
        AdapterError::UnknownNetwork {
            value: "unknown".to_owned(),
        },
        AdapterError::PolicyContextTierMismatch {
            adapter: TrustTier::Managed,
            requested: TrustTier::Strict,
        },
        AdapterError::MalformedProviderResponse(ProviderResponseField::Signature),
    ];

    for error in errors {
        let encoded = serde_json::to_value(&error).unwrap();
        let decoded: AdapterError = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, error);
    }
}

#[test]
fn security_policy_and_binding_dtos_reject_unknown_fields() {
    assert!(serde_json::from_str::<TrustPolicy>(
        r#"{"tier":"managed","minimum_attestation":"tee","extra":true}"#
    )
    .is_err());
    assert!(serde_json::from_str::<TrustPolicy>(
        r#"{"tier":"future","minimum_attestation":"tee"}"#
    )
    .is_err());
    assert!(serde_json::from_str::<RailTrustPolicy>(
        r#"{"requested_core_tier":"managed","observed_sdk_tier":"t2","extra":true}"#
    )
    .is_err());
    assert!(serde_json::from_str::<RequestPolicyContext>(
        r#"{"network":"testnet","rail":{"requested_core_tier":"managed","observed_sdk_tier":"t2"},"extra":true}"#
    )
    .is_err());

    let manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let adapter = adapter(manager, TrustTier::Managed);
    let signed = sign_digest(
        &adapter,
        &digest_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1),
    )
    .unwrap();

    let mut binding = serde_json::to_value(&signed.replay_binding).unwrap();
    binding["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ReplayBinding>(binding).is_err());

    let mut response = serde_json::to_value(&signed).unwrap();
    response["extra"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<lib_conxian_core_enclave::EnclaveSignResponse>(response).is_err()
    );
}
