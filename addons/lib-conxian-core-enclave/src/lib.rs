//! Fail-closed adapter contracts between `lib-conxian-core` and
//! `conxius-enclave-sdk` `2.0.11`.
//!
//! This crate is deliberately a companion crate rather than a feature inside
//! Core. Core remains the owner of canonical request, invariant, trust-tier,
//! and BIP-110 contracts, while this crate owns the exact SDK mapping and the
//! injected-manager boundary. It never owns networking, persistence, replay
//! state, telemetry, attestation verification, or private key material.
//!
//! The SDK's `EnclaveManager::sign` API accepts a field named `message_hash`
//! and does not carry a digest-algorithm discriminator. Consequently, this
//! adapter accepts only an explicit 32-byte Core `Sha256` digest. It rejects
//! Core messages and other digest algorithms rather than hashing or relabeling
//! caller data. The adapter derives a domain-separated digest from the
//! canonical Core descriptor, the original digest, and the validated request
//! policy context; duplicate-cache storage remains outside this crate.
//!
//! This addon is pre-release. Its signing APIs intentionally require the
//! canonical Core `SignedEnvelopeDescriptor` and an adapter-owned
//! `RequestPolicyContext`; callers no longer supply a replay-binding DTO as
//! signing authority. Provider signatures cover the bound digest, not the
//! original digest.

pub mod policy;

use std::fmt;
use std::sync::Arc;

use conxius_enclave_sdk::enclave::{
    attestation::{AttestationLevel as SdkAttestationLevel, DeviceIntegrityReport},
    EnclaveManager, SignRequest as SdkSignRequest, SignResponse as SdkSignResponse,
    SigningAlgorithm as SdkSigningAlgorithm,
};
use lib_conxian_core::{
    control_model::{
        validate_bip110_preflight, validate_signed_envelope_descriptor, Bip110PreflightRequest,
        Chain, ChainFamily, SignedEnvelopeDescriptor, TrustTier,
    },
    signing::{
        DerivationContext, DerivationPathError, DigestAlgorithm, PublicVerificationKey,
        SignRequest, Signature, SignatureEncoding, SigningAlgorithm, SigningPayload, SigningTarget,
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha2Digest, Sha256};

/// The exact published SDK release targeted by this adapter.
pub const SUPPORTED_SDK_VERSION: &str = "2.0.11";

/// Core permits at most this many structured derivation components.
pub const MAX_DERIVATION_COMPONENTS: usize = 255;

const REPLAY_BINDING_DOMAIN: &[u8] = b"lib-conxian-core-enclave/replay-binding/v2";

pub use policy::{
    core_trust_to_sdk_rail_tier, sdk_rail_tier_to_core_observation, sdk_rail_tier_to_observation,
    validate_rail_trust, NetworkPolicy, RailTrustPolicy, RailTrustTier, RequestPolicyContext,
};

/// A stable adapter-owned summary of the SDK's attestation level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttestationLevel {
    /// Development-only software attestation. This adapter never permits it for signing.
    Software,
    /// Trusted execution environment attestation.
    Tee,
    /// Hardware-backed StrongBox attestation.
    StrongBox,
    /// Hardware-backed cloud TEE attestation.
    CloudTee,
}

impl fmt::Display for AttestationLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Software => "software",
            Self::Tee => "tee",
            Self::StrongBox => "strongbox",
            Self::CloudTee => "cloud_tee",
        })
    }
}

/// Minimum attestation strength required by a Core trust tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinimumAttestation {
    /// TEE or a stronger level is required.
    Tee,
    /// StrongBox or CloudTEE is required.
    HardwareBacked,
}

/// Conservative Core trust-tier policy used by the adapter.
///
/// `ObserverOnly` is retained as a valid policy value so callers can model
/// observation contexts, but every signing entry point rejects it. `Software`
/// attestation is rejected for every signing tier because the published SDK
/// documents its software path as development-only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrustPolicy {
    tier: TrustTier,
    minimum_attestation: Option<MinimumAttestation>,
}

impl TrustPolicy {
    /// Constructs the conservative policy for a Core trust tier.
    pub fn for_tier(tier: TrustTier) -> Self {
        let minimum_attestation = match &tier {
            TrustTier::Strict => Some(MinimumAttestation::HardwareBacked),
            TrustTier::Managed | TrustTier::Expedient => Some(MinimumAttestation::Tee),
            TrustTier::ObserverOnly => None,
        };

        Self {
            tier,
            minimum_attestation,
        }
    }

    /// Returns the Core tier represented by this policy.
    pub fn tier(&self) -> &TrustTier {
        &self.tier
    }

    /// Returns the minimum SDK attestation strength for signing, if signing is allowed.
    pub const fn minimum_attestation(&self) -> Option<MinimumAttestation> {
        self.minimum_attestation
    }

    /// Rejects custom or deserialized policies that weaken the canonical
    /// minimum for their Core trust tier.
    pub fn validate(&self) -> Result<(), AdapterError> {
        let required = match &self.tier {
            TrustTier::Strict => Some(MinimumAttestation::HardwareBacked),
            TrustTier::Managed | TrustTier::Expedient => Some(MinimumAttestation::Tee),
            TrustTier::ObserverOnly => None,
        };

        let valid = matches!(
            (&self.tier, self.minimum_attestation, required),
            (
                TrustTier::Strict,
                Some(MinimumAttestation::HardwareBacked),
                Some(_)
            ) | (
                TrustTier::Managed | TrustTier::Expedient,
                Some(MinimumAttestation::Tee | MinimumAttestation::HardwareBacked),
                Some(_),
            ) | (TrustTier::ObserverOnly, None, None)
        );

        if valid {
            Ok(())
        } else {
            Err(AdapterError::InvalidTrustPolicy {
                tier: self.tier.clone(),
                required,
                configured: self.minimum_attestation,
            })
        }
    }

    fn ensure_signing_allowed(&self) -> Result<(), AdapterError> {
        if matches!(&self.tier, TrustTier::ObserverOnly) {
            return Err(AdapterError::ObserverOnlyCannotSign);
        }
        Ok(())
    }

    fn validate_attestation(
        &self,
        provider_attestation: Option<&str>,
        expected_nonce: &[u8],
    ) -> Result<AttestationSummary, AdapterError> {
        self.ensure_signing_allowed()?;

        let serialized = provider_attestation.ok_or(AdapterError::MissingAttestation)?;
        if serialized.trim().is_empty() {
            return Err(AdapterError::MissingAttestation);
        }

        let report: DeviceIntegrityReport =
            serde_json::from_str(serialized).map_err(|_| AdapterError::InvalidAttestation)?;
        if report.challenge_nonce != expected_nonce {
            return Err(AdapterError::AttestationChallengeMismatch);
        }
        let level = AttestationLevel::from_sdk(report.level);

        let allowed = match self.minimum_attestation {
            Some(MinimumAttestation::Tee) => {
                matches!(
                    level,
                    AttestationLevel::Tee
                        | AttestationLevel::StrongBox
                        | AttestationLevel::CloudTee
                )
            }
            Some(MinimumAttestation::HardwareBacked) => {
                matches!(
                    level,
                    AttestationLevel::StrongBox | AttestationLevel::CloudTee
                )
            }
            None => false,
        };

        if !allowed {
            return Err(AdapterError::InsufficientAttestation {
                required: self.minimum_attestation,
                observed: level,
            });
        }

        Ok(AttestationSummary {
            level,
            device_fingerprint: report.get_device_fingerprint(),
            evidence: AttestationEvidence {
                raw_report: serialized.to_owned(),
                level,
                challenge_nonce: report.challenge_nonce,
                signature: report.signature,
                certificate_chain: report.certificate_chain,
                timestamp: report.timestamp,
                extension_data: report.extension_data,
            },
        })
    }
}

/// Opaque and structured attestation evidence retained for downstream
/// verification.
///
/// The adapter preserves the exact SDK JSON plus every field in
/// [`DeviceIntegrityReport`]. It performs only JSON parsing, request-nonce
/// binding, and trust-level gating; it does not verify report signatures,
/// certificate chains, freshness, or hardware claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AttestationEvidence {
    pub raw_report: String,
    pub level: AttestationLevel,
    pub challenge_nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub certificate_chain: Vec<String>,
    pub timestamp: u64,
    pub extension_data: String,
}

/// Public attestation metadata returned by a successful adapter call.
///
/// The summary contains the adapter's conservative level-gate result and the
/// complete evidence needed by a downstream SDK/provider verifier. This layer
/// does not claim cryptographic verification of the report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AttestationSummary {
    pub level: AttestationLevel,
    pub device_fingerprint: String,
    pub evidence: AttestationEvidence,
}

/// Deterministic, adapter-owned binding between a Core signed-envelope
/// identity and the digest sent to the SDK.
///
/// The adapter does not store this binding or implement a replay cache. The
/// SDK or a higher runtime owns duplicate detection, cache TTLs, and durable
/// idempotency state. This value only makes the provider input unambiguous and
/// rejects a missing, altered, or request-mismatched identity before provider
/// invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReplayBinding {
    publisher: String,
    event_id: String,
    sequence: u64,
    payload_hash: String,
    commitments: Vec<String>,
    core_digest: Vec<u8>,
    policy_context: RequestPolicyContext,
    bound_digest: Vec<u8>,
}

impl ReplayBinding {
    /// Creates a binding from Core's canonical signed-envelope descriptor.
    ///
    /// This constructor is private by design. A binding is response evidence,
    /// never caller-supplied signing authority.
    fn from_envelope(
        descriptor: &SignedEnvelopeDescriptor,
        core_digest: &[u8],
        policy_context: &RequestPolicyContext,
    ) -> Result<Self, AdapterError> {
        let bound_digest = replay_binding_digest(descriptor, core_digest, policy_context)?;
        Ok(Self {
            publisher: descriptor.publisher.clone(),
            event_id: descriptor.event_id.clone(),
            sequence: descriptor.sequence,
            payload_hash: descriptor.payload_hash.clone(),
            commitments: descriptor.commitments.clone(),
            core_digest: core_digest.to_vec(),
            policy_context: policy_context.clone(),
            bound_digest,
        })
    }

    /// Returns the descriptor publisher carried as response evidence.
    pub fn publisher(&self) -> &str {
        &self.publisher
    }

    /// Returns the descriptor event identifier carried as response evidence.
    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    /// Returns the descriptor sequence carried as response evidence.
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the descriptor payload hash carried as response evidence.
    pub fn payload_hash(&self) -> &str {
        &self.payload_hash
    }

    /// Returns the descriptor commitments carried as response evidence.
    pub fn commitments(&self) -> &[String] {
        &self.commitments
    }

    /// Returns the original Core digest carried as response evidence.
    pub fn core_digest(&self) -> &[u8] {
        &self.core_digest
    }

    /// Returns the validated request policy context carried as response evidence.
    pub fn policy_context(&self) -> &RequestPolicyContext {
        &self.policy_context
    }

    /// Returns the digest actually sent to the SDK provider.
    pub fn bound_digest(&self) -> &[u8] {
        &self.bound_digest
    }

    /// Returns Core's display idempotency key for compatibility and logging.
    ///
    /// This colon-joined value is not used as cryptographic signing input.
    pub fn idempotency_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.publisher.trim(),
            self.event_id.trim(),
            self.sequence
        )
    }
}

/// Computes the domain-separated digest that is sent to SDK `2.0.11`.
///
/// The canonical descriptor fields are encoded independently with explicit
/// length prefixes. This deliberately avoids Core's human-readable,
/// colon-joined `idempotency_key()` representation, which is not injective.
pub fn replay_binding_digest(
    descriptor: &SignedEnvelopeDescriptor,
    core_digest: &[u8],
    policy_context: &RequestPolicyContext,
) -> Result<Vec<u8>, AdapterError> {
    validate_signed_envelope_descriptor(descriptor)
        .map_err(|_| AdapterError::InvalidReplayBinding)?;
    if core_digest.is_empty() {
        return Err(AdapterError::InvalidReplayBinding);
    }
    policy_context.validate()?;

    let mut hasher = Sha256::new();
    hasher.update(REPLAY_BINDING_DOMAIN);
    append_binding_field(&mut hasher, b"publisher", descriptor.publisher.as_bytes());
    append_binding_field(&mut hasher, b"event_id", descriptor.event_id.as_bytes());
    append_binding_field(&mut hasher, b"sequence", &descriptor.sequence.to_be_bytes());
    append_binding_field(
        &mut hasher,
        b"payload_hash",
        descriptor.payload_hash.as_bytes(),
    );
    append_binding_field(
        &mut hasher,
        b"commitment_count",
        &(descriptor.commitments.len() as u64).to_be_bytes(),
    );
    for commitment in &descriptor.commitments {
        append_binding_field(&mut hasher, b"commitment", commitment.as_bytes());
    }
    append_binding_field(&mut hasher, b"core_digest", core_digest);
    append_binding_field(
        &mut hasher,
        b"network",
        policy_context.network.wire_name().as_bytes(),
    );
    append_binding_field(
        &mut hasher,
        b"requested_core_tier",
        trust_tier_wire_name(&policy_context.rail.requested_core_tier).as_bytes(),
    );
    append_binding_field(
        &mut hasher,
        b"observed_sdk_rail_tier",
        policy_context.rail.observed_sdk_tier.wire_name().as_bytes(),
    );
    Ok(hasher.finalize().to_vec())
}

fn append_binding_field(hasher: &mut Sha256, label: &[u8], value: &[u8]) {
    hasher.update((label.len() as u64).to_be_bytes());
    hasher.update(label);
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn trust_tier_wire_name(tier: &TrustTier) -> &'static str {
    match tier {
        TrustTier::Strict => "strict",
        TrustTier::Managed => "managed",
        TrustTier::Expedient => "expedient",
        TrustTier::ObserverOnly => "observer_only",
    }
}

/// Adapter-owned signing result. It contains public signature material only;
/// private keys, seeds, shares, and provider handles never cross this boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnclaveSignResponse {
    pub target: SigningTarget,
    pub algorithm: SigningAlgorithm,
    pub signature: Signature,
    pub verification_key: PublicVerificationKey,
    pub derivation: DerivationContext,
    pub replay_binding: ReplayBinding,
    pub attestation: AttestationSummary,
}

/// Provider response fields that can be malformed without exposing their contents.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderResponseField {
    Signature,
    VerificationKey,
    Attestation,
}

/// Secret-safe typed error taxonomy for the adapter boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdapterError {
    InvalidConfiguration,
    InvalidTarget,
    ObserverOnlyCannotSign,
    InvalidTrustPolicy {
        tier: TrustTier,
        required: Option<MinimumAttestation>,
        configured: Option<MinimumAttestation>,
    },
    UnknownRailTrustTier {
        value: String,
    },
    RailTrustDowngrade {
        requested: TrustTier,
        observed: RailTrustTier,
    },
    PolicyContextTierMismatch {
        adapter: TrustTier,
        requested: TrustTier,
    },
    UnknownNetwork {
        value: String,
    },
    MessagePayloadRejected,
    UnsupportedDigestAlgorithm(DigestAlgorithm),
    InvalidDigestLength {
        expected: usize,
        actual: usize,
    },
    InvalidDerivationPath(DerivationPathError),
    UnsupportedChainAlgorithm {
        chain: Chain,
        family: ChainFamily,
        algorithm: SigningAlgorithm,
    },
    UnsupportedPublicKeyDerivation(SigningAlgorithm),
    PreflightRequired,
    PreflightTargetMismatch,
    PreflightRejected {
        code: String,
    },
    ProviderFailure,
    MalformedProviderResponse(ProviderResponseField),
    MissingAttestation,
    InvalidAttestation,
    AttestationChallengeMismatch,
    InvalidReplayBinding,
    InsufficientAttestation {
        required: Option<MinimumAttestation>,
        observed: AttestationLevel,
    },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => formatter.write_str("invalid adapter configuration"),
            Self::InvalidTarget => formatter.write_str("invalid signing target"),
            Self::ObserverOnlyCannotSign => formatter.write_str("observer-only policy cannot sign"),
            Self::InvalidTrustPolicy {
                tier,
                required,
                configured,
            } => write!(
                formatter,
                "trust policy for {tier:?} requires {required:?}, but configured {configured:?}"
            ),
            Self::UnknownRailTrustTier { value } => {
                write!(formatter, "unknown SDK rail trust tier: {value}")
            }
            Self::RailTrustDowngrade {
                requested,
                observed,
            } => write!(
                formatter,
                "observed SDK rail tier {observed:?} is weaker than requested Core tier {requested:?}"
            ),
            Self::PolicyContextTierMismatch { adapter, requested } => write!(
                formatter,
                "request policy tier {requested:?} does not match adapter Core tier {adapter:?}"
            ),
            Self::UnknownNetwork { value } => {
                write!(formatter, "unknown SDK network value: {value}")
            }
            Self::MessagePayloadRejected => formatter
                .write_str("message payloads are rejected; provide an explicit supported digest"),
            Self::UnsupportedDigestAlgorithm(algorithm) => {
                write!(
                    formatter,
                    "digest algorithm {algorithm:?} is unsupported by SDK 2.0.11"
                )
            }
            Self::InvalidDigestLength { expected, actual } => write!(
                formatter,
                "digest length {actual} does not match the required length {expected}"
            ),
            Self::InvalidDerivationPath(error) => {
                write!(formatter, "invalid derivation path: {error}")
            }
            Self::UnsupportedChainAlgorithm {
                chain,
                family,
                algorithm,
            } => write!(
                formatter,
                "signing algorithm {algorithm:?} is not allowed for {chain:?} in {family:?}"
            ),
            Self::UnsupportedPublicKeyDerivation(algorithm) => write!(
                formatter,
                "public-key derivation is unsupported for {algorithm:?} with SDK 2.0.11"
            ),
            Self::PreflightRequired => {
                formatter.write_str("Bitcoin signing requires a compliant Core BIP-110 preflight")
            }
            Self::PreflightTargetMismatch => {
                formatter.write_str("BIP-110 preflight is only valid for a Bitcoin target")
            }
            Self::PreflightRejected { code } => {
                write!(formatter, "Core BIP-110 preflight rejected signing: {code}")
            }
            Self::ProviderFailure => formatter.write_str("enclave provider failed"),
            Self::MalformedProviderResponse(field) => {
                write!(formatter, "malformed enclave provider response: {field:?}")
            }
            Self::MissingAttestation => formatter.write_str("enclave provider omitted attestation"),
            Self::InvalidAttestation => {
                formatter.write_str("enclave provider returned invalid attestation")
            }
            Self::AttestationChallengeMismatch => {
                formatter.write_str("enclave attestation is not bound to the signing digest")
            }
            Self::InvalidReplayBinding => {
                formatter.write_str("invalid Core envelope replay binding")
            }
            Self::InsufficientAttestation { required, observed } => write!(
                formatter,
                "attestation level {observed} does not meet required level {required:?}"
            ),
        }
    }
}

impl std::error::Error for AdapterError {}

/// Converts a Core algorithm to the exact SDK `2.0.11` enum.
pub fn to_sdk_algorithm(algorithm: SigningAlgorithm) -> SdkSigningAlgorithm {
    match algorithm {
        SigningAlgorithm::EcdsaSecp256k1 => SdkSigningAlgorithm::EcdsaSecp256k1,
        SigningAlgorithm::SchnorrSecp256k1 => SdkSigningAlgorithm::SchnorrSecp256k1,
        SigningAlgorithm::Ed25519 => SdkSigningAlgorithm::Ed25519,
    }
}

/// Converts an exact SDK `2.0.11` algorithm to the Core enum.
pub fn from_sdk_algorithm(algorithm: SdkSigningAlgorithm) -> SigningAlgorithm {
    match algorithm {
        SdkSigningAlgorithm::EcdsaSecp256k1 => SigningAlgorithm::EcdsaSecp256k1,
        SdkSigningAlgorithm::SchnorrSecp256k1 => SigningAlgorithm::SchnorrSecp256k1,
        SdkSigningAlgorithm::Ed25519 => SigningAlgorithm::Ed25519,
    }
}

impl AttestationLevel {
    fn from_sdk(level: SdkAttestationLevel) -> Self {
        match level {
            SdkAttestationLevel::Software => Self::Software,
            SdkAttestationLevel::TEE => Self::Tee,
            SdkAttestationLevel::StrongBox => Self::StrongBox,
            SdkAttestationLevel::CloudTEE => Self::CloudTee,
        }
    }
}

/// Returns whether the adapter's conservative SDK `2.0.11` capability
/// allowlist permits an algorithm for a canonical Core target.
///
/// The allowlist intentionally names concrete chains instead of treating a
/// coarse `ChainFamily` as proof of SDK support. Bitcoin-family monetary
/// targets use secp256k1 algorithms, Stacks uses ECDSA secp256k1, Ethereum uses
/// ECDSA secp256k1, and Solana uses Ed25519. Other Core chains are denied until
/// an exact SDK mapping is established.
pub fn is_supported_chain_algorithm(target: &SigningTarget, algorithm: SigningAlgorithm) -> bool {
    matches!(
        (&target.chain, &target.family, algorithm),
        (
            Chain::Bitcoin | Chain::Lightning,
            ChainFamily::BitcoinUtxo,
            SigningAlgorithm::EcdsaSecp256k1 | SigningAlgorithm::SchnorrSecp256k1,
        ) | (
            Chain::Liquid,
            ChainFamily::Federation,
            SigningAlgorithm::EcdsaSecp256k1 | SigningAlgorithm::SchnorrSecp256k1,
        ) | (
            Chain::Babylon,
            ChainFamily::BPoS,
            SigningAlgorithm::EcdsaSecp256k1 | SigningAlgorithm::SchnorrSecp256k1,
        ) | (
            Chain::Stacks,
            ChainFamily::Anchor,
            SigningAlgorithm::EcdsaSecp256k1
        ) | (
            Chain::Ethereum,
            ChainFamily::Evm,
            SigningAlgorithm::EcdsaSecp256k1
        ) | (
            Chain::Solana,
            ChainFamily::SolanaSvm,
            SigningAlgorithm::Ed25519
        )
    )
}

/// Validates a canonical Core target and the adapter's explicit chain/
/// algorithm capability mapping before any provider call.
pub fn validate_chain_algorithm(
    target: &SigningTarget,
    algorithm: SigningAlgorithm,
) -> Result<(), AdapterError> {
    target.validate().map_err(|_| AdapterError::InvalidTarget)?;
    if is_supported_chain_algorithm(target, algorithm) {
        Ok(())
    } else {
        Err(AdapterError::UnsupportedChainAlgorithm {
            chain: target.chain.clone(),
            family: target.family.clone(),
            algorithm,
        })
    }
}

/// Renders Core's structured derivation context into the exact path string
/// accepted by SDK `2.0.11`.
///
/// The Core purpose is intentionally not encoded: the SDK request has no
/// purpose field, and inventing a purpose-to-path convention would change key
/// derivation semantics. The purpose remains in the adapter-owned response.
pub fn render_derivation_path(context: &DerivationContext) -> Result<String, AdapterError> {
    if context.path.components.len() > MAX_DERIVATION_COMPONENTS {
        return Err(AdapterError::InvalidDerivationPath(
            DerivationPathError::TooManyComponents,
        ));
    }

    let mut rendered = String::from("m");
    for component in &context.path.components {
        rendered.push('/');
        rendered.push_str(&component.index.to_string());
        if component.hardened {
            rendered.push('\'');
        }
    }
    Ok(rendered)
}

/// Extracts a digest without hashing, encoding, or otherwise changing caller bytes.
///
/// SDK `2.0.11` exposes only `message_hash: Vec<u8>`, so the adapter supports
/// the unambiguous 32-byte Core `Sha256` representation and rejects all other
/// forms. In particular, a Core `Message` is never silently hashed.
pub fn extract_sdk_digest(payload: &SigningPayload) -> Result<Vec<u8>, AdapterError> {
    match payload {
        SigningPayload::Message { .. } => Err(AdapterError::MessagePayloadRejected),
        SigningPayload::Digest { algorithm, bytes } => {
            if *algorithm != DigestAlgorithm::Sha256 {
                return Err(AdapterError::UnsupportedDigestAlgorithm(*algorithm));
            }
            if bytes.len() != DigestAlgorithm::Sha256.output_len() {
                return Err(AdapterError::InvalidDigestLength {
                    expected: DigestAlgorithm::Sha256.output_len(),
                    actual: bytes.len(),
                });
            }
            Ok(bytes.clone())
        }
    }
}

/// Injected-manager adapter over the exact SDK `2.0.11` `EnclaveManager`
/// interface.
pub struct EnclaveSdkAdapter {
    manager: Arc<dyn EnclaveManager>,
    key_id: String,
    trust_policy: TrustPolicy,
}

struct PreparedSigningRequest {
    sdk_request: SdkSignRequest,
    binding: ReplayBinding,
}

impl EnclaveSdkAdapter {
    /// Creates an adapter with a stable SDK key identifier and Core trust tier.
    pub fn new(
        manager: Arc<dyn EnclaveManager>,
        key_id: impl Into<String>,
        tier: TrustTier,
    ) -> Result<Self, AdapterError> {
        Self::with_policy(manager, key_id, TrustPolicy::for_tier(tier))
    }

    /// Creates an adapter with an explicit conservative trust policy.
    pub fn with_policy(
        manager: Arc<dyn EnclaveManager>,
        key_id: impl Into<String>,
        trust_policy: TrustPolicy,
    ) -> Result<Self, AdapterError> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() {
            return Err(AdapterError::InvalidConfiguration);
        }
        trust_policy.validate()?;

        Ok(Self {
            manager,
            key_id,
            trust_policy,
        })
    }

    /// Returns the configured Core trust policy.
    pub fn trust_policy(&self) -> &TrustPolicy {
        &self.trust_policy
    }

    /// Builds the exact SDK request after Core-side target, payload, descriptor,
    /// request-policy, and derivation validation. The replay binding is derived
    /// internally from the canonical descriptor; callers cannot provide one as
    /// signing authority. This is a request-builder boundary and does not
    /// invoke the provider.
    pub fn build_sdk_sign_request(
        &self,
        request: &SignRequest,
        descriptor: &SignedEnvelopeDescriptor,
        policy_context: &RequestPolicyContext,
    ) -> Result<SdkSignRequest, AdapterError> {
        Ok(self
            .prepare_signing_request(request, descriptor, policy_context)?
            .sdk_request)
    }

    /// Signs an explicit supported digest for a non-Bitcoin target.
    ///
    /// Bitcoin targets must use [`Self::sign_digest_with_bip110_preflight`]
    /// so the Core preflight is evaluated before provider invocation.
    pub fn sign_digest(
        &self,
        request: &SignRequest,
        descriptor: &SignedEnvelopeDescriptor,
        policy_context: &RequestPolicyContext,
    ) -> Result<EnclaveSignResponse, AdapterError> {
        self.validate_signing_request(request, policy_context)?;
        if request.target.chain == Chain::Bitcoin {
            return Err(AdapterError::PreflightRequired);
        }
        self.sign_digest_without_preflight(request, descriptor, policy_context)
    }

    /// Runs the exact Core BIP-110 preflight validator before invoking the
    /// injected SDK manager. Any structural error or size violation returns
    /// before `EnclaveManager::sign` is called.
    pub fn sign_digest_with_bip110_preflight(
        &self,
        request: &SignRequest,
        preflight: &Bip110PreflightRequest,
        descriptor: &SignedEnvelopeDescriptor,
        policy_context: &RequestPolicyContext,
    ) -> Result<EnclaveSignResponse, AdapterError> {
        self.validate_signing_request(request, policy_context)?;
        let result = validate_bip110_preflight(preflight);
        if !result.is_compliant {
            let code = result
                .findings
                .first()
                .map(|finding| finding.code())
                .unwrap_or("non_compliant");
            return Err(AdapterError::PreflightRejected {
                code: code.to_owned(),
            });
        }

        if request.target.chain != Chain::Bitcoin {
            return Err(AdapterError::PreflightTargetMismatch);
        }

        self.sign_digest_without_preflight(request, descriptor, policy_context)
    }

    /// Derives a public verification key through the exact SDK manager API.
    /// No private key material or raw provider response crosses the boundary.
    pub fn derive_public_key(
        &self,
        target: &SigningTarget,
        algorithm: SigningAlgorithm,
        derivation: &DerivationContext,
    ) -> Result<PublicVerificationKey, AdapterError> {
        validate_chain_algorithm(target, algorithm)?;
        if matches!(
            algorithm,
            SigningAlgorithm::SchnorrSecp256k1 | SigningAlgorithm::Ed25519
        ) {
            return Err(AdapterError::UnsupportedPublicKeyDerivation(algorithm));
        }
        let derivation_path = render_derivation_path(derivation)?;
        let public_key_hex = self
            .manager
            .get_public_key(&derivation_path)
            .map_err(|_| AdapterError::ProviderFailure)?;
        let public_key_bytes =
            decode_provider_hex(ProviderResponseField::VerificationKey, &public_key_hex)?;
        validate_public_key_length(algorithm, public_key_bytes.len())?;

        Ok(PublicVerificationKey::new(algorithm, public_key_bytes))
    }

    fn validate_signing_request(
        &self,
        request: &SignRequest,
        policy_context: &RequestPolicyContext,
    ) -> Result<(), AdapterError> {
        self.trust_policy.validate()?;
        self.trust_policy.ensure_signing_allowed()?;
        policy_context.validate()?;
        if self.trust_policy.tier() != &policy_context.rail.requested_core_tier {
            return Err(AdapterError::PolicyContextTierMismatch {
                adapter: self.trust_policy.tier().clone(),
                requested: policy_context.rail.requested_core_tier.clone(),
            });
        }
        let _ = policy_context.rail.signing_sdk_tier()?;
        let _ = policy_context.network.to_sdk();
        validate_chain_algorithm(&request.target, request.algorithm)
    }

    fn prepare_signing_request(
        &self,
        request: &SignRequest,
        descriptor: &SignedEnvelopeDescriptor,
        policy_context: &RequestPolicyContext,
    ) -> Result<PreparedSigningRequest, AdapterError> {
        self.validate_signing_request(request, policy_context)?;
        let core_digest = extract_sdk_digest(&request.payload)?;
        let binding = ReplayBinding::from_envelope(descriptor, &core_digest, policy_context)?;
        let derivation_path = render_derivation_path(&request.derivation)?;
        let sdk_request = SdkSignRequest {
            algorithm: to_sdk_algorithm(request.algorithm),
            message_hash: binding.bound_digest().to_vec(),
            derivation_path,
            key_id: self.key_id.clone(),
            taproot_tweak: None,
        };

        Ok(PreparedSigningRequest {
            sdk_request,
            binding,
        })
    }

    fn sign_digest_without_preflight(
        &self,
        request: &SignRequest,
        descriptor: &SignedEnvelopeDescriptor,
        policy_context: &RequestPolicyContext,
    ) -> Result<EnclaveSignResponse, AdapterError> {
        let PreparedSigningRequest {
            sdk_request,
            binding,
        } = self.prepare_signing_request(request, descriptor, policy_context)?;
        let message_hash = sdk_request.message_hash.clone();
        let sdk_response = self
            .manager
            .sign(sdk_request)
            .map_err(|_| AdapterError::ProviderFailure)?;
        self.map_sdk_response(request, &message_hash, &binding, sdk_response)
    }

    fn map_sdk_response(
        &self,
        request: &SignRequest,
        message_hash: &[u8],
        replay_binding: &ReplayBinding,
        response: SdkSignResponse,
    ) -> Result<EnclaveSignResponse, AdapterError> {
        let signature_bytes =
            decode_provider_hex(ProviderResponseField::Signature, &response.signature_hex)?;
        let verification_key_bytes = decode_provider_hex(
            ProviderResponseField::VerificationKey,
            &response.public_key_hex,
        )?;
        let algorithm = request.algorithm;
        let encoding = signature_encoding(algorithm, signature_bytes.len())?;
        validate_public_key_length(algorithm, verification_key_bytes.len())?;
        let attestation = self
            .trust_policy
            .validate_attestation(response.device_attestation.as_deref(), message_hash)?;

        Ok(EnclaveSignResponse {
            target: request.target.clone(),
            algorithm,
            signature: Signature::new(algorithm, encoding, signature_bytes),
            verification_key: PublicVerificationKey::new(algorithm, verification_key_bytes),
            derivation: request.derivation.clone(),
            replay_binding: replay_binding.clone(),
            attestation,
        })
    }
}

fn decode_provider_hex(field: ProviderResponseField, value: &str) -> Result<Vec<u8>, AdapterError> {
    if value.is_empty() {
        return Err(AdapterError::MalformedProviderResponse(field));
    }
    hex::decode(value).map_err(|_| AdapterError::MalformedProviderResponse(field))
}

fn signature_encoding(
    algorithm: SigningAlgorithm,
    byte_len: usize,
) -> Result<SignatureEncoding, AdapterError> {
    let encoding = match algorithm {
        SigningAlgorithm::EcdsaSecp256k1 => match byte_len {
            64 => SignatureEncoding::Compact,
            65 => SignatureEncoding::Recoverable,
            _ => {
                return Err(AdapterError::MalformedProviderResponse(
                    ProviderResponseField::Signature,
                ))
            }
        },
        SigningAlgorithm::SchnorrSecp256k1 if byte_len == 64 => SignatureEncoding::Compact,
        SigningAlgorithm::Ed25519 if byte_len == 64 => SignatureEncoding::Raw,
        _ => {
            return Err(AdapterError::MalformedProviderResponse(
                ProviderResponseField::Signature,
            ))
        }
    };
    Ok(encoding)
}

fn validate_public_key_length(
    algorithm: SigningAlgorithm,
    byte_len: usize,
) -> Result<(), AdapterError> {
    let valid = match algorithm {
        SigningAlgorithm::EcdsaSecp256k1 => matches!(byte_len, 33 | 65),
        SigningAlgorithm::SchnorrSecp256k1 => byte_len == 32,
        SigningAlgorithm::Ed25519 => byte_len == 32,
    };

    if valid {
        Ok(())
    } else {
        Err(AdapterError::MalformedProviderResponse(
            ProviderResponseField::VerificationKey,
        ))
    }
}
