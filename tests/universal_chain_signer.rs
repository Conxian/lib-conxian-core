use lib_conxian_core::control_model::Chain;
use lib_conxian_core::signing::{
    AddressDerivationRequest, AddressDerivationResponse, AddressFormat, ChainAddress,
    ChainSigningCapability, DerivationContext, DerivationPath, DerivationPurpose,
    PublicVerificationKey, SignRequest, SignResponse, Signature, SignatureEncoding,
    SignerCapabilities, SigningAlgorithm, SigningOperation, SigningPayload, SigningTarget,
    UniversalChainSigner, VerificationRequest, VerificationResult,
    UNIVERSAL_CHAIN_SIGNER_API_VERSION,
};
use sha2::{Digest, Sha256};

struct DeterministicMockSigner {
    capabilities: SignerCapabilities,
    verification_key: PublicVerificationKey,
    address: ChainAddress,
}

impl DeterministicMockSigner {
    fn new() -> Self {
        let target = SigningTarget::for_chain(Chain::Bitcoin);
        let capabilities = SignerCapabilities::new(
            UNIVERSAL_CHAIN_SIGNER_API_VERSION,
            vec![ChainSigningCapability::new(
                target,
                vec![SigningAlgorithm::EcdsaSecp256k1],
                vec![
                    SigningOperation::SignMessage,
                    SigningOperation::DeriveAddress,
                    SigningOperation::VerifySignature,
                ],
                vec![AddressFormat::BitcoinBech32],
            )],
        );

        Self {
            capabilities,
            verification_key: PublicVerificationKey::new(
                SigningAlgorithm::EcdsaSecp256k1,
                vec![2; 33],
            ),
            address: ChainAddress::new(
                Chain::Bitcoin,
                AddressFormat::BitcoinBech32,
                "bc1qdeterministicmock",
            ),
        }
    }

    fn signature_for(&self, request: &SignRequest) -> Signature {
        let mut hasher = Sha256::new();
        hasher.update(request.payload.bytes());
        hasher.update(&self.verification_key.bytes);
        hasher.update(format!("{:?}", request.target.chain).as_bytes());
        Signature::new(
            request.algorithm,
            SignatureEncoding::Raw,
            hasher.finalize().to_vec(),
        )
    }

    fn derivation() -> DerivationContext {
        DerivationContext::new(DerivationPath::root(), DerivationPurpose::Payment)
    }
}

impl UniversalChainSigner for DeterministicMockSigner {
    fn capabilities(&self) -> &SignerCapabilities {
        &self.capabilities
    }

    fn sign_impl(
        &self,
        request: &SignRequest,
    ) -> Result<SignResponse, lib_conxian_core::signing::SigningError> {
        Ok(SignResponse {
            signature: self.signature_for(request),
            verification_key: self.verification_key.clone(),
            address: self.address.clone(),
            derivation: request.derivation.clone(),
        })
    }

    fn derive_address_impl(
        &self,
        request: &AddressDerivationRequest,
    ) -> Result<AddressDerivationResponse, lib_conxian_core::signing::SigningError> {
        Ok(AddressDerivationResponse {
            verification_key: self.verification_key.clone(),
            address: self.address.clone(),
            derivation: request.derivation.clone(),
        })
    }

    fn verify_signature_impl(
        &self,
        request: &VerificationRequest,
    ) -> Result<VerificationResult, lib_conxian_core::signing::SigningError> {
        let expected = {
            let mut hasher = Sha256::new();
            hasher.update(request.payload.bytes());
            hasher.update(&request.verification_key.bytes);
            hasher.update(format!("{:?}", request.target.chain).as_bytes());
            hasher.finalize().to_vec()
        };
        let valid = request.signature.bytes == expected;
        Ok(if valid {
            VerificationResult::valid(request.target.clone(), request.algorithm)
        } else {
            VerificationResult::invalid(request.target.clone(), request.algorithm)
        })
    }
}

fn sign_request(payload: SigningPayload) -> SignRequest {
    SignRequest::new(
        SigningTarget::for_chain(Chain::Bitcoin),
        SigningAlgorithm::EcdsaSecp256k1,
        payload,
        DeterministicMockSigner::derivation(),
    )
}

#[test]
fn supported_signing_and_address_derivation_are_complete() {
    let signer = DeterministicMockSigner::new();
    let request = sign_request(SigningPayload::message(b"bitcoin payment".to_vec()));

    let response = signer.sign(&request).expect("supported signing succeeds");
    assert_eq!(response.address.chain, Chain::Bitcoin);
    assert_eq!(response.verification_key.algorithm, request.algorithm);

    let derivation_request = AddressDerivationRequest::new(
        request.target.clone(),
        request.algorithm,
        request.derivation.clone(),
    );
    let derived = signer
        .derive_address(&derivation_request)
        .expect("supported derivation succeeds");
    assert_eq!(derived.address, response.address);
    assert_eq!(derived.verification_key, response.verification_key);
}

#[test]
fn positive_and_negative_signature_verification_are_distinct() {
    let signer = DeterministicMockSigner::new();
    let request = sign_request(SigningPayload::message(b"message-a".to_vec()));
    let response = signer.sign(&request).expect("signing succeeds");

    let verification = VerificationRequest::new(
        request.target.clone(),
        request.algorithm,
        request.payload.clone(),
        response.signature.clone(),
        response.verification_key.clone(),
        Some(response.address.clone()),
    );
    assert!(
        signer
            .verify_signature(&verification)
            .expect("verification request is valid")
            .valid
    );

    let negative = VerificationRequest::new(
        request.target,
        request.algorithm,
        SigningPayload::message(b"message-b".to_vec()),
        response.signature,
        response.verification_key,
        None,
    );
    assert!(
        !signer
            .verify_signature(&negative)
            .expect("negative verification is still a valid request")
            .valid
    );
}

#[test]
fn unsupported_chain_algorithm_and_operation_fail_closed() {
    let signer = DeterministicMockSigner::new();

    let unsupported_chain = SignRequest::new(
        SigningTarget::for_chain(Chain::Ethereum),
        SigningAlgorithm::EcdsaSecp256k1,
        SigningPayload::message(b"evm message".to_vec()),
        DeterministicMockSigner::derivation(),
    );
    assert!(matches!(
        signer.sign(&unsupported_chain),
        Err(lib_conxian_core::signing::SigningError::UnsupportedChain { .. })
    ));

    let unsupported_algorithm = SignRequest::new(
        SigningTarget::for_chain(Chain::Bitcoin),
        SigningAlgorithm::SchnorrSecp256k1,
        SigningPayload::message(b"wrong algorithm".to_vec()),
        DeterministicMockSigner::derivation(),
    );
    assert!(matches!(
        signer.sign(&unsupported_algorithm),
        Err(lib_conxian_core::signing::SigningError::UnsupportedAlgorithm { .. })
    ));

    let unsupported_operation = sign_request(SigningPayload::digest(
        lib_conxian_core::signing::DigestAlgorithm::Sha256,
        vec![0; 32],
    ));
    assert!(matches!(
        signer.sign(&unsupported_operation),
        Err(lib_conxian_core::signing::SigningError::UnsupportedOperation { .. })
    ));
}

#[test]
fn contract_models_round_trip_through_serde() {
    let request = sign_request(SigningPayload::message(b"serde".to_vec()));
    let encoded = serde_json::to_string(&request).expect("request serializes");
    let decoded: SignRequest = serde_json::from_str(&encoded).expect("request deserializes");
    assert_eq!(decoded, request);
}
