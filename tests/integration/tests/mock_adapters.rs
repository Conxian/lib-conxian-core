//! Mock adapter implementations and shared test fixtures for integration tests.
//!
//! Provides [`MockVerifierBackend`] and [`MockSigner`] with configurable
//! response queues and call recording, plus helpers for constructing
//! Bitcoin-family request/response values.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use lib_conxian_core::control_model::{
    Chain, ChainFamily, FinalityClass, TrustTier, VerificationClass, VerificationStatus,
};
use lib_conxian_core::signing::{
    AddressFormat, ChainAddress, ChainSigningCapability, DerivationContext, DerivationIndex,
    DerivationPath, DerivationPurpose, PublicVerificationKey, SignRequest, SignResponse,
    Signature, SignatureEncoding, SignerCapabilities, SigningAlgorithm, SigningOperation,
    SigningPayload, SigningTarget, UniversalChainSigner,
    UNIVERSAL_CHAIN_SIGNER_API_VERSION,
};
use lib_conxian_core::verifier::{
    ChainId, LatestVerifiedBlock, ProofVerificationRequest, ProofVerificationResult,
    ProtocolVerifierBackend, ProtocolVerifierError, TransactionFinalityRequest,
    TransactionFinalityResult, TransactionFinalityStatus, VerificationProvenance,
    VerifierCapabilities, VerifierCapability,
};

// ── Shared test helpers ──

pub fn btc_target() -> SigningTarget {
    SigningTarget::new(Chain::Bitcoin, ChainFamily::BitcoinUtxo)
}

pub fn dctx(purpose: DerivationPurpose) -> DerivationContext {
    DerivationContext::new(
        DerivationPath::new(vec![DerivationIndex::new(84, true)]),
        purpose,
    )
}

pub fn btc_sig(bytes: &[u8]) -> Signature {
    Signature::new(SigningAlgorithm::EcdsaSecp256k1, SignatureEncoding::Der, bytes)
}

pub fn btc_pubkey(bytes: &[u8]) -> PublicVerificationKey {
    PublicVerificationKey::new(SigningAlgorithm::EcdsaSecp256k1, bytes)
}

pub fn btc_chain() -> ChainId {
    ChainId::new(ChainFamily::BitcoinUtxo, "mainnet")
}

pub fn bitcoin_sign_request() -> SignRequest {
    SignRequest::new(
        btc_target(),
        SigningAlgorithm::EcdsaSecp256k1,
        SigningPayload::message(b"hello bitcoin".to_vec()),
        dctx(DerivationPurpose::Payment),
    )
}

fn timestamp() -> chrono::DateTime<Utc> {
    Utc::now()
}

// ── MockVerifierBackend ──

/// Records calls and returns pre-programmed or default responses for each
/// backend method. Uses `HashMap<String, TransactionFinalityStatus>` keyed by
/// transaction id rather than `ChainId` (which may not implement `Hash`).
pub struct MockVerifierBackend {
    pub capabilities: VerifierCapabilities,
    pub finality_statuses: Mutex<HashMap<String, TransactionFinalityStatus>>,
    pub latest_blocks: Mutex<HashMap<String, LatestVerifiedBlock>>,
    pub verify_chain_state_calls: Mutex<Vec<ProofVerificationRequest>>,
    pub get_latest_block_calls: Mutex<Vec<ChainId>>,
    pub verify_finality_calls: Mutex<Vec<TransactionFinalityRequest>>,
}

impl MockVerifierBackend {
    pub fn new(capabilities: VerifierCapabilities) -> Self {
        Self {
            capabilities,
            finality_statuses: Mutex::new(HashMap::new()),
            latest_blocks: Mutex::new(HashMap::new()),
            verify_chain_state_calls: Mutex::new(Vec::new()),
            get_latest_block_calls: Mutex::new(Vec::new()),
            verify_finality_calls: Mutex::new(Vec::new()),
        }
    }

    pub fn bitcoin_capable() -> Self {
        let chain = btc_chain();
        Self::new(VerifierCapabilities {
            verifier_id: "mock-verifier".to_string(),
            version: "1".to_string(),
            supported_chains: vec![chain.clone()],
            supported_families: vec![ChainFamily::BitcoinUtxo],
            capabilities: vec![
                VerifierCapability::StateProofVerification,
                VerifierCapability::LatestVerifiedBlock,
                VerifierCapability::TransactionFinality,
            ],
            proof_formats: vec![lib_conxian_core::verifier::ProofFormat::HeaderChain],
            verification_classes: vec![
                VerificationClass::LightClient,
                VerificationClass::NativeObservation,
            ],
            finality_classes: vec![FinalityClass::Economic, FinalityClass::Probabilistic],
            trust_tiers: vec![TrustTier::Strict, TrustTier::Managed, TrustTier::Expedient],
        })
    }

    /// Pre-programs a finality response for the given transaction id.
    pub fn set_finality_status(&self, tx_id: &str, status: TransactionFinalityStatus) {
        self.finality_statuses
            .lock()
            .unwrap()
            .insert(tx_id.to_string(), status);
    }

    /// Pre-programs a latest-block response for the given chain.
    pub fn set_latest_block(&self, chain: &ChainId, block: LatestVerifiedBlock) {
        self.latest_blocks
            .lock()
            .unwrap()
            .insert(chain.to_string(), block);
    }

    fn make_finality_result(
        &self,
        request: &TransactionFinalityRequest,
        status: TransactionFinalityStatus,
    ) -> TransactionFinalityResult {
        let observed = status.confirmations();
        TransactionFinalityResult {
            chain: request.chain.clone(),
            transaction_id: request.transaction_id.clone(),
            status,
            finality_class: FinalityClass::Economic,
            required_confirmations: request.min_confirmations,
            observed_confirmations: observed,
            latest_block: None,
            verification_class: VerificationClass::NativeObservation,
            trust_tier: TrustTier::Managed,
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: "mock-verifier".to_string(),
                evidence_ref: None,
                verified_at: timestamp(),
            },
        }
    }
}

impl ProtocolVerifierBackend for MockVerifierBackend {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.verify_chain_state_calls
            .lock()
            .unwrap()
            .push(request.clone());
        Err(ProtocolVerifierError::UnsupportedCapability {
            chain: request.chain.clone(),
            capability: VerifierCapability::StateProofVerification,
        })
    }

    fn backend_get_latest_verified_block(
        &self,
        chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.get_latest_block_calls
            .lock()
            .unwrap()
            .push(chain.clone());

        if let Some(block) = self.latest_blocks.lock().unwrap().get(&chain.to_string()) {
            return Ok(block.clone());
        }

        Err(ProtocolVerifierError::UnsupportedChain {
            chain: chain.clone(),
        })
    }

    fn backend_verify_transaction_finality(
        &self,
        request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.verify_finality_calls
            .lock()
            .unwrap()
            .push(request.clone());

        let status = self
            .finality_statuses
            .lock()
            .unwrap()
            .get(&request.transaction_id)
            .cloned()
            .unwrap_or(TransactionFinalityStatus::Pending);

        Ok(self.make_finality_result(request, status))
    }
}

// ── MockSigner ──

/// Mock [`UniversalChainSigner`] that records calls and returns queued or
/// default [`SignResponse`] values.
pub struct MockSigner {
    pub capabilities: SignerCapabilities,
    pub queued_responses: Mutex<Vec<SignResponse>>,
    pub sign_calls: Mutex<Vec<SignRequest>>,
}

impl MockSigner {
    pub fn new(capabilities: SignerCapabilities) -> Self {
        Self {
            capabilities,
            queued_responses: Mutex::new(Vec::new()),
            sign_calls: Mutex::new(Vec::new()),
        }
    }

    pub fn bitcoin_capable() -> Self {
        let target = btc_target();
        Self::new(SignerCapabilities::new(
            UNIVERSAL_CHAIN_SIGNER_API_VERSION,
            vec![ChainSigningCapability::new(
                target,
                vec![SigningAlgorithm::EcdsaSecp256k1],
                vec![SigningOperation::SignMessage],
                vec![AddressFormat::BitcoinBech32],
            )],
        ))
    }

    /// Queues a response to be returned by the next `sign_impl` call.
    pub fn queue_response(&self, response: SignResponse) {
        self.queued_responses.lock().unwrap().push(response);
    }

    fn default_response_for(&self, request: &SignRequest) -> SignResponse {
        SignResponse {
            signature: btc_sig(&[0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x02]),
            verification_key: btc_pubkey(&[
                0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            ]),
            address: ChainAddress::new(Chain::Bitcoin, AddressFormat::BitcoinBech32, "bc1qtest"),
            derivation: request.derivation.clone(),
        }
    }
}

impl UniversalChainSigner for MockSigner {
    fn capabilities(&self) -> &SignerCapabilities {
        &self.capabilities
    }

    fn sign_impl(&self, request: &SignRequest) -> Result<SignResponse, lib_conxian_core::signing::SigningError> {
        self.sign_calls.lock().unwrap().push(request.clone());

        if let Some(response) = self.queued_responses.lock().unwrap().pop() {
            return Ok(response);
        }

        Ok(self.default_response_for(request))
    }
}

// ── Tests ──

#[test]
fn mock_verifier_backend_records_finality_calls() {
    let backend = MockVerifierBackend::bitcoin_capable();
    let chain = btc_chain();
    let request =
        TransactionFinalityRequest::new(chain.clone(), "tx-abc123", 6, true);

    backend.set_finality_status("tx-abc123", TransactionFinalityStatus::Confirmed { confirmations: 3 });
    let result = backend
        .backend_verify_transaction_finality(&request)
        .expect("should succeed");

    assert_eq!(result.transaction_id, "tx-abc123");
    assert_eq!(result.observed_confirmations, 3);
    assert_eq!(
        backend.verify_finality_calls.lock().unwrap().len(),
        1
    );
}

#[test]
fn mock_verifier_backend_defaults_to_pending_when_no_status_set() {
    let backend = MockVerifierBackend::bitcoin_capable();
    let request =
        TransactionFinalityRequest::new(btc_chain(), "unknown-tx", 1, false);

    let result = backend
        .backend_verify_transaction_finality(&request)
        .expect("should succeed");

    assert_eq!(result.status, TransactionFinalityStatus::Pending);
}

#[test]
fn mock_signer_records_and_returns_default_response() {
    let signer = MockSigner::bitcoin_capable();
    let request = bitcoin_sign_request();

    let response = signer.sign(&request).expect("sign should succeed");

    assert_eq!(response.address.value, "bc1qtest");
    assert_eq!(response.derivation, request.derivation);
    assert_eq!(signer.sign_calls.lock().unwrap().len(), 1);
}

#[test]
fn mock_signer_returns_queued_response() {
    let signer = MockSigner::bitcoin_capable();
    let request = bitcoin_sign_request();

    let queued = SignResponse {
        signature: btc_sig(&[0x30, 0x07, 0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x04]),
        verification_key: btc_pubkey(&[
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
        ]),
        address: ChainAddress::new(
            Chain::Bitcoin,
            AddressFormat::BitcoinBech32,
            "bc1qqueued",
        ),
        derivation: request.derivation.clone(),
    };
    signer.queue_response(queued);

    let response = signer.sign(&request).expect("sign should succeed");
    assert_eq!(response.address.value, "bc1qqueued");
}

#[test]
fn mock_verifier_backend_has_bitcoin_capability() {
    let backend = MockVerifierBackend::bitcoin_capable();
    let caps = backend.capabilities();

    assert!(caps.supports(VerifierCapability::TransactionFinality));
    assert!(caps.supports(VerifierCapability::LatestVerifiedBlock));
    assert!(caps.supports_chain(&btc_chain()));
}
