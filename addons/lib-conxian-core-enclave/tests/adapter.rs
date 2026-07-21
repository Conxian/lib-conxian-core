use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use conxius_enclave_sdk::{
    enclave::{
        attestation::{AttestationLevel as SdkAttestationLevel, DeviceIntegrityReport},
        EnclaveManager, SignRequest as SdkSignRequest, SignResponse as SdkSignResponse,
        SigningAlgorithm as SdkSigningAlgorithm,
    },
    ConclaveError, ConclaveResult,
};
use lib_conxian_core::{
    control_model::{
        Bip110PreflightMeasurements, Bip110PreflightPhase, Bip110PreflightRequest, Chain, TrustTier,
    },
    signing::{
        DerivationContext, DerivationIndex, DerivationPath, DerivationPurpose, DigestAlgorithm,
        SignRequest, SignatureEncoding, SigningAlgorithm, SigningPayload, SigningTarget,
    },
};
use lib_conxian_core_enclave::{
    extract_sdk_digest, from_sdk_algorithm, render_derivation_path, to_sdk_algorithm, AdapterError,
    AttestationLevel, EnclaveSdkAdapter, MinimumAttestation, ProviderResponseField, TrustPolicy,
};

#[derive(Clone)]
enum Outcome {
    Response(SdkSignResponse),
    Failure,
}

struct RecordingManager {
    calls: AtomicUsize,
    outcome: Mutex<Outcome>,
    last_request: Mutex<Option<SdkSignRequest>>,
}

impl RecordingManager {
    fn new(response: SdkSignResponse) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            outcome: Mutex::new(Outcome::Response(response)),
            last_request: Mutex::new(None),
        }
    }

    fn failing() -> Self {
        Self {
            calls: AtomicUsize::new(0),
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
    serde_json::to_string(&DeviceIntegrityReport {
        level,
        challenge_nonce: vec![1, 2, 3],
        signature: vec![4, 5, 6],
        certificate_chain: vec!["device".to_owned(), "root".to_owned()],
        timestamp: 1,
        extension_data: "deterministic-test-double".to_owned(),
    })
    .unwrap()
}

fn response(
    signature_bytes: usize,
    public_key_bytes: usize,
    level: SdkAttestationLevel,
) -> SdkSignResponse {
    SdkSignResponse {
        signature_hex: hex::encode(vec![0x11; signature_bytes]),
        public_key_hex: hex::encode(vec![0x22; public_key_bytes]),
        device_attestation: Some(attestation(level)),
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
        SigningPayload::digest(DigestAlgorithm::Sha256, vec![7; 32]),
    )
}

fn adapter(manager: Arc<RecordingManager>, tier: TrustTier) -> EnclaveSdkAdapter {
    EnclaveSdkAdapter::new(manager, "test-key", tier).unwrap()
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

    let sdk_request = adapter.build_sdk_sign_request(&request).unwrap();
    assert_eq!(sdk_request.algorithm, SdkSigningAlgorithm::EcdsaSecp256k1);
    assert_eq!(sdk_request.message_hash, vec![7; 32]);
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
    let compact = adapter
        .sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1,
        ))
        .unwrap();
    assert_eq!(compact.signature.encoding, SignatureEncoding::Compact);
    assert_eq!(compact.signature.bytes.len(), 64);
    assert_eq!(compact.verification_key.bytes.len(), 33);
    let forwarded = manager.last_request();
    assert_eq!(forwarded.algorithm, SdkSigningAlgorithm::EcdsaSecp256k1);
    assert_eq!(forwarded.message_hash, vec![7; 32]);
    assert_eq!(forwarded.derivation_path, "m/44'/4294967295");
    assert_eq!(forwarded.key_id, "test-key");

    manager.set_response(response(65, 65, SdkAttestationLevel::TEE));
    let recoverable = adapter
        .sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1,
        ))
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
    let schnorr = schnorr_adapter
        .sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::SchnorrSecp256k1,
        ))
        .unwrap();
    assert_eq!(schnorr.signature.encoding, SignatureEncoding::Compact);
    assert_eq!(schnorr.verification_key.bytes.len(), 32);

    let ed_manager = Arc::new(RecordingManager::new(response(
        64,
        32,
        SdkAttestationLevel::TEE,
    )));
    let ed_adapter = EnclaveSdkAdapter::new(ed_manager, "test-key", TrustTier::Managed).unwrap();
    let ed25519 = ed_adapter
        .sign_digest(&digest_request(Chain::Ethereum, SigningAlgorithm::Ed25519))
        .unwrap();
    assert_eq!(ed25519.signature.encoding, SignatureEncoding::Raw);
    assert_eq!(ed25519.verification_key.bytes.len(), 32);
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
        let error = adapter
            .sign_digest(&digest_request(
                Chain::Ethereum,
                SigningAlgorithm::EcdsaSecp256k1,
            ))
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
        observer.sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1
        )),
        Err(AdapterError::ObserverOnlyCannotSign)
    ));
    assert_eq!(observer_manager.calls(), 0);

    let strict_tee_manager = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::TEE,
    )));
    let strict_tee = adapter(strict_tee_manager.clone(), TrustTier::Strict);
    assert!(matches!(
        strict_tee.sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1
        )),
        Err(AdapterError::InsufficientAttestation {
            required: Some(_),
            observed: AttestationLevel::Tee
        })
    ));
    assert_eq!(strict_tee_manager.calls(), 1);

    let strict_hardware = Arc::new(RecordingManager::new(response(
        64,
        33,
        SdkAttestationLevel::StrongBox,
    )));
    let strict = adapter(strict_hardware.clone(), TrustTier::Strict);
    assert_eq!(
        strict
            .sign_digest(&digest_request(
                Chain::Ethereum,
                SigningAlgorithm::EcdsaSecp256k1
            ))
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
        managed.sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1
        )),
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
        adapter.sign_digest_with_bip110_preflight(&request, &preflight),
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
        adapter.sign_digest_with_bip110_preflight(&request, &missing),
        Err(AdapterError::PreflightRejected { code }) if code == "missing_measurement_data"
    ));
    assert_eq!(manager.calls(), 0);

    let unsupported = Bip110PreflightRequest::new(
        Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::Taproot,
        Bip110PreflightMeasurements::new(vec![], vec![], vec![], vec![]),
    );
    assert!(matches!(
        adapter.sign_digest_with_bip110_preflight(&request, &unsupported),
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

    let result = adapter
        .sign_digest_with_bip110_preflight(&request, &preflight)
        .unwrap();
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
        adapter.sign_digest(&digest_request(
            Chain::Bitcoin,
            SigningAlgorithm::EcdsaSecp256k1
        )),
        Err(AdapterError::PreflightRequired)
    ));
    assert_eq!(manager.calls(), 0);

    adapter
        .sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1,
        ))
        .unwrap();
    assert_eq!(manager.calls(), 1);
}

#[test]
fn provider_failures_are_collapsed_to_secret_safe_errors() {
    let manager = Arc::new(RecordingManager::failing());
    let adapter = adapter(manager.clone(), TrustTier::Managed);
    let error = adapter
        .sign_digest(&digest_request(
            Chain::Ethereum,
            SigningAlgorithm::EcdsaSecp256k1,
        ))
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
