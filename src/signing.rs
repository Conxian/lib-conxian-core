//! Platform-neutral universal chain signing contracts.
//!
//! [`UniversalChainSigner`] describes the protocol-facing boundary between core
//! and concrete signer implementations. It contains no private-key material,
//! network clients, persistence, or hardware-specific behavior. A signer must
//! advertise its capabilities and the provided trait methods reject unsupported
//! chains, algorithms, and operations before invoking implementation hooks.
//!
//! # Example
//!
//! ```
//! use lib_conxian_core::control_model::{Chain, ChainFamily};
//! use lib_conxian_core::signing::{
//!     ChainSigningCapability, DerivationContext, DerivationPath, DerivationPurpose,
//!     SignerCapabilities, SigningAlgorithm, SigningOperation, SigningPayload, SigningTarget,
//!     UNIVERSAL_CHAIN_SIGNER_API_VERSION,
//! };
//!
//! let target = SigningTarget::for_chain(Chain::Bitcoin);
//! let capability = ChainSigningCapability::new(
//!     target.clone(),
//!     vec![SigningAlgorithm::SchnorrSecp256k1],
//!     vec![SigningOperation::SignMessage],
//!     vec![],
//! );
//! let capabilities = SignerCapabilities::new(
//!     UNIVERSAL_CHAIN_SIGNER_API_VERSION,
//!     vec![capability],
//! );
//! let request = lib_conxian_core::signing::SignRequest::new(
//!     target,
//!     SigningAlgorithm::SchnorrSecp256k1,
//!     SigningPayload::message(b"hello bitcoin".to_vec()),
//!     DerivationContext::new(
//!         DerivationPath::root(),
//!         DerivationPurpose::MessageSigning,
//!     ),
//! );
//!
//! assert!(capabilities
//!     .require(
//!         &request.target,
//!         request.algorithm,
//!         request.operation(),
//!     )
//!     .is_ok());
//! assert_eq!(request.target.family, ChainFamily::BitcoinUtxo);
//! ```

use crate::control_model::{chain_family_for, Chain, ChainFamily};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Version of the platform-neutral signing contract and DTOs.
pub const UNIVERSAL_CHAIN_SIGNER_API_VERSION: u16 = 1;

/// Algorithms that a concrete signer may advertise.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningAlgorithm {
    /// ECDSA over secp256k1, used by Bitcoin-family and EVM adapters.
    EcdsaSecp256k1,
    /// Schnorr over secp256k1, including Bitcoin-native signing flows.
    SchnorrSecp256k1,
    /// Ed25519, used by Solana-family adapters.
    Ed25519,
}

/// Digest encodings accepted by [`SigningPayload::Digest`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
    Sha512,
    Keccak256,
    Blake2b256,
}

impl DigestAlgorithm {
    /// Returns the canonical output length for this digest algorithm.
    pub const fn output_len(self) -> usize {
        match self {
            Self::Sha256 | Self::Keccak256 | Self::Blake2b256 => 32,
            Self::Sha512 => 64,
        }
    }
}

/// Encoding of the signature bytes returned by a signer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SignatureEncoding {
    /// A fixed-size or algorithm-defined raw signature.
    Raw,
    /// DER-encoded ECDSA signature.
    Der,
    /// Fixed-size compact signature, such as a 64-byte Schnorr signature.
    Compact,
    /// Recoverable signature containing its recovery identifier.
    Recoverable,
    /// A chain-specific encoding owned by the concrete adapter.
    ChainSpecific,
}

/// Address representation formats understood by the core contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AddressFormat {
    // ── Bitcoin UTXO ──
    BitcoinBase58,
    BitcoinBech32,
    // ── Bitcoin L2 families ──
    StacksC32,
    RootstockBase58,
    LiquidConfidential,
    StatechainPubkey,
    ArkVtxo,
    BPoSBech32,
    FederationAddress,
    MergeMinedBase58,
    AnchorC32,
    RollupEvmHex,
    AltRollupHex,
    AltLayer1Hex,
    CsvContractId,
    HybridAddress,
    // ── Cross-ecosystem ──
    EvmHex,
    SolanaBase58,
    CosmosBech32,
    MoveHex,
    SubstrateSs58,
    /// An opaque, non-empty address whose syntax is owned by an adapter.
    Generic,
}

impl AddressFormat {
    /// Returns whether this format is suitable for the requested signing target.
    ///
    /// This is a family/format guard, not a full address parser. Concrete SDK
    /// adapters remain responsible for chain-specific checksum and network
    /// validation.
    pub fn is_compatible_with(self, target: &SigningTarget) -> bool {
        match self {
            // ── Bitcoin L1 ──
            Self::BitcoinBase58 | Self::BitcoinBech32 => {
                target.family == ChainFamily::BitcoinUtxo
                    && !matches!(target.chain, Chain::Stacks | Chain::Rootstock)
            }
            // ── Bitcoin L2 ──
            Self::StacksC32 | Self::AnchorC32 => {
                matches!(
                    target.family,
                    ChainFamily::Anchor | ChainFamily::BitcoinUtxo
                ) && target.chain == Chain::Stacks
            }
            Self::RootstockBase58 | Self::MergeMinedBase58 => {
                target.family == ChainFamily::MergeMined
            }
            Self::LiquidConfidential | Self::FederationAddress => {
                target.family == ChainFamily::Federation
            }
            Self::StatechainPubkey => target.family == ChainFamily::Statechain,
            Self::ArkVtxo => target.family == ChainFamily::Ark,
            Self::BPoSBech32 => target.family == ChainFamily::BPoS,
            Self::RollupEvmHex => target.family == ChainFamily::Rollup,
            Self::AltRollupHex => target.family == ChainFamily::AltRollup,
            Self::AltLayer1Hex => target.family == ChainFamily::AltLayer1,
            Self::CsvContractId => target.family == ChainFamily::Csv,
            Self::HybridAddress => target.family == ChainFamily::Hybrid,
            // ── Cross-ecosystem ──
            Self::EvmHex => target.family == ChainFamily::Evm,
            Self::SolanaBase58 => target.family == ChainFamily::SolanaSvm,
            Self::CosmosBech32 => target.family == ChainFamily::CosmosIbc,
            Self::MoveHex => target.family == ChainFamily::Move,
            Self::SubstrateSs58 => target.family == ChainFamily::Substrate,
            Self::Generic => true,
        }
    }
}

/// Operations that a signer can explicitly advertise.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningOperation {
    SignMessage,
    SignDigest,
    DeriveAddress,
    VerifySignature,
}

/// A chain and its canonical family, carried together to prevent ambiguous
/// cross-chain requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SigningTarget {
    pub chain: Chain,
    pub family: ChainFamily,
}

impl SigningTarget {
    /// Creates an explicit target from a chain and family.
    pub const fn new(chain: Chain, family: ChainFamily) -> Self {
        Self { chain, family }
    }

    /// Creates a target using the core's existing chain-family mapping.
    pub fn for_chain(chain: Chain) -> Self {
        let family = chain_family_for(&chain);
        Self { chain, family }
    }

    /// Rejects an inconsistent chain/family pair before capability lookup.
    pub fn validate(&self) -> Result<(), SigningError> {
        let expected_family = chain_family_for(&self.chain);
        if expected_family != self.family {
            return Err(SigningError::InvalidTarget {
                chain: self.chain.clone(),
                expected_family,
                provided_family: self.family.clone(),
            });
        }
        Ok(())
    }
}

/// A single derivation index. This carries path metadata only; it never carries
/// a seed, private key, share, or key handle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivationIndex {
    pub index: u32,
    pub hardened: bool,
}

impl DerivationIndex {
    pub const fn new(index: u32, hardened: bool) -> Self {
        Self { index, hardened }
    }
}

/// Structured derivation path metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivationPath {
    pub components: Vec<DerivationIndex>,
}

impl DerivationPath {
    /// Creates a path from structured components.
    pub fn new(components: Vec<DerivationIndex>) -> Self {
        Self { components }
    }

    /// Returns the root path.
    pub fn root() -> Self {
        Self::default()
    }

    fn validate(&self) -> Result<(), SigningError> {
        if self.components.len() > 255 {
            return Err(SigningError::InvalidDerivationPath(
                DerivationPathError::TooManyComponents,
            ));
        }
        Ok(())
    }
}

/// Purpose metadata for a derivation request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DerivationPurpose {
    Payment,
    Change,
    Identity,
    Staking,
    Contract,
    MessageSigning,
}

/// Explicit derivation context without any secret key material.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DerivationContext {
    pub path: DerivationPath,
    pub purpose: DerivationPurpose,
}

impl DerivationContext {
    pub const fn new(path: DerivationPath, purpose: DerivationPurpose) -> Self {
        Self { path, purpose }
    }

    fn validate(&self) -> Result<(), SigningError> {
        self.path.validate()
    }
}

/// Explicit bytes to sign. A caller must choose either a message or a
/// precomputed digest; no hashing or chain-domain encoding is implicit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningPayload {
    Message {
        bytes: Vec<u8>,
    },
    Digest {
        algorithm: DigestAlgorithm,
        bytes: Vec<u8>,
    },
}

impl SigningPayload {
    pub fn message(bytes: impl Into<Vec<u8>>) -> Self {
        Self::Message {
            bytes: bytes.into(),
        }
    }

    pub fn digest(algorithm: DigestAlgorithm, bytes: impl Into<Vec<u8>>) -> Self {
        Self::Digest {
            algorithm,
            bytes: bytes.into(),
        }
    }

    pub const fn operation(&self) -> SigningOperation {
        match self {
            Self::Message { .. } => SigningOperation::SignMessage,
            Self::Digest { .. } => SigningOperation::SignDigest,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Message { bytes } | Self::Digest { bytes, .. } => bytes,
        }
    }

    fn validate(&self) -> Result<(), SigningError> {
        match self {
            Self::Message { bytes } if bytes.is_empty() => {
                Err(SigningError::InvalidPayload(PayloadError::EmptyMessage))
            }
            Self::Message { .. } => Ok(()),
            Self::Digest { algorithm, bytes } if bytes.len() != algorithm.output_len() => Err(
                SigningError::InvalidPayload(PayloadError::InvalidDigestLength {
                    algorithm: *algorithm,
                    expected: algorithm.output_len(),
                    actual: bytes.len(),
                }),
            ),
            Self::Digest { .. } => Ok(()),
        }
    }
}

/// Request to sign an explicit message or digest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignRequest {
    pub target: SigningTarget,
    pub algorithm: SigningAlgorithm,
    pub payload: SigningPayload,
    pub derivation: DerivationContext,
}

impl SignRequest {
    pub fn new(
        target: SigningTarget,
        algorithm: SigningAlgorithm,
        payload: SigningPayload,
        derivation: DerivationContext,
    ) -> Self {
        Self {
            target,
            algorithm,
            payload,
            derivation,
        }
    }

    pub const fn operation(&self) -> SigningOperation {
        self.payload.operation()
    }

    fn validate(&self) -> Result<(), SigningError> {
        self.target.validate()?;
        self.payload.validate()?;
        self.derivation.validate()
    }
}

/// Public verification key returned by, or supplied to, a signer.
///
/// The bytes are public material only. This type intentionally has no field or
/// conversion for private keys, seeds, shares, or enclave handles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicVerificationKey {
    pub algorithm: SigningAlgorithm,
    pub bytes: Vec<u8>,
}

impl PublicVerificationKey {
    pub fn new(algorithm: SigningAlgorithm, bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            algorithm,
            bytes: bytes.into(),
        }
    }

    fn validate(&self) -> Result<(), SigningError> {
        let valid_len = match self.algorithm {
            SigningAlgorithm::EcdsaSecp256k1 => matches!(self.bytes.len(), 33 | 65),
            SigningAlgorithm::SchnorrSecp256k1 => matches!(self.bytes.len(), 32 | 33 | 65),
            SigningAlgorithm::Ed25519 => self.bytes.len() == 32,
        };

        if valid_len {
            Ok(())
        } else {
            Err(SigningError::InvalidVerificationKey)
        }
    }
}

/// Signature bytes and their declared algorithm/encoding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Signature {
    pub algorithm: SigningAlgorithm,
    pub encoding: SignatureEncoding,
    pub bytes: Vec<u8>,
}

impl Signature {
    pub fn new(
        algorithm: SigningAlgorithm,
        encoding: SignatureEncoding,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            algorithm,
            encoding,
            bytes: bytes.into(),
        }
    }

    fn validate(&self) -> Result<(), SigningError> {
        let valid = match self.encoding {
            SignatureEncoding::Raw | SignatureEncoding::ChainSpecific => !self.bytes.is_empty(),
            SignatureEncoding::Der => (8..=72).contains(&self.bytes.len()),
            SignatureEncoding::Compact => self.bytes.len() == 64,
            SignatureEncoding::Recoverable => self.bytes.len() == 65,
        };

        if valid {
            Ok(())
        } else {
            Err(SigningError::InvalidSignature)
        }
    }
}

/// Canonical chain address returned by address derivation or attached to a
/// verification request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainAddress {
    pub chain: Chain,
    pub format: AddressFormat,
    pub value: String,
}

impl ChainAddress {
    pub fn new(chain: Chain, format: AddressFormat, value: impl Into<String>) -> Self {
        Self {
            chain,
            format,
            value: value.into(),
        }
    }

    fn validate_for(&self, target: &SigningTarget) -> Result<(), SigningError> {
        if self.value.trim().is_empty() {
            return Err(SigningError::InvalidAddress(AddressError::Empty));
        }
        if self.chain != target.chain {
            return Err(SigningError::InvalidAddress(AddressError::ChainMismatch));
        }
        if !self.format.is_compatible_with(target) {
            return Err(SigningError::InvalidAddress(AddressError::FormatMismatch));
        }
        Ok(())
    }
}

/// Successful signing output. It includes only public verification metadata;
/// private key material remains inside the concrete signer implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignResponse {
    pub signature: Signature,
    pub verification_key: PublicVerificationKey,
    pub address: ChainAddress,
    pub derivation: DerivationContext,
}

impl SignResponse {
    fn validate_for(
        &self,
        request: &SignRequest,
        capability: &ChainSigningCapability,
    ) -> Result<(), SigningError> {
        self.signature.validate()?;
        self.verification_key.validate()?;
        self.address.validate_for(&request.target)?;
        self.derivation.validate()?;

        if self.signature.algorithm != request.algorithm {
            return Err(SigningError::InvalidResponse(
                ResponseError::SignatureAlgorithmMismatch,
            ));
        }
        if self.verification_key.algorithm != request.algorithm {
            return Err(SigningError::InvalidResponse(
                ResponseError::VerificationKeyAlgorithmMismatch,
            ));
        }
        if self.derivation != request.derivation {
            return Err(SigningError::InvalidResponse(
                ResponseError::DerivationMismatch,
            ));
        }
        if !capability.supports_address_format(self.address.format) {
            return Err(SigningError::InvalidResponse(
                ResponseError::AddressFormatUnsupported,
            ));
        }
        Ok(())
    }
}

/// Request to derive a public address for an explicit chain and derivation
/// context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressDerivationRequest {
    pub target: SigningTarget,
    pub algorithm: SigningAlgorithm,
    pub derivation: DerivationContext,
}

impl AddressDerivationRequest {
    pub const fn new(
        target: SigningTarget,
        algorithm: SigningAlgorithm,
        derivation: DerivationContext,
    ) -> Self {
        Self {
            target,
            algorithm,
            derivation,
        }
    }

    fn validate(&self) -> Result<(), SigningError> {
        self.target.validate()?;
        self.derivation.validate()
    }
}

/// Address derivation output with the public key needed for later verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressDerivationResponse {
    pub verification_key: PublicVerificationKey,
    pub address: ChainAddress,
    pub derivation: DerivationContext,
}

impl AddressDerivationResponse {
    fn validate_for(
        &self,
        request: &AddressDerivationRequest,
        capability: &ChainSigningCapability,
    ) -> Result<(), SigningError> {
        self.verification_key.validate()?;
        self.address.validate_for(&request.target)?;
        self.derivation.validate()?;

        if self.verification_key.algorithm != request.algorithm {
            return Err(SigningError::InvalidResponse(
                ResponseError::VerificationKeyAlgorithmMismatch,
            ));
        }
        if self.derivation != request.derivation {
            return Err(SigningError::InvalidResponse(
                ResponseError::DerivationMismatch,
            ));
        }
        if !capability.supports_address_format(self.address.format) {
            return Err(SigningError::InvalidResponse(
                ResponseError::AddressFormatUnsupported,
            ));
        }
        Ok(())
    }
}

/// Complete signature-verification input. The payload, signature, and public
/// verification key are all required; an address may additionally bind the
/// verification to an expected chain address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationRequest {
    pub target: SigningTarget,
    pub algorithm: SigningAlgorithm,
    pub payload: SigningPayload,
    pub signature: Signature,
    pub verification_key: PublicVerificationKey,
    pub address: Option<ChainAddress>,
}

impl VerificationRequest {
    pub fn new(
        target: SigningTarget,
        algorithm: SigningAlgorithm,
        payload: SigningPayload,
        signature: Signature,
        verification_key: PublicVerificationKey,
        address: Option<ChainAddress>,
    ) -> Self {
        Self {
            target,
            algorithm,
            payload,
            signature,
            verification_key,
            address,
        }
    }

    fn validate(&self) -> Result<(), SigningError> {
        self.target.validate()?;
        self.payload.validate()?;
        self.signature.validate()?;
        self.verification_key.validate()?;

        if self.signature.algorithm != self.algorithm {
            return Err(SigningError::InvalidRequest(
                RequestError::SignatureAlgorithmMismatch,
            ));
        }
        if self.verification_key.algorithm != self.algorithm {
            return Err(SigningError::InvalidRequest(
                RequestError::VerificationKeyAlgorithmMismatch,
            ));
        }
        if let Some(address) = &self.address {
            address.validate_for(&self.target)?;
        }
        Ok(())
    }

    pub const fn operation(&self) -> SigningOperation {
        SigningOperation::VerifySignature
    }
}

/// Verification result. An invalid signature is a valid negative result;
/// malformed or unsupported requests are returned as [`SigningError`] instead.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationResult {
    pub valid: bool,
    pub target: SigningTarget,
    pub algorithm: SigningAlgorithm,
}

impl VerificationResult {
    pub const fn valid(target: SigningTarget, algorithm: SigningAlgorithm) -> Self {
        Self {
            valid: true,
            target,
            algorithm,
        }
    }

    pub const fn invalid(target: SigningTarget, algorithm: SigningAlgorithm) -> Self {
        Self {
            valid: false,
            target,
            algorithm,
        }
    }

    fn validate_for(&self, request: &VerificationRequest) -> Result<(), SigningError> {
        if self.target != request.target || self.algorithm != request.algorithm {
            return Err(SigningError::InvalidResponse(
                ResponseError::VerificationMetadataMismatch,
            ));
        }
        Ok(())
    }
}

/// Capability declaration for one chain/family target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainSigningCapability {
    pub target: SigningTarget,
    pub algorithms: Vec<SigningAlgorithm>,
    pub operations: Vec<SigningOperation>,
    pub address_formats: Vec<AddressFormat>,
}

impl ChainSigningCapability {
    pub fn new(
        target: SigningTarget,
        algorithms: Vec<SigningAlgorithm>,
        operations: Vec<SigningOperation>,
        address_formats: Vec<AddressFormat>,
    ) -> Self {
        Self {
            target,
            algorithms,
            operations,
            address_formats,
        }
    }

    pub fn supports_algorithm(&self, algorithm: SigningAlgorithm) -> bool {
        self.algorithms.contains(&algorithm)
    }

    pub fn supports_operation(&self, operation: SigningOperation) -> bool {
        self.operations.contains(&operation)
    }

    pub fn supports_address_format(&self, format: AddressFormat) -> bool {
        self.address_formats.contains(&format)
    }
}

/// Versioned capability discovery response for a concrete signer.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignerCapabilities {
    pub api_version: u16,
    pub chains: Vec<ChainSigningCapability>,
}

impl SignerCapabilities {
    pub fn new(api_version: u16, chains: Vec<ChainSigningCapability>) -> Self {
        Self {
            api_version,
            chains,
        }
    }

    pub fn capability_for(&self, target: &SigningTarget) -> Option<&ChainSigningCapability> {
        self.chains
            .iter()
            .find(|capability| capability.target == *target)
    }

    /// Checks all capability dimensions and returns the matching declaration.
    /// This is the fail-closed gate used by [`UniversalChainSigner`].
    pub fn require(
        &self,
        target: &SigningTarget,
        algorithm: SigningAlgorithm,
        operation: SigningOperation,
    ) -> Result<&ChainSigningCapability, SigningError> {
        target.validate()?;

        let capability =
            self.capability_for(target)
                .ok_or_else(|| SigningError::UnsupportedChain {
                    chain: target.chain.clone(),
                    family: target.family.clone(),
                })?;

        if !capability.supports_algorithm(algorithm) {
            return Err(SigningError::UnsupportedAlgorithm {
                chain: target.chain.clone(),
                algorithm,
            });
        }
        if !capability.supports_operation(operation) {
            return Err(SigningError::UnsupportedOperation {
                chain: target.chain.clone(),
                operation,
            });
        }
        Ok(capability)
    }
}

/// Structured payload validation failures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PayloadError {
    EmptyMessage,
    InvalidDigestLength {
        algorithm: DigestAlgorithm,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessage => write!(f, "signing message must not be empty"),
            Self::InvalidDigestLength {
                algorithm,
                expected,
                actual,
            } => write!(
                f,
                "digest length {actual} does not match {algorithm:?} length {expected}"
            ),
        }
    }
}

impl std::error::Error for PayloadError {}

/// Structured derivation-path validation failures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DerivationPathError {
    TooManyComponents,
}

impl fmt::Display for DerivationPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyComponents => write!(f, "derivation path has too many components"),
        }
    }
}

impl std::error::Error for DerivationPathError {}

/// Structured address validation failures. Address values are never included
/// in these errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AddressError {
    Empty,
    ChainMismatch,
    FormatMismatch,
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "address must not be empty"),
            Self::ChainMismatch => write!(f, "address chain does not match signing target"),
            Self::FormatMismatch => write!(f, "address format does not match signing target"),
        }
    }
}

impl std::error::Error for AddressError {}

/// Structured errors for malformed signing requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestError {
    SignatureAlgorithmMismatch,
    VerificationKeyAlgorithmMismatch,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignatureAlgorithmMismatch => write!(f, "signature algorithm mismatch"),
            Self::VerificationKeyAlgorithmMismatch => {
                write!(f, "verification key algorithm mismatch")
            }
        }
    }
}

impl std::error::Error for RequestError {}

/// Structured errors for malformed implementation responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseError {
    SignatureAlgorithmMismatch,
    VerificationKeyAlgorithmMismatch,
    AddressFormatUnsupported,
    DerivationMismatch,
    VerificationMetadataMismatch,
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignatureAlgorithmMismatch => write!(f, "signature algorithm mismatch"),
            Self::VerificationKeyAlgorithmMismatch => {
                write!(f, "verification key algorithm mismatch")
            }
            Self::AddressFormatUnsupported => write!(f, "response address format is unsupported"),
            Self::DerivationMismatch => write!(f, "response derivation context mismatch"),
            Self::VerificationMetadataMismatch => {
                write!(f, "verification result metadata mismatch")
            }
        }
    }
}

impl std::error::Error for ResponseError {}

/// Secret-safe error taxonomy for universal signing operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SigningError {
    InvalidTarget {
        chain: Chain,
        expected_family: ChainFamily,
        provided_family: ChainFamily,
    },
    UnsupportedChain {
        chain: Chain,
        family: ChainFamily,
    },
    UnsupportedAlgorithm {
        chain: Chain,
        algorithm: SigningAlgorithm,
    },
    UnsupportedOperation {
        chain: Chain,
        operation: SigningOperation,
    },
    InvalidPayload(PayloadError),
    InvalidDerivationPath(DerivationPathError),
    InvalidAddress(AddressError),
    InvalidRequest(RequestError),
    InvalidSignature,
    InvalidVerificationKey,
    InvalidResponse(ResponseError),
    BackendFailure,
}

impl fmt::Display for SigningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget {
                chain,
                expected_family,
                provided_family,
            } => write!(
                f,
                "invalid signing target for {chain:?}: expected {expected_family:?}, got {provided_family:?}"
            ),
            Self::UnsupportedChain { chain, family } => {
                write!(f, "signer does not support {chain:?} in {family:?}")
            }
            Self::UnsupportedAlgorithm { chain, algorithm } => {
                write!(f, "signer does not support {algorithm:?} for {chain:?}")
            }
            Self::UnsupportedOperation { chain, operation } => {
                write!(f, "signer does not support {operation:?} for {chain:?}")
            }
            Self::InvalidPayload(error) => write!(f, "invalid signing payload: {error}"),
            Self::InvalidDerivationPath(error) => {
                write!(f, "invalid derivation path: {error}")
            }
            Self::InvalidAddress(error) => write!(f, "invalid signing address: {error}"),
            Self::InvalidRequest(error) => write!(f, "invalid signing request: {error}"),
            Self::InvalidSignature => write!(f, "invalid signature"),
            Self::InvalidVerificationKey => write!(f, "invalid public verification key"),
            Self::InvalidResponse(error) => write!(f, "invalid signer response: {error}"),
            Self::BackendFailure => write!(f, "signer backend failed without exposing details"),
        }
    }
}

impl std::error::Error for SigningError {}

/// Platform-neutral signing contract for SDK and Gateway adapters.
pub trait UniversalChainSigner {
    /// Returns a versioned, explicit declaration of supported chains,
    /// algorithms, operations, and address formats.
    fn capabilities(&self) -> &SignerCapabilities;

    /// Validates the request and capability declaration before signing.
    fn sign(&self, request: &SignRequest) -> Result<SignResponse, SigningError> {
        let capability =
            self.capabilities()
                .require(&request.target, request.algorithm, request.operation())?;
        request.validate()?;
        let response = self.sign_impl(request)?;
        response.validate_for(request, capability)?;
        Ok(response)
    }

    /// Implementation hook invoked only after [`Self::sign`] has validated the
    /// request. It must keep all key material within the concrete signer.
    fn sign_impl(&self, request: &SignRequest) -> Result<SignResponse, SigningError> {
        Err(SigningError::UnsupportedOperation {
            chain: request.target.chain.clone(),
            operation: request.operation(),
        })
    }

    /// Validates the request and derives a public address through the concrete
    /// implementation.
    fn derive_address(
        &self,
        request: &AddressDerivationRequest,
    ) -> Result<AddressDerivationResponse, SigningError> {
        let capability = self.capabilities().require(
            &request.target,
            request.algorithm,
            SigningOperation::DeriveAddress,
        )?;
        request.validate()?;
        let response = self.derive_address_impl(request)?;
        response.validate_for(request, capability)?;
        Ok(response)
    }

    /// Implementation hook for public-address derivation.
    fn derive_address_impl(
        &self,
        request: &AddressDerivationRequest,
    ) -> Result<AddressDerivationResponse, SigningError> {
        Err(SigningError::UnsupportedOperation {
            chain: request.target.chain.clone(),
            operation: SigningOperation::DeriveAddress,
        })
    }

    /// Validates complete verification inputs and returns a positive or
    /// negative verification result. Invalid requests remain errors.
    fn verify_signature(
        &self,
        request: &VerificationRequest,
    ) -> Result<VerificationResult, SigningError> {
        let capability =
            self.capabilities()
                .require(&request.target, request.algorithm, request.operation())?;
        request.validate()?;
        let result = self.verify_signature_impl(request)?;
        result.validate_for(request)?;
        // Keep the capability borrow in this method so a future implementation
        // cannot accidentally bypass the declared verification operation.
        let _ = capability;
        Ok(result)
    }

    /// Implementation hook for cryptographic verification.
    fn verify_signature_impl(
        &self,
        request: &VerificationRequest,
    ) -> Result<VerificationResult, SigningError> {
        Err(SigningError::UnsupportedOperation {
            chain: request.target.chain.clone(),
            operation: SigningOperation::VerifySignature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_mapping_covers_existing_chain_families() {
        assert_eq!(
            SigningTarget::for_chain(Chain::Bitcoin).family,
            ChainFamily::BitcoinUtxo
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Stacks).family,
            ChainFamily::Anchor
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Ethereum).family,
            ChainFamily::Evm
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Solana).family,
            ChainFamily::SolanaSvm
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Babylon).family,
            ChainFamily::BPoS
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Liquid).family,
            ChainFamily::Federation
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Rootstock).family,
            ChainFamily::MergeMined
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Citrea).family,
            ChainFamily::Rollup
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Spark).family,
            ChainFamily::Statechain
        );
        assert_eq!(
            SigningTarget::for_chain(Chain::Second).family,
            ChainFamily::Ark
        );
    }

    #[test]
    fn payload_validation_is_explicit_and_fail_closed() {
        assert_eq!(
            SigningPayload::message(Vec::<u8>::new()).validate(),
            Err(SigningError::InvalidPayload(PayloadError::EmptyMessage))
        );
        assert!(SigningPayload::digest(DigestAlgorithm::Sha256, vec![0; 32])
            .validate()
            .is_ok());
        assert!(matches!(
            SigningPayload::digest(DigestAlgorithm::Sha256, vec![0; 31]).validate(),
            Err(SigningError::InvalidPayload(
                PayloadError::InvalidDigestLength { .. }
            ))
        ));
    }

    #[test]
    fn capabilities_reject_unknown_algorithm_and_operation() {
        let target = SigningTarget::for_chain(Chain::Bitcoin);
        let capabilities = SignerCapabilities::new(
            UNIVERSAL_CHAIN_SIGNER_API_VERSION,
            vec![ChainSigningCapability::new(
                target.clone(),
                vec![SigningAlgorithm::SchnorrSecp256k1],
                vec![SigningOperation::SignMessage],
                vec![],
            )],
        );

        assert!(matches!(
            capabilities.require(
                &target,
                SigningAlgorithm::Ed25519,
                SigningOperation::SignMessage,
            ),
            Err(SigningError::UnsupportedAlgorithm { .. })
        ));
        assert!(matches!(
            capabilities.require(
                &target,
                SigningAlgorithm::SchnorrSecp256k1,
                SigningOperation::VerifySignature,
            ),
            Err(SigningError::UnsupportedOperation { .. })
        ));
    }

    #[test]
    fn serialization_round_trip_preserves_contract_models() {
        let request = SignRequest::new(
            SigningTarget::for_chain(Chain::Stacks),
            SigningAlgorithm::EcdsaSecp256k1,
            SigningPayload::message(b"stacks message".to_vec()),
            DerivationContext::new(
                DerivationPath::new(vec![DerivationIndex::new(44, true)]),
                DerivationPurpose::MessageSigning,
            ),
        );

        let encoded = serde_json::to_string(&request).expect("request serializes");
        let decoded: SignRequest = serde_json::from_str(&encoded).expect("request deserializes");
        assert_eq!(decoded, request);
    }
}
