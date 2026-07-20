use chrono::{DateTime, TimeZone, Utc};
use lib_conxian_core::control_model::{
    ChainFamily, FinalityClass, TrustTier, VerificationClass, VerificationStatus,
};
use lib_conxian_core::verifier::{
    validate_finality_transition, BlockHeader, ChainId, ChainStateReference, LatestVerifiedBlock,
    ProofData, ProofFormat, ProofVerificationRequest, ProofVerificationResult, ProtocolVerifier,
    ProtocolVerifierError, TransactionFinalityRequest, TransactionFinalityResult,
    TransactionFinalityStatus, VerificationProvenance, VerifierCapabilities, VerifierCapability,
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

#[derive(Clone)]
struct MockVerifier {
    capabilities: VerifierCapabilities,
}

impl MockVerifier {
    fn new(capabilities: Vec<VerifierCapability>) -> Self {
        Self {
            capabilities: VerifierCapabilities {
                verifier_id: "integration-mock".to_string(),
                version: "1".to_string(),
                supported_chains: vec![bitcoin()],
                supported_families: vec![ChainFamily::BitcoinUtxo],
                capabilities,
                proof_formats: vec![ProofFormat::HeaderChain, ProofFormat::Merkle],
                verification_classes: vec![VerificationClass::LightClient],
                finality_classes: vec![FinalityClass::Probabilistic],
                trust_tiers: vec![TrustTier::Strict],
            },
        }
    }

    fn block(&self, chain: ChainId) -> LatestVerifiedBlock {
        LatestVerifiedBlock {
            chain,
            header: BlockHeader {
                hash: "block-hash".to_string(),
                parent_hash: Some("parent-hash".to_string()),
                height: 100,
                timestamp: timestamp(1_000),
                state_root: Some("state-root".to_string()),
            },
            finality_class: FinalityClass::Probabilistic,
            confirmations: 6,
            verification_class: VerificationClass::LightClient,
            trust_tier: TrustTier::Strict,
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: self.capabilities.verifier_id.clone(),
                evidence_ref: Some("mock-proof".to_string()),
                verified_at: timestamp(1_010),
            },
        }
    }
}

impl ProtocolVerifier for MockVerifier {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn verify_chain_state(
        &self,
        request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.ensure_capability(&request.chain, VerifierCapability::StateProofVerification)?;
        request.validate()?;
        self.ensure_proof_format(&request.chain, &request.proof.format)?;

        if request.proof.bytes == [0] {
            return Err(ProtocolVerifierError::InvalidProof {
                reason: "mock proof marker is invalid".to_string(),
            });
        }

        let result = ProofVerificationResult {
            chain: request.chain.clone(),
            state: request.state.clone(),
            proof_format: request.proof.format.clone(),
            verified_block: self.block(request.chain.clone()),
        };
        result.validate()?;
        Ok(result)
    }

    fn get_latest_verified_block(
        &self,
        chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.ensure_capability(chain, VerifierCapability::LatestVerifiedBlock)?;
        let block = self.block(chain.clone());
        block.validate()?;
        Ok(block)
    }

    fn verify_transaction_finality(
        &self,
        request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.ensure_capability(&request.chain, VerifierCapability::TransactionFinality)?;
        request.validate()?;

        let observed_confirmations = 6;
        let status = if request.min_confirmations <= observed_confirmations {
            TransactionFinalityStatus::Finalized {
                confirmations: observed_confirmations,
            }
        } else {
            TransactionFinalityStatus::Confirmed {
                confirmations: observed_confirmations,
            }
        };
        let result = TransactionFinalityResult {
            chain: request.chain.clone(),
            transaction_id: request.transaction_id.clone(),
            status,
            finality_class: FinalityClass::Probabilistic,
            required_confirmations: request.min_confirmations,
            observed_confirmations,
            latest_block: Some(self.block(request.chain.clone())),
            verification_class: VerificationClass::LightClient,
            trust_tier: TrustTier::Strict,
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: self.capabilities.verifier_id.clone(),
                evidence_ref: Some("mock-finality".to_string()),
                verified_at: timestamp(1_010),
            },
        };
        self.validate_finality_result(request, &result)?;
        Ok(result)
    }
}

#[test]
fn verifier_accepts_valid_proof_and_round_trips_result() {
    let verifier = MockVerifier::new(vec![
        VerifierCapability::StateProofVerification,
        VerifierCapability::LatestVerifiedBlock,
        VerifierCapability::TransactionFinality,
    ]);
    let request = ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, Some("state-root".to_string())),
        ProofData::new(ProofFormat::HeaderChain, vec![1, 2, 3]),
    );

    let result = verifier.verify_chain_state(&request).expect("valid proof");
    assert!(result.is_verified());

    let encoded = serde_json::to_vec(&result).expect("serialize result");
    let decoded: ProofVerificationResult =
        serde_json::from_slice(&encoded).expect("deserialize result");
    assert_eq!(decoded, result);
}

#[test]
fn verifier_rejects_malformed_and_invalid_proofs() {
    let verifier = MockVerifier::new(vec![VerifierCapability::StateProofVerification]);
    let malformed = ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::Merkle, Vec::new()),
    );
    assert!(matches!(
        verifier.verify_chain_state(&malformed),
        Err(ProtocolVerifierError::InsufficientProofData { .. })
    ));

    let invalid = ProofVerificationRequest::new(
        bitcoin(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::Merkle, vec![0]),
    );
    assert!(matches!(
        verifier.verify_chain_state(&invalid),
        Err(ProtocolVerifierError::InvalidProof { .. })
    ));
}

#[test]
fn verifier_rejects_unsupported_chain_and_capability() {
    let block_only = MockVerifier::new(vec![VerifierCapability::LatestVerifiedBlock]);
    let state_request = ProofVerificationRequest::new(
        ethereum(),
        ChainStateReference::new("block-hash", 100, None),
        ProofData::new(ProofFormat::HeaderChain, vec![1]),
    );
    assert!(matches!(
        block_only.verify_chain_state(&state_request),
        Err(ProtocolVerifierError::UnsupportedChain { .. })
    ));

    let bitcoin_request = TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, false);
    assert!(matches!(
        block_only.verify_transaction_finality(&bitcoin_request),
        Err(ProtocolVerifierError::UnsupportedCapability { .. })
    ));
}

#[test]
fn verifier_returns_latest_block_and_finality_transitions() {
    let verifier = MockVerifier::new(vec![
        VerifierCapability::LatestVerifiedBlock,
        VerifierCapability::TransactionFinality,
    ]);
    let latest = verifier
        .get_latest_verified_block(&bitcoin())
        .expect("latest verified block");
    assert_eq!(latest.header.height, 100);
    assert!(latest.is_verified());

    let pending = TransactionFinalityStatus::Pending;
    let confirmed = TransactionFinalityStatus::Confirmed { confirmations: 3 };
    let finalized = TransactionFinalityStatus::Finalized { confirmations: 6 };
    assert!(validate_finality_transition(&pending, &confirmed).is_ok());
    assert!(validate_finality_transition(&confirmed, &finalized).is_ok());
    assert!(validate_finality_transition(&finalized, &pending).is_err());

    let result = verifier
        .verify_transaction_finality(&TransactionFinalityRequest::new(bitcoin(), "tx-1", 6, true))
        .expect("finalized transaction");
    assert!(result.is_final());
}

#[test]
fn capability_and_result_validation_fail_closed_on_trust_invariants() {
    let mut capabilities = MockVerifier::new(vec![VerifierCapability::LatestVerifiedBlock]);
    capabilities.capabilities.verification_classes = vec![VerificationClass::ExternalQuorum];
    assert!(matches!(
        capabilities.capabilities.validate(),
        Err(ProtocolVerifierError::InvariantViolation { .. })
    ));

    let mut result =
        MockVerifier::new(vec![VerifierCapability::LatestVerifiedBlock]).block(bitcoin());
    result.trust_tier = TrustTier::ObserverOnly;
    assert!(matches!(
        result.validate(),
        Err(ProtocolVerifierError::PolicyBlocked { .. })
    ));
}
