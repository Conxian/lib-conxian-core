#![allow(dead_code)]

use chrono::{DateTime, TimeZone, Utc};
use lib_conxian_core::control_model::Chain;
use lib_conxian_core::signing::{
    AddressDerivationRequest, AddressDerivationResponse, AddressFormat, ChainAddress,
    ChainSigningCapability, DerivationContext, DerivationPath, DerivationPurpose,
    PublicVerificationKey, SignRequest, SignResponse, Signature, SignatureEncoding,
    SignerCapabilities, SigningAlgorithm, SigningError, SigningOperation, UniversalChainSigner,
    VerificationRequest, VerificationResult, UNIVERSAL_CHAIN_SIGNER_API_VERSION,
};
use lib_conxian_core::verifier::{
    ChainId, LatestVerifiedBlock, ProofVerificationRequest, ProofVerificationResult,
    ProtocolVerifierBackend, ProtocolVerifierError, TransactionFinalityRequest,
    TransactionFinalityResult, VerifierCapabilities,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug, Deserialize)]
pub struct FixtureManifest {
    pub schema_version: u16,
    pub package_name: String,
    pub package_version: String,
    pub contract_versions: ContractVersions,
    pub evidence_binding_domain: String,
    pub fixtures: Vec<ManifestFixture>,
}

#[derive(Debug, Deserialize)]
pub struct ContractVersions {
    pub universal_chain_signer_api_version: u16,
    pub bip110_preflight_api_version: u16,
    pub protocol_verifier_evidence_binding_version: u8,
}

#[derive(Debug, Deserialize)]
pub struct ManifestFixture {
    pub id: String,
    pub file: String,
    pub outcome: String,
}

pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn load_fixture_value(file: &str) -> Value {
    let path = fixtures_dir().join(file);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("failed to parse fixture {}: {error}", path.display()))
}

pub fn load_fixture<T: DeserializeOwned>(file: &str) -> T {
    serde_json::from_value(load_fixture_value(file))
        .unwrap_or_else(|error| panic!("failed to decode fixture {file}: {error}"))
}

pub fn load_manifest() -> FixtureManifest {
    load_fixture("manifest.json")
}

pub fn assert_json_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let encoded = serde_json::to_value(value).expect("fixture value serializes");
    let decoded: T = serde_json::from_value(encoded).expect("fixture value deserializes");
    assert_eq!(&decoded, value);
}

pub fn assert_json_structural_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned,
{
    let encoded = serde_json::to_value(value).expect("fixture value serializes");
    let decoded: T = serde_json::from_value(encoded.clone()).expect("fixture value deserializes");
    let reencoded = serde_json::to_value(decoded).expect("decoded fixture value serializes");
    assert_eq!(reencoded, encoded);
}

pub fn fixed_now() -> DateTime<Utc> {
    Utc.timestamp_opt(2_000, 0)
        .single()
        .expect("fixture clock is valid")
}

#[derive(Debug, Clone, Copy)]
pub enum SignerResponseMode {
    Valid,
    SignatureAlgorithmMismatch,
    VerificationKeyAlgorithmMismatch,
    DerivationMismatch,
    AddressFormatUnsupported,
}

impl SignerResponseMode {
    pub fn from_wire(value: Option<&str>) -> Self {
        match value {
            Some("signature_algorithm_mismatch") => Self::SignatureAlgorithmMismatch,
            Some("verification_key_algorithm_mismatch") => Self::VerificationKeyAlgorithmMismatch,
            Some("derivation_mismatch") => Self::DerivationMismatch,
            Some("address_format_unsupported") => Self::AddressFormatUnsupported,
            Some(other) => panic!("unknown signer response mode {other}"),
            None => Self::Valid,
        }
    }
}

pub struct DeterministicSigner {
    capabilities: SignerCapabilities,
    response_mode: SignerResponseMode,
}

impl DeterministicSigner {
    pub fn new(response_mode: SignerResponseMode) -> Self {
        let target = lib_conxian_core::signing::SigningTarget::for_chain(Chain::Bitcoin);
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
            response_mode,
        }
    }

    fn verification_key() -> PublicVerificationKey {
        PublicVerificationKey::new(SigningAlgorithm::EcdsaSecp256k1, vec![2; 33])
    }

    fn address() -> ChainAddress {
        ChainAddress::new(
            Chain::Bitcoin,
            AddressFormat::BitcoinBech32,
            "bc1qdeterministicfixture",
        )
    }

    fn signature() -> Signature {
        Signature::new(
            SigningAlgorithm::EcdsaSecp256k1,
            SignatureEncoding::Raw,
            vec![42; 32],
        )
    }
}

impl UniversalChainSigner for DeterministicSigner {
    fn capabilities(&self) -> &SignerCapabilities {
        &self.capabilities
    }

    fn sign_impl(&self, request: &SignRequest) -> Result<SignResponse, SigningError> {
        let mut signature = Self::signature();
        let mut verification_key = Self::verification_key();
        let mut address = Self::address();
        let mut derivation = request.derivation.clone();

        match self.response_mode {
            SignerResponseMode::Valid => {}
            SignerResponseMode::SignatureAlgorithmMismatch => {
                signature.algorithm = SigningAlgorithm::SchnorrSecp256k1;
            }
            SignerResponseMode::VerificationKeyAlgorithmMismatch => {
                verification_key.algorithm = SigningAlgorithm::SchnorrSecp256k1;
            }
            SignerResponseMode::DerivationMismatch => {
                derivation =
                    DerivationContext::new(DerivationPath::root(), DerivationPurpose::Change);
            }
            SignerResponseMode::AddressFormatUnsupported => {
                address.format = AddressFormat::BitcoinBase58;
            }
        }

        Ok(SignResponse {
            signature,
            verification_key,
            address,
            derivation,
        })
    }

    fn derive_address_impl(
        &self,
        request: &AddressDerivationRequest,
    ) -> Result<AddressDerivationResponse, SigningError> {
        Ok(AddressDerivationResponse {
            verification_key: Self::verification_key(),
            address: Self::address(),
            derivation: request.derivation.clone(),
        })
    }

    fn verify_signature_impl(
        &self,
        request: &VerificationRequest,
    ) -> Result<VerificationResult, SigningError> {
        let valid = request.signature.bytes == Self::signature().bytes;
        Ok(if valid {
            VerificationResult::valid(request.target.clone(), request.algorithm)
        } else {
            VerificationResult::invalid(request.target.clone(), request.algorithm)
        })
    }
}

#[derive(Clone)]
pub struct DeterministicVerifierBackend {
    capabilities: VerifierCapabilities,
    state_result: Option<ProofVerificationResult>,
    latest_block: Option<LatestVerifiedBlock>,
    finality_result: Option<TransactionFinalityResult>,
    state_calls: Arc<AtomicUsize>,
    latest_calls: Arc<AtomicUsize>,
    finality_calls: Arc<AtomicUsize>,
}

impl DeterministicVerifierBackend {
    pub fn new(
        capabilities: VerifierCapabilities,
        state_result: Option<ProofVerificationResult>,
        finality_result: Option<TransactionFinalityResult>,
    ) -> Self {
        let latest_block = state_result
            .as_ref()
            .map(|result| result.verified_block.clone())
            .or_else(|| {
                finality_result
                    .as_ref()
                    .and_then(|result| result.latest_block.clone())
            });
        Self {
            capabilities,
            state_result,
            latest_block,
            finality_result,
            state_calls: Arc::new(AtomicUsize::new(0)),
            latest_calls: Arc::new(AtomicUsize::new(0)),
            finality_calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn state_calls(&self) -> usize {
        self.state_calls.load(Ordering::SeqCst)
    }

    pub fn latest_calls(&self) -> usize {
        self.latest_calls.load(Ordering::SeqCst)
    }

    pub fn finality_calls(&self) -> usize {
        self.finality_calls.load(Ordering::SeqCst)
    }
}

impl ProtocolVerifierBackend for DeterministicVerifierBackend {
    fn capabilities(&self) -> &VerifierCapabilities {
        &self.capabilities
    }

    fn backend_verify_chain_state(
        &self,
        _request: &ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, ProtocolVerifierError> {
        self.state_calls.fetch_add(1, Ordering::SeqCst);
        self.state_result
            .clone()
            .ok_or_else(|| ProtocolVerifierError::UnavailableEvidence {
                reference: "fixture-state-result".to_string(),
            })
    }

    fn backend_get_latest_verified_block(
        &self,
        _chain: &ChainId,
    ) -> Result<LatestVerifiedBlock, ProtocolVerifierError> {
        self.latest_calls.fetch_add(1, Ordering::SeqCst);
        self.latest_block
            .clone()
            .ok_or_else(|| ProtocolVerifierError::UnavailableEvidence {
                reference: "fixture-latest-block".to_string(),
            })
    }

    fn backend_verify_transaction_finality(
        &self,
        _request: &TransactionFinalityRequest,
    ) -> Result<TransactionFinalityResult, ProtocolVerifierError> {
        self.finality_calls.fetch_add(1, Ordering::SeqCst);
        self.finality_result
            .clone()
            .ok_or_else(|| ProtocolVerifierError::UnavailableEvidence {
                reference: "fixture-finality-result".to_string(),
            })
    }
}
