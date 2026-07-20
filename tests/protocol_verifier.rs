use chrono::{DateTime, TimeZone, Utc};
use lib_conxian_core::control_model::{
    BridgeSystem, Chain, ChainFamily, FinalityClass, TrustTier, VerificationClass,
    VerificationStatus,
};
use lib_conxian_core::verifier::{
    compute_evidence_binding_hash, validate_finality_transition, BlockHeader, ChainId,
    ChainStateReference, DynProtocolVerifier, LatestVerifiedBlock, ProofData, ProofFormat,
    ProofVerificationRequest, ProofVerificationResult, ProtocolVerifier, ProtocolVerifierBackend,
    ProtocolVerifierError, TransactionFinalityRequest, TransactionFinalityResult,
    TransactionFinalityStatus, VerificationProvenance, VerifierCapabilities, VerifierCapability,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn timestamp(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0)
        .single()
        .expect("valid timestamp")
}

fn bitcoin() -> ChainId {
    ChainId::new(ChainFamily::BitcoinUtxo, "mainnet")
}

fn ethereum() -> ChainId {
    ChainId::new(ChainFamily::Evm, "ethereum-mainnet")
}

fn make_capabilities(capabilities: Vec<VerifierCapability>) -> VerifierCapabilities {
    VerifierCapabilities {
        verifier_id: "integration-mock".to_string(),
        version: "1".to_string(),
        supported_chains: vec![bitcoin()],
        supported_families: vec![ChainFamily::BitcoinUtxo],
        capabilities,
        proof_formats: vec![ProofFormat::HeaderChain, ProofFormat::Merkle],
        verification_classes: vec![VerificationClass::LightClient],
        finality_classes: vec![FinalityClass::Probabilistic],
        trust_tiers: vec![TrustTier::Strict],
    }
}

#[derive(Clone)]
struct ResultPolicyMetadata {
    finality_class: FinalityClass,
    verification_class: VerificationClass,
    trust_tier: TrustTier,
    provenance_verifier_id: String,
}

impl Default for ResultPolicyMetadata {
    fn default() -> Self {
        Self {
            finality_class: FinalityClass::Probabilistic,
            verification_class: VerificationClass::LightClient,
            trust_tier: TrustTier::Strict,
            provenance_verifier_id: "integration-mock".to_string(),
        }
    }
}

fn block(
    chain: ChainId,
    state_root: Option<String>,
    status: VerificationStatus,
    metadata: &ResultPolicyMetadata,
) -> LatestVerifiedBlock {
    LatestVerifiedBlock {
        chain,
        header: BlockHeader {
            hash: "block-hash".to_string(),
            parent_hash: Some("parent-hash".to_string()),
            height: 100,
            timestamp: timestamp(1_000),
            state_root,
        },
        finality_class: metadata.finality_class.clone(),
        confirmations: 6,
        verification_class: metadata.verification_class.clone(),
        trust_tier: metadata.trust_tier.clone(),
        verification_status: status,
        provenance: VerificationProvenance {
            verifier_id: metadata.provenance_verifier_id.clone(),
            evidence_ref: Some("mock-proof".to_string()),
            verified_at: timestamp(1_010),
        },
    }
}

fn valid_request() -> ProofVerificationRequest {
    ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, Some("state-root".to_string())),
        ProofData::new(ProofFormat::HeaderChain, vec![1, 2, 3]),
    )
}

fn envelope(
    destination: &ChainId,
    observed_at: i64,
    expires_at: i64,
) -> lib_conxian_core::control_model::ProofEnvelope {
    lib_conxian_core::control_model::ProofEnvelope {
        system: BridgeSystem::Ibc,
        system_version: "v1".to_string(),
        trust_tier: TrustTier::Managed,
        verification_class: VerificationClass::LightClient,
        source_chain_id: "source-chain".to_string(),
        destination_chain_id: destination.canonical_id(),
        finality_class: FinalityClass::Probabilistic,
        min_confirmations: 6,
        observed_at: timestamp(observed_at),
        expires_at: timestamp(expires_at),
        proof_ref: "proof-1".to_string(),
        evidence_hash: "placeholder".to_string(),
        evidence_uri: Some("evidence://proof-1".to_string()),
        verifier_set_ref: "set-1".to_string(),
        security_params: serde_json::json!({"threshold": 2, "committee": ["a", "b"]}),
        verification_status: VerificationStatus::Verified,
        verification_reason: Some("verified by light client".to_string()),
    }
}

fn bound_request() -> ProofVerificationRequest {
    let mut request =
        valid_request().with_envelope(envelope(&bitcoin(), 1_784_000_000, 1_790_000_000));
    let binding = compute_evidence_binding_hash(&request).expect("placeholder envelope is valid");
    request.proof.evidence_hash = Some(binding.clone());
    request.envelope.as_mut().expect("envelope").evidence_hash = binding;
    request
}

#[derive(Clone, Copy)]
enum StateResponse {
    Valid,
    WrongChain,
    WrongBlock,
    MissingStateRoot,
    MismatchedStateRoot,
    WrongFormat,
    Degraded,
}

type RequestMutation = Box<dyn Fn(&mut ProofVerificationRequest)>;

#[derive(Clone)]
struct MockBackend {
    capabilities: VerifierCapabilities,
    state_response: StateResponse,
    result_metadata: ResultPolicyMetadata,
    calls: Arc<AtomicUsize>,
}

impl MockBackend {
    fn new(capabilities: Vec<VerifierCapability>) -> Self {
        Self {
            capabilities: make_capabilities(capabilities),
            state_response: StateResponse::Valid,
            result_metadata: ResultPolicyMetadata::default(),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn with_state_response(mut self, state_response: StateResponse) -> Self {
        self.state_response = state_response;
        self
    }

    fn with_result_metadata(mut self, result_metadata: ResultPolicyMetadata) -> Self {
        self.result_metadata = result_metadata;
        self
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for MockBackend {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let (chain, state, proof_format, state_root, status) = match self.state_response {
            StateResponse::Valid => (
                request.chain.clone(),
                request.state.clone(),
                request.proof.format.clone(),
                request.state.state_root.clone(),
                VerificationStatus::Verified,
            ),
            StateResponse::WrongChain => (
                ethereum(),
                request.state.clone(),
                request.proof.format.clone(),
                request.state.state_root.clone(),
                VerificationStatus::Verified,
            ),
            StateResponse::WrongBlock => (
                request.chain.clone(),
                ChainStateReference::new("other-block", 101, request.state.state_root.clone()),
                request.proof.format.clone(),
                request.state.state_root.clone(),
                VerificationStatus::Verified,
            ),
            StateResponse::MissingStateRoot => (
                request.chain.clone(),
                ChainStateReference::new(
                    request.state.block_hash.clone(),
                    request.state.block_height,
                    None,
                ),
                request.proof.format.clone(),
                None,
                VerificationStatus::Verified,
            ),
            StateResponse::MismatchedStateRoot => (
                request.chain.clone(),
                ChainStateReference::new(
                    request.state.block_hash.clone(),
                    request.state.block_height,
                    Some("different-root".to_string()),
                ),
                request.proof.format.clone(),
                Some("different-root".to_string()),
                VerificationStatus::Verified,
            ),
            StateResponse::WrongFormat => (
                request.chain.clone(),
                request.state.clone(),
                ProofFormat::Merkle,
                request.state.state_root.clone(),
                VerificationStatus::Verified,
            ),
            StateResponse::Degraded => (
                request.chain.clone(),
                request.state.clone(),
                request.proof.format.clone(),
                request.state.state_root.clone(),
                VerificationStatus::Degraded,
            ),
        };

        Ok(ProofVerificationResult {
            chain: chain.clone(),
            state,
            proof_format,
            verified_block: block(chain, state_root, status, &self.result_metadata),
        })
    }

    fn backend_get_latest_verified_block(
        &self,
        chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(block(
            chain.clone(),
            Some("state-root".to_string()),
            VerificationStatus::Verified,
            &self.result_metadata,
        ))
    }

    fn backend_verify_transaction_finality(
        &self,
        request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(TransactionFinalityResult {
            chain: request.chain.clone(),
            transaction_id: request.transaction_id.clone(),
            status: TransactionFinalityStatus::Finalized { confirmations: 6 },
            finality_class: self.result_metadata.finality_class.clone(),
            required_confirmations: request.min_confirmations,
            observed_confirmations: 6,
            latest_block: Some(block(
                request.chain.clone(),
                Some("state-root".to_string()),
                VerificationStatus::Verified,
                &self.result_metadata,
            )),
            verification_class: self.result_metadata.verification_class.clone(),
            trust_tier: self.result_metadata.trust_tier.clone(),
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: self.result_metadata.provenance_verifier_id.clone(),
                evidence_ref: Some("mock-finality".to_string()),
                verified_at: timestamp(1_010),
            },
        })
    }
}

#[test]
fn facade_accepts_valid_proof_and_supports_dynamic_dispatch() {
    let backend = MockBackend::new(vec![
        VerifierCapability::StateProofVerification,
        VerifierCapability::LatestVerifiedBlock,
        VerifierCapability::TransactionFinality,
    ]);
    let verifier = ProtocolVerifier::new(backend.clone());
    let result = verifier
        .verify_chain_state_at(&valid_request(), timestamp(2_000))
        .expect("valid proof");
    assert!(result.is_verified());

    let latest = verifier
        .get_latest_verified_block_at(&bitcoin(), timestamp(2_000))
        .expect("valid latest block");
    assert!(latest.is_verified());

    let finality = verifier
        .verify_transaction_finality_at(
            &TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true),
            timestamp(2_000),
        )
        .expect("valid finality");
    assert!(finality.is_final());
    assert_eq!(backend.calls(), 3);

    let encoded = serde_json::to_vec(&result).expect("serialize result");
    let decoded: ProofVerificationResult =
        serde_json::from_slice(&encoded).expect("deserialize result");
    assert_eq!(decoded, result);

    let dynamic: DynProtocolVerifier = ProtocolVerifier::new(Box::new(backend));
    assert!(dynamic
        .verify_chain_state_at(&valid_request(), timestamp(2_000))
        .is_ok());
}

#[test]
fn facade_rejects_trust_tier_downgrade_and_upgrade() {
    let downgrade_backend = MockBackend::new(vec![VerifierCapability::StateProofVerification])
        .with_result_metadata(ResultPolicyMetadata {
            trust_tier: TrustTier::Managed,
            ..ResultPolicyMetadata::default()
        });
    let downgrade_verifier = ProtocolVerifier::new(downgrade_backend.clone());
    assert!(matches!(
        downgrade_verifier.verify_chain_state_at(&valid_request(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedTrustTier {
            trust_tier: TrustTier::Managed,
            ..
        })
    ));
    assert_eq!(downgrade_backend.calls(), 1);

    let mut upgrade_backend = MockBackend::new(vec![VerifierCapability::LatestVerifiedBlock]);
    upgrade_backend.capabilities.trust_tiers = vec![TrustTier::Managed];
    let upgrade_verifier = ProtocolVerifier::new(upgrade_backend.clone());
    assert!(matches!(
        upgrade_verifier.get_latest_verified_block_at(&bitcoin(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedTrustTier {
            trust_tier: TrustTier::Strict,
            ..
        })
    ));
    assert_eq!(upgrade_backend.calls(), 1);
}

#[test]
fn facade_accepts_valid_advertised_managed_external_quorum_combination() {
    let mut backend = MockBackend::new(vec![VerifierCapability::StateProofVerification]);
    backend.capabilities.verification_classes = vec![VerificationClass::ExternalQuorum];
    backend.capabilities.trust_tiers = vec![TrustTier::Managed];
    backend.result_metadata = ResultPolicyMetadata {
        verification_class: VerificationClass::ExternalQuorum,
        trust_tier: TrustTier::Managed,
        ..ResultPolicyMetadata::default()
    };

    let verifier = ProtocolVerifier::new(backend.clone());
    let result = verifier
        .verify_chain_state_at(&valid_request(), timestamp(2_000))
        .expect("advertised managed external-quorum result");
    assert!(result.is_verified());
    assert_eq!(backend.calls(), 1);
}

#[test]
fn facade_rejects_unadvertised_policy_metadata_for_every_operation() {
    let mut state_class_backend =
        MockBackend::new(vec![VerifierCapability::StateProofVerification]);
    state_class_backend.capabilities.trust_tiers = vec![TrustTier::Strict, TrustTier::Managed];
    state_class_backend.result_metadata = ResultPolicyMetadata {
        trust_tier: TrustTier::Managed,
        verification_class: VerificationClass::ExternalQuorum,
        ..ResultPolicyMetadata::default()
    };
    let state_class_verifier = ProtocolVerifier::new(state_class_backend.clone());
    assert!(matches!(
        state_class_verifier.verify_chain_state_at(&valid_request(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedVerificationClass {
            verification_class: VerificationClass::ExternalQuorum,
            ..
        })
    ));
    assert_eq!(state_class_backend.calls(), 1);

    let state_finality_backend = MockBackend::new(vec![VerifierCapability::StateProofVerification])
        .with_result_metadata(ResultPolicyMetadata {
            finality_class: FinalityClass::Deterministic,
            ..ResultPolicyMetadata::default()
        });
    let state_finality_verifier = ProtocolVerifier::new(state_finality_backend.clone());
    assert!(matches!(
        state_finality_verifier.verify_chain_state_at(&valid_request(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedFinalityClass {
            finality_class: FinalityClass::Deterministic,
            ..
        })
    ));
    assert_eq!(state_finality_backend.calls(), 1);

    let mut latest_class_backend = MockBackend::new(vec![VerifierCapability::LatestVerifiedBlock]);
    latest_class_backend.capabilities.trust_tiers = vec![TrustTier::Managed];
    latest_class_backend.result_metadata = ResultPolicyMetadata {
        trust_tier: TrustTier::Managed,
        verification_class: VerificationClass::ExternalQuorum,
        ..ResultPolicyMetadata::default()
    };
    let latest_class_verifier = ProtocolVerifier::new(latest_class_backend.clone());
    assert!(matches!(
        latest_class_verifier.get_latest_verified_block_at(&bitcoin(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedVerificationClass {
            verification_class: VerificationClass::ExternalQuorum,
            ..
        })
    ));
    assert_eq!(latest_class_backend.calls(), 1);

    let latest_finality_backend = MockBackend::new(vec![VerifierCapability::LatestVerifiedBlock])
        .with_result_metadata(ResultPolicyMetadata {
            finality_class: FinalityClass::Deterministic,
            ..ResultPolicyMetadata::default()
        });
    let latest_finality_verifier = ProtocolVerifier::new(latest_finality_backend.clone());
    assert!(matches!(
        latest_finality_verifier.get_latest_verified_block_at(&bitcoin(), timestamp(2_000)),
        Err(ProtocolVerifierError::UnadvertisedFinalityClass {
            finality_class: FinalityClass::Deterministic,
            ..
        })
    ));
    assert_eq!(latest_finality_backend.calls(), 1);

    let mut finality_class_backend =
        MockBackend::new(vec![VerifierCapability::TransactionFinality]);
    finality_class_backend.capabilities.trust_tiers = vec![TrustTier::Strict, TrustTier::Managed];
    finality_class_backend.result_metadata = ResultPolicyMetadata {
        trust_tier: TrustTier::Managed,
        verification_class: VerificationClass::ExternalQuorum,
        ..ResultPolicyMetadata::default()
    };
    let finality_class_verifier = ProtocolVerifier::new(finality_class_backend.clone());
    assert!(matches!(
        finality_class_verifier.verify_transaction_finality_at(
            &TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true),
            timestamp(2_000),
        ),
        Err(ProtocolVerifierError::UnadvertisedVerificationClass {
            verification_class: VerificationClass::ExternalQuorum,
            ..
        })
    ));
    assert_eq!(finality_class_backend.calls(), 1);

    let finality_backend = MockBackend::new(vec![VerifierCapability::TransactionFinality])
        .with_result_metadata(ResultPolicyMetadata {
            finality_class: FinalityClass::Deterministic,
            ..ResultPolicyMetadata::default()
        });
    let finality_verifier = ProtocolVerifier::new(finality_backend.clone());
    assert!(matches!(
        finality_verifier.verify_transaction_finality_at(
            &TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true),
            timestamp(2_000),
        ),
        Err(ProtocolVerifierError::UnadvertisedFinalityClass {
            finality_class: FinalityClass::Deterministic,
            ..
        })
    ));
    assert_eq!(finality_backend.calls(), 1);
}

#[test]
fn facade_rejects_result_provenance_from_another_verifier() {
    let backend = MockBackend::new(vec![VerifierCapability::LatestVerifiedBlock])
        .with_result_metadata(ResultPolicyMetadata {
            provenance_verifier_id: "other-verifier".to_string(),
            ..ResultPolicyMetadata::default()
        });
    let verifier = ProtocolVerifier::new(backend.clone());
    assert!(matches!(
        verifier.get_latest_verified_block_at(&bitcoin(), timestamp(2_000)),
        Err(ProtocolVerifierError::VerifierIdentityMismatch {
            expected,
            actual,
        }) if expected == "integration-mock" && actual == "other-verifier"
    ));
    assert_eq!(backend.calls(), 1);
}

#[test]
fn adversarial_backend_cannot_bypass_facade_postconditions() {
    let cases = [
        StateResponse::WrongChain,
        StateResponse::WrongBlock,
        StateResponse::MissingStateRoot,
        StateResponse::MismatchedStateRoot,
        StateResponse::WrongFormat,
        StateResponse::Degraded,
    ];

    for response in cases {
        let backend = MockBackend::new(vec![VerifierCapability::StateProofVerification])
            .with_state_response(response);
        let verifier = ProtocolVerifier::new(backend.clone());
        assert!(verifier
            .verify_chain_state_at(&valid_request(), timestamp(2_000))
            .is_err());
        assert_eq!(backend.calls(), 1);
    }
}

#[test]
fn invalid_requests_and_capabilities_do_not_call_backend() {
    let backend = MockBackend::new(vec![VerifierCapability::StateProofVerification]);
    let verifier = ProtocolVerifier::new(backend.clone());

    let empty_proof = ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::Merkle, Vec::new()),
    );
    assert!(matches!(
        verifier.verify_chain_state_at(&empty_proof, timestamp(2_000)),
        Err(ProtocolVerifierError::InsufficientProofData { .. })
    ));
    assert_eq!(backend.calls(), 0);

    let unsupported_format = ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::ZkProof, vec![1]),
    );
    assert!(matches!(
        verifier.verify_chain_state_at(&unsupported_format, timestamp(2_000)),
        Err(ProtocolVerifierError::UnsupportedProofFormat { .. })
    ));
    assert_eq!(backend.calls(), 0);

    let block_only = MockBackend::new(vec![VerifierCapability::LatestVerifiedBlock]);
    let block_verifier = ProtocolVerifier::new(block_only.clone());
    let finality_request = TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true);
    assert!(matches!(
        block_verifier.verify_transaction_finality_at(&finality_request, timestamp(2_000)),
        Err(ProtocolVerifierError::UnsupportedCapability { .. })
    ));
    assert_eq!(block_only.calls(), 0);

    let unsupported_chain = ProofVerificationRequest::new(
        ethereum(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::HeaderChain, vec![1]),
    );
    assert!(matches!(
        verifier.verify_chain_state_at(&unsupported_chain, timestamp(2_000)),
        Err(ProtocolVerifierError::UnsupportedChain { .. })
    ));
    assert_eq!(backend.calls(), 0);

    let mut invalid_capabilities =
        MockBackend::new(vec![VerifierCapability::StateProofVerification]);
    invalid_capabilities.capabilities.trust_tiers.clear();
    let invalid_verifier = ProtocolVerifier::new(invalid_capabilities.clone());
    assert!(matches!(
        invalid_verifier.verify_chain_state_at(&valid_request(), timestamp(2_000)),
        Err(ProtocolVerifierError::InvariantViolation { .. })
    ));
    assert_eq!(invalid_capabilities.calls(), 0);
    assert!(ProtocolVerifier::try_new(invalid_capabilities).is_err());
}

#[test]
fn chain_family_mapping_is_checked_in_constructor_and_deserialization() {
    let known = ChainId::from_chain(Chain::Ethereum, "mainnet");
    assert_eq!(known.family, ChainFamily::Evm);
    assert!(matches!(
        ChainId::try_from_parts(Some(Chain::Ethereum), ChainFamily::BitcoinUtxo, "mainnet"),
        Err(ProtocolVerifierError::InvalidChainFamily { .. })
    ));

    let mismatched = serde_json::json!({
        "family": "bitcoin_utxo",
        "chain": "ethereum",
        "network": "mainnet"
    });
    assert!(serde_json::from_value::<ChainId>(mismatched).is_err());

    let encoded = serde_json::to_string(&known).expect("known chain serializes");
    let decoded: ChainId = serde_json::from_str(&encoded).expect("known chain deserializes");
    assert_eq!(decoded, known);
}

#[test]
fn proof_result_requires_requested_state_root_and_exact_identity() {
    for response in [
        StateResponse::MissingStateRoot,
        StateResponse::MismatchedStateRoot,
    ] {
        let backend = MockBackend::new(vec![VerifierCapability::StateProofVerification])
            .with_state_response(response);
        let verifier = ProtocolVerifier::new(backend);
        let error = verifier
            .verify_chain_state_at(&valid_request(), timestamp(2_000))
            .expect_err("invalid state root must fail closed");
        assert!(matches!(
            error,
            ProtocolVerifierError::MissingStateRoot { .. }
                | ProtocolVerifierError::MismatchedStateRoot { .. }
        ));
    }
}

#[test]
fn envelope_timestamps_are_future_safe_expiry_safe_and_malformed_safe() {
    let future = valid_request().with_envelope(envelope(&bitcoin(), 3_000, 4_000));
    assert!(matches!(
        future.validate_at(timestamp(2_000)),
        Err(ProtocolVerifierError::FutureDatedEvidence { .. })
    ));

    let expired = valid_request().with_envelope(envelope(&bitcoin(), 1_000, 2_000));
    assert!(matches!(
        expired.validate_at(timestamp(2_000)),
        Err(ProtocolVerifierError::ExpiredEvidence { .. })
    ));

    let malformed = valid_request().with_envelope(envelope(&bitcoin(), 2_000, 2_000));
    assert!(matches!(
        malformed.validate_at(timestamp(2_000)),
        Err(ProtocolVerifierError::MalformedProof { .. })
    ));
}

#[test]
fn evidence_binding_is_deterministic_and_detects_every_material_mutation() {
    let request = bound_request();
    let expected = request.evidence_binding_hash().expect("valid binding");
    assert_eq!(
        request.proof.evidence_hash.as_deref(),
        Some(expected.as_str())
    );
    assert_eq!(compute_evidence_binding_hash(&request).unwrap(), expected);

    let mutations: Vec<RequestMutation> = vec![
        Box::new(|request| request.chain.family = ChainFamily::Evm),
        Box::new(|request| request.chain.chain = Some(Chain::Bitcoin)),
        Box::new(|request| request.chain.network.push_str("-mutated")),
        Box::new(|request| request.state.block_hash.push_str("-mutated")),
        Box::new(|request| request.state.block_height += 1),
        Box::new(|request| request.state.state_root = Some("other-root".to_string())),
        Box::new(|request| request.proof.format = ProofFormat::Merkle),
        Box::new(|request| request.proof.bytes.push(9)),
        Box::new(|request| request.proof.evidence_hash.as_mut().unwrap().push('0')),
        Box::new(|request| request.envelope.as_mut().unwrap().system = BridgeSystem::Hyperlane),
        Box::new(|request| request.envelope.as_mut().unwrap().system_version.push('2')),
        Box::new(|request| request.envelope.as_mut().unwrap().trust_tier = TrustTier::Strict),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().verification_class = VerificationClass::ZkVerified
        }),
        Box::new(|request| {
            request
                .envelope
                .as_mut()
                .unwrap()
                .source_chain_id
                .push_str("-mutated")
        }),
        Box::new(|request| {
            request
                .envelope
                .as_mut()
                .unwrap()
                .destination_chain_id
                .push_str("-mutated")
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().finality_class = FinalityClass::Deterministic
        }),
        Box::new(|request| request.envelope.as_mut().unwrap().min_confirmations += 1),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().observed_at = timestamp(1_784_000_001)
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().expires_at = timestamp(1_790_000_001)
        }),
        Box::new(|request| {
            request
                .envelope
                .as_mut()
                .unwrap()
                .proof_ref
                .push_str("-mutated")
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().evidence_uri = Some("evidence://mutated".to_string())
        }),
        Box::new(|request| {
            request
                .envelope
                .as_mut()
                .unwrap()
                .verifier_set_ref
                .push_str("-mutated")
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().security_params["threshold"] = serde_json::json!(3)
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().verification_status = VerificationStatus::Degraded
        }),
        Box::new(|request| {
            request.envelope.as_mut().unwrap().verification_reason = Some("mutated".to_string())
        }),
        Box::new(|request| request.envelope.as_mut().unwrap().evidence_hash.push('0')),
    ];

    for mutate in mutations {
        let mut mutated = request.clone();
        mutate(&mut mutated);
        assert!(mutated.validate_at(timestamp(1_785_000_000)).is_err());
    }
}

#[test]
fn finality_facade_checks_request_result_and_transitions() {
    let backend = MockBackend::new(vec![VerifierCapability::TransactionFinality]);
    let verifier = ProtocolVerifier::new(backend.clone());
    let result = verifier
        .verify_transaction_finality_at(
            &TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true),
            timestamp(2_000),
        )
        .expect("finalized transaction");
    assert!(result.is_final());
    assert_eq!(backend.calls(), 1);

    let pending = TransactionFinalityStatus::Pending;
    let confirmed = TransactionFinalityStatus::Confirmed { confirmations: 3 };
    let finalized = TransactionFinalityStatus::Finalized { confirmations: 6 };
    assert!(validate_finality_transition(&pending, &confirmed).is_ok());
    assert!(validate_finality_transition(&confirmed, &finalized).is_ok());
    assert!(validate_finality_transition(&finalized, &pending).is_err());
}
