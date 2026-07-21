//! Neutral structural contracts for BIP-341, BIP-342, and Miniscript handoff.
//!
//! This module deliberately stops at public byte-shape and metadata invariants. It does not
//! parse transactions, verify Schnorr signatures, tweak keys, verify Taproot commitments,
//! execute Tapscript, compile Miniscript, or construct satisfaction witnesses. Those operations
//! remain downstream-owned by a transaction-aware adapter or the enclave SDK.

use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

/// Version of the neutral Taproot structural contract.
pub const TAPROOT_STRUCTURAL_API_VERSION: u16 = 1;
/// Version of the neutral Miniscript metadata handoff contract.
pub const MINISCRIPT_HANDOFF_API_VERSION: u16 = 1;

/// BIP-341 witness version for P2TR outputs.
pub const P2TR_WITNESS_VERSION: u8 = 1;
/// BIP-341 witness-program length for P2TR outputs.
pub const P2TR_WITNESS_PROGRAM_BYTES: usize = 32;
/// BIP-340 Schnorr signature length without an explicit sighash byte.
pub const KEY_PATH_SIGNATURE_BYTES: usize = 64;
/// BIP-340 Schnorr signature length with an explicit sighash byte.
pub const KEY_PATH_SIGNATURE_WITH_SIGHASH_BYTES: usize = 65;
/// The first byte and opaque internal key in a BIP-341 control block.
pub const TAPROOT_CONTROL_BLOCK_BASE_BYTES: usize = 33;
/// One opaque Merkle-path node in a BIP-341 control block.
pub const TAPROOT_MERKLE_PATH_NODE_BYTES: usize = 32;
/// Maximum BIP-341 Merkle-path depth.
pub const MAX_TAPROOT_MERKLE_DEPTH: usize = 128;
/// Maximum BIP-341 control-block length: `33 + 32 * 128` bytes.
pub const MAX_TAPROOT_CONTROL_BLOCK_BYTES: usize =
    TAPROOT_CONTROL_BLOCK_BASE_BYTES + TAPROOT_MERKLE_PATH_NODE_BYTES * MAX_TAPROOT_MERKLE_DEPTH;
/// Current BIP-342 Tapscript leaf version after masking the parity bit.
pub const TAPSCRIPT_LEAF_VERSION: u8 = 0xc0;

/// The only claim made by this module's successful validations.
///
/// Keeping the claim explicit prevents a structural result from being mistaken for a signature,
/// key-tweak, Taproot-commitment, or script-execution result.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ValidationClaim {
    /// Only public byte-shape or static metadata invariants were checked.
    StructuralOnly,
}

impl ValidationClaim {
    /// Returns whether cryptographic verification was performed.
    pub const fn cryptographic_verification_performed(self) -> bool {
        false
    }

    /// Returns whether transaction or script runtime execution was performed.
    pub const fn runtime_execution_performed(self) -> bool {
        false
    }
}

/// Stable category for errors returned by the core boundary validators.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinBoundaryErrorCategory {
    /// The supplied shape or metadata cannot satisfy the contract.
    Malformed,
    /// The value is well-formed but outside this API version's supported surface.
    Unsupported,
    /// A downstream parser, compiler, cryptographic verifier, or runtime owns the decision.
    DownstreamOwned,
}

impl BitcoinBoundaryErrorCategory {
    /// Returns the stable machine-readable category.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::Unsupported => "unsupported",
            Self::DownstreamOwned => "downstream_owned",
        }
    }
}

/// Stable machine-readable error code for the Taproot/Miniscript boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BitcoinBoundaryErrorCode {
    EmptyWitness,
    UnsupportedWitnessVersion,
    WitnessProgramWrongLength,
    KeyPathSignatureWrongLength,
    KeyPathSignatureZeroSighash,
    ScriptPathWitnessTooShort,
    WitnessPositionOverflow,
    ControlBlockTooShort,
    ControlBlockLengthMisaligned,
    ControlBlockDepthExceeded,
    UnknownTaprootLeafVersion,
    UnsupportedMiniscriptApiVersion,
    UnsupportedMiniscriptContext,
    MissingStaticMetadataCapability,
    MissingStructuralHandoffCapability,
    InvalidMiniscriptMetadata,
    MiniscriptContextMismatch,
    DownstreamOwnedMiniscriptCapability,
}

impl BitcoinBoundaryErrorCode {
    /// Returns the stable machine-readable error code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyWitness => "empty_witness",
            Self::UnsupportedWitnessVersion => "unsupported_witness_version",
            Self::WitnessProgramWrongLength => "witness_program_wrong_length",
            Self::KeyPathSignatureWrongLength => "key_path_signature_wrong_length",
            Self::KeyPathSignatureZeroSighash => "key_path_signature_zero_sighash",
            Self::ScriptPathWitnessTooShort => "script_path_witness_too_short",
            Self::WitnessPositionOverflow => "witness_position_overflow",
            Self::ControlBlockTooShort => "control_block_too_short",
            Self::ControlBlockLengthMisaligned => "control_block_length_misaligned",
            Self::ControlBlockDepthExceeded => "control_block_depth_exceeded",
            Self::UnknownTaprootLeafVersion => "unknown_taproot_leaf_version",
            Self::UnsupportedMiniscriptApiVersion => "unsupported_miniscript_api_version",
            Self::UnsupportedMiniscriptContext => "unsupported_miniscript_context",
            Self::MissingStaticMetadataCapability => "missing_static_metadata_capability",
            Self::MissingStructuralHandoffCapability => "missing_structural_handoff_capability",
            Self::InvalidMiniscriptMetadata => "invalid_miniscript_metadata",
            Self::MiniscriptContextMismatch => "miniscript_context_mismatch",
            Self::DownstreamOwnedMiniscriptCapability => "downstream_owned_miniscript_capability",
        }
    }
}

/// Redacted, serde-stable error returned by the structural boundary validators.
///
/// Errors contain only category and code. They intentionally do not include raw transaction,
/// script, signature, control-block, key, or secret bytes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BitcoinBoundaryError {
    pub category: BitcoinBoundaryErrorCategory,
    pub code: BitcoinBoundaryErrorCode,
}

impl BitcoinBoundaryError {
    const fn malformed(code: BitcoinBoundaryErrorCode) -> Self {
        Self {
            category: BitcoinBoundaryErrorCategory::Malformed,
            code,
        }
    }

    const fn unsupported(code: BitcoinBoundaryErrorCode) -> Self {
        Self {
            category: BitcoinBoundaryErrorCategory::Unsupported,
            code,
        }
    }

    const fn downstream_owned(code: BitcoinBoundaryErrorCode) -> Self {
        Self {
            category: BitcoinBoundaryErrorCategory::DownstreamOwned,
            code,
        }
    }

    /// Returns the stable category string.
    pub const fn category_code(self) -> &'static str {
        self.category.code()
    }

    /// Returns the stable error-code string.
    pub const fn code_str(self) -> &'static str {
        self.code.code()
    }

    /// Returns whether the failure is a malformed input.
    pub const fn is_malformed(self) -> bool {
        matches!(self.category, BitcoinBoundaryErrorCategory::Malformed)
    }

    /// Returns whether the failure is unsupported by this API version.
    pub const fn is_unsupported(self) -> bool {
        matches!(self.category, BitcoinBoundaryErrorCategory::Unsupported)
    }

    /// Returns whether a downstream component owns the missing validation.
    pub const fn is_downstream_owned(self) -> bool {
        matches!(self.category, BitcoinBoundaryErrorCategory::DownstreamOwned)
    }
}

impl fmt::Display for BitcoinBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.category_code(), self.code_str())
    }
}

impl Error for BitcoinBoundaryError {}

/// Compatibility alias for callers using Taproot-specific terminology.
pub type TaprootInvariantError = BitcoinBoundaryError;
/// Compatibility alias for callers using Miniscript-specific terminology.
pub type MiniscriptInvariantError = BitcoinBoundaryError;

/// A validated P2TR witness-program shape with its opaque 32-byte output key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct P2trWitnessProgram {
    pub witness_version: u8,
    pub program_length_bytes: u8,
    pub program: [u8; P2TR_WITNESS_PROGRAM_BYTES],
    pub claim: ValidationClaim,
}

/// Validates the neutral BIP-341 P2TR witness-program shape.
pub fn validate_p2tr_witness_program(
    witness_version: u8,
    program: &[u8],
) -> Result<P2trWitnessProgram, TaprootInvariantError> {
    if witness_version != P2TR_WITNESS_VERSION {
        return Err(BitcoinBoundaryError::unsupported(
            BitcoinBoundaryErrorCode::UnsupportedWitnessVersion,
        ));
    }

    if program.len() != P2TR_WITNESS_PROGRAM_BYTES {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::WitnessProgramWrongLength,
        ));
    }

    let program = <[u8; P2TR_WITNESS_PROGRAM_BYTES]>::try_from(program).map_err(|_| {
        BitcoinBoundaryError::malformed(BitcoinBoundaryErrorCode::WitnessProgramWrongLength)
    })?;

    Ok(P2trWitnessProgram {
        witness_version,
        program_length_bytes: P2TR_WITNESS_PROGRAM_BYTES as u8,
        program,
        claim: ValidationClaim::StructuralOnly,
    })
}

/// Shape-only metadata for a key-path Schnorr signature.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyPathSignatureShape {
    pub length_bytes: u8,
    /// Present only for a 65-byte signature with a non-zero explicit sighash byte.
    pub explicit_sighash_byte: Option<u8>,
    pub claim: ValidationClaim,
}

/// Validates the BIP-341 key-path signature length and explicit-sighash shape.
pub fn validate_key_path_signature(
    signature: &[u8],
) -> Result<KeyPathSignatureShape, TaprootInvariantError> {
    match signature.len() {
        KEY_PATH_SIGNATURE_BYTES => Ok(KeyPathSignatureShape {
            length_bytes: KEY_PATH_SIGNATURE_BYTES as u8,
            explicit_sighash_byte: None,
            claim: ValidationClaim::StructuralOnly,
        }),
        KEY_PATH_SIGNATURE_WITH_SIGHASH_BYTES => {
            let sighash_byte = signature[KEY_PATH_SIGNATURE_BYTES];
            if sighash_byte == 0 {
                return Err(BitcoinBoundaryError::malformed(
                    BitcoinBoundaryErrorCode::KeyPathSignatureZeroSighash,
                ));
            }

            Ok(KeyPathSignatureShape {
                length_bytes: KEY_PATH_SIGNATURE_WITH_SIGHASH_BYTES as u8,
                explicit_sighash_byte: Some(sighash_byte),
                claim: ValidationClaim::StructuralOnly,
            })
        }
        _ => Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::KeyPathSignatureWrongLength,
        )),
    }
}

/// The parity bit carried in the low bit of a Taproot control byte.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaprootParity {
    Even,
    Odd,
}

impl TaprootParity {
    const fn from_control_byte(control_byte: u8) -> Self {
        if control_byte & 1 == 0 {
            Self::Even
        } else {
            Self::Odd
        }
    }
}

/// Support classification for a masked Taproot leaf version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaprootLeafVersionSupport {
    /// Encoded control bytes `0xc0` and `0xc1` both mask to current BIP-342 Tapscript `0xc0`.
    CurrentTapscript,
    /// A structurally shaped future/unknown version that must be handed downstream.
    FutureOrUnknown { leaf_version: u8 },
}

impl TaprootLeafVersionSupport {
    /// Returns whether this leaf version is the currently specified BIP-342 Tapscript version.
    pub const fn is_current_tapscript(&self) -> bool {
        matches!(self, Self::CurrentTapscript)
    }
}

/// Classifies an encoded control-byte leaf version without interpreting script bytes.
pub const fn classify_taproot_leaf_version(encoded_control_byte: u8) -> TaprootLeafVersionSupport {
    let leaf_version = encoded_control_byte & 0xfe;
    if leaf_version == TAPSCRIPT_LEAF_VERSION {
        TaprootLeafVersionSupport::CurrentTapscript
    } else {
        TaprootLeafVersionSupport::FutureOrUnknown { leaf_version }
    }
}

/// Shape-only metadata for a BIP-341 control block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlBlockShape {
    pub serialized_length_bytes: u16,
    pub control_byte: u8,
    pub parity: TaprootParity,
    /// The control byte with its low parity bit masked off.
    pub leaf_version: u8,
    pub leaf_version_support: TaprootLeafVersionSupport,
    /// Opaque 32-byte internal-key encoding; no curve or point validation is performed.
    pub internal_key: [u8; P2TR_WITNESS_PROGRAM_BYTES],
    pub merkle_path_depth: u8,
    pub claim: ValidationClaim,
}

/// Inspects BIP-341 control-block shape, preserving unknown leaf versions for handoff.
///
/// This function validates only the `33 + 32m` length form, the `m <= 128` bound, and the
/// positions of the control byte and opaque internal key. It does not verify a curve point,
/// Taproot commitment, Merkle path, or script execution.
pub fn inspect_control_block(
    control_block: &[u8],
) -> Result<ControlBlockShape, TaprootInvariantError> {
    if control_block.len() < TAPROOT_CONTROL_BLOCK_BASE_BYTES {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::ControlBlockTooShort,
        ));
    }

    let path_bytes = control_block.len() - TAPROOT_CONTROL_BLOCK_BASE_BYTES;
    if path_bytes % TAPROOT_MERKLE_PATH_NODE_BYTES != 0 {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::ControlBlockLengthMisaligned,
        ));
    }

    let merkle_path_depth = path_bytes / TAPROOT_MERKLE_PATH_NODE_BYTES;
    if merkle_path_depth > MAX_TAPROOT_MERKLE_DEPTH {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::ControlBlockDepthExceeded,
        ));
    }

    let control_byte = control_block[0];
    let leaf_version = control_byte & 0xfe;
    let internal_key = <[u8; P2TR_WITNESS_PROGRAM_BYTES]>::try_from(&control_block[1..33])
        .map_err(|_| {
            BitcoinBoundaryError::malformed(BitcoinBoundaryErrorCode::ControlBlockTooShort)
        })?;

    Ok(ControlBlockShape {
        serialized_length_bytes: control_block.len() as u16,
        control_byte,
        parity: TaprootParity::from_control_byte(control_byte),
        leaf_version,
        leaf_version_support: classify_taproot_leaf_version(control_byte),
        internal_key,
        merkle_path_depth: merkle_path_depth as u8,
        claim: ValidationClaim::StructuralOnly,
    })
}

/// Validates a control block for the currently supported BIP-342 Tapscript leaf version.
///
/// Unknown/future leaf versions remain structurally inspectable through
/// [`inspect_control_block`] but fail closed here as downstream-owned rather than malformed.
pub fn validate_control_block(
    control_block: &[u8],
) -> Result<ControlBlockShape, TaprootInvariantError> {
    let shape = inspect_control_block(control_block)?;
    if !shape.leaf_version_support.is_current_tapscript() {
        return Err(BitcoinBoundaryError::downstream_owned(
            BitcoinBoundaryErrorCode::UnknownTaprootLeafVersion,
        ));
    }
    Ok(shape)
}

/// Position and size metadata for one witness element.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WitnessElementShape {
    pub position: u32,
    pub size_bytes: u32,
}

impl WitnessElementShape {
    fn from_element(position: usize, element: &[u8]) -> Result<Self, TaprootInvariantError> {
        Ok(Self {
            position: u32::try_from(position).map_err(|_| {
                BitcoinBoundaryError::malformed(BitcoinBoundaryErrorCode::WitnessPositionOverflow)
            })?,
            size_bytes: u32::try_from(element.len()).map_err(|_| {
                BitcoinBoundaryError::malformed(BitcoinBoundaryErrorCode::WitnessPositionOverflow)
            })?,
        })
    }
}

/// Shape classification for a Taproot witness after optional annex removal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "spend_path", rename_all = "snake_case")]
pub enum TaprootWitnessClassification {
    KeyPath {
        signature: KeyPathSignatureShape,
        annex: Option<WitnessElementShape>,
        claim: ValidationClaim,
    },
    ScriptPath {
        classification: TaprootScriptPathClassification,
    },
}

impl TaprootWitnessClassification {
    /// Returns the explicit structural-only claim.
    pub const fn claim(&self) -> ValidationClaim {
        ValidationClaim::StructuralOnly
    }
}

/// Structural classification of a Taproot script-path witness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaprootScriptPathClassification {
    pub annex: Option<WitnessElementShape>,
    pub script_leaf: WitnessElementShape,
    pub control_block: ControlBlockShape,
    /// Number of initial witness elements passed as script arguments by the downstream runtime.
    pub stack_argument_count: u32,
    pub leaf_version_support: TaprootLeafVersionSupport,
    pub claim: ValidationClaim,
}

/// Inspects key-path or script-path witness positions and public byte-shape invariants.
///
/// Unknown/future leaf versions are returned as `FutureOrUnknown` so a downstream implementation
/// can decide how to handle them. Use [`validate_taproot_witness`] for fail-closed support checks.
pub fn inspect_taproot_witness(
    witness: &[Vec<u8>],
) -> Result<TaprootWitnessClassification, TaprootInvariantError> {
    if witness.is_empty() {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::EmptyWitness,
        ));
    }

    let annex = if witness.len() >= 2
        && witness.last().and_then(|element| element.first()).copied() == Some(0x50)
    {
        Some(WitnessElementShape::from_element(
            witness.len() - 1,
            witness.last().expect("last element exists"),
        )?)
    } else {
        None
    };

    let effective_len = witness.len() - usize::from(annex.is_some());
    if effective_len == 1 {
        let signature = validate_key_path_signature(&witness[0])?;
        return Ok(TaprootWitnessClassification::KeyPath {
            signature,
            annex,
            claim: ValidationClaim::StructuralOnly,
        });
    }

    if effective_len < 2 {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::ScriptPathWitnessTooShort,
        ));
    }

    let script_position = effective_len - 2;
    let control_position = effective_len - 1;
    let script_leaf =
        WitnessElementShape::from_element(script_position, &witness[script_position])?;
    let control_block = inspect_control_block(&witness[control_position])?;
    let stack_argument_count = u32::try_from(script_position).map_err(|_| {
        BitcoinBoundaryError::malformed(BitcoinBoundaryErrorCode::WitnessPositionOverflow)
    })?;

    Ok(TaprootWitnessClassification::ScriptPath {
        classification: TaprootScriptPathClassification {
            annex,
            script_leaf,
            control_block: control_block.clone(),
            stack_argument_count,
            leaf_version_support: control_block.leaf_version_support,
            claim: ValidationClaim::StructuralOnly,
        },
    })
}

/// Validates a Taproot witness shape and rejects unknown/future leaf versions fail-closed.
pub fn validate_taproot_witness(
    witness: &[Vec<u8>],
) -> Result<TaprootWitnessClassification, TaprootInvariantError> {
    let classification = inspect_taproot_witness(witness)?;
    if let TaprootWitnessClassification::ScriptPath { classification } = &classification {
        if !classification.leaf_version_support.is_current_tapscript() {
            return Err(BitcoinBoundaryError::downstream_owned(
                BitcoinBoundaryErrorCode::UnknownTaprootLeafVersion,
            ));
        }
    }
    Ok(classification)
}

/// Context in which a static Miniscript metadata handoff is intended to be consumed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum MiniscriptContext {
    /// A Miniscript/Tapscript leaf in a P2TR script path.
    TaprootScriptPath,
    /// A SegWit v0 witness script handoff.
    SegwitV0,
    /// A context whose parser/compiler belongs to a downstream owner.
    Other(String),
}

impl MiniscriptContext {
    /// Returns whether this context is covered by the neutral metadata contract.
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::TaprootScriptPath | Self::SegwitV0)
    }
}

/// Coarse static policy category supplied by a downstream Miniscript analyzer.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MiniscriptPolicyKind {
    SingleKey,
    Threshold,
    Timelock,
    Hashlock,
    Composite,
}

/// Public, non-secret, compiler-produced Miniscript metadata.
///
/// This is intentionally not a policy expression, descriptor, key list, preimage, signature, or
/// satisfaction witness. Core validates relationships among the fields but does not derive them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MiniscriptPolicyMetadata {
    pub policy_kind: MiniscriptPolicyKind,
    pub required_signatures: u16,
    pub candidate_signers: u16,
    pub max_satisfaction_elements: u16,
    pub uses_timelock: bool,
    pub uses_hashlock: bool,
    pub uses_checksigadd: bool,
}

/// Capability named by a Miniscript handoff.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MiniscriptCapability {
    /// Validate and carry public static metadata.
    StaticMetadata,
    /// Carry a structural script-path/leaf handoff without interpreting it.
    StructuralHandoff,
    /// Parse or compile a policy into Miniscript/Bitcoin Script.
    Compilation,
    /// Produce or validate a satisfaction witness.
    Satisfaction,
    /// Execute or interpret a Miniscript/Tapscript policy.
    Execution,
    /// Verify keys, commitments, signatures, or other cryptographic claims.
    CryptographicVerification,
}

impl MiniscriptCapability {
    /// Returns whether this capability is supported by this core module.
    pub const fn is_core_supported(self) -> bool {
        matches!(self, Self::StaticMetadata | Self::StructuralHandoff)
    }

    /// Returns the stable ownership classification.
    pub const fn owner(self) -> MiniscriptCapabilityOwner {
        if self.is_core_supported() {
            MiniscriptCapabilityOwner::Core
        } else {
            MiniscriptCapabilityOwner::DownstreamOwned
        }
    }

    /// Returns all capability values in stable matrix order.
    pub const fn all() -> [Self; 6] {
        [
            Self::StaticMetadata,
            Self::StructuralHandoff,
            Self::Compilation,
            Self::Satisfaction,
            Self::Execution,
            Self::CryptographicVerification,
        ]
    }
}

/// Ownership classification for a Miniscript capability.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MiniscriptCapabilityOwner {
    Core,
    DownstreamOwned,
}

/// Neutral handoff from a downstream Miniscript analyzer to core.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MiniscriptHandoff {
    pub api_version: u16,
    pub context: MiniscriptContext,
    pub metadata: MiniscriptPolicyMetadata,
    pub requested_capabilities: Vec<MiniscriptCapability>,
}

/// Result of validating a supported, structural-only Miniscript handoff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MiniscriptHandoffAssessment {
    pub api_version: u16,
    pub context: MiniscriptContext,
    pub accepted_capabilities: Vec<MiniscriptCapability>,
    pub downstream_owned_capabilities: Vec<MiniscriptCapability>,
    pub claim: ValidationClaim,
}

/// Validates public Miniscript metadata without parsing, compiling, satisfying, or executing it.
pub fn validate_miniscript_policy_metadata(
    context: &MiniscriptContext,
    metadata: &MiniscriptPolicyMetadata,
) -> Result<(), MiniscriptInvariantError> {
    if !context.is_supported() {
        return Err(BitcoinBoundaryError::downstream_owned(
            BitcoinBoundaryErrorCode::UnsupportedMiniscriptContext,
        ));
    }

    if metadata.required_signatures > metadata.candidate_signers {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata,
        ));
    }

    match metadata.policy_kind {
        MiniscriptPolicyKind::SingleKey
            if metadata.required_signatures != 1 || metadata.candidate_signers != 1 =>
        {
            return Err(BitcoinBoundaryError::malformed(
                BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata,
            ));
        }
        MiniscriptPolicyKind::Threshold if metadata.required_signatures == 0 => {
            return Err(BitcoinBoundaryError::malformed(
                BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata,
            ));
        }
        MiniscriptPolicyKind::Timelock if !metadata.uses_timelock => {
            return Err(BitcoinBoundaryError::malformed(
                BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata,
            ));
        }
        MiniscriptPolicyKind::Hashlock if !metadata.uses_hashlock => {
            return Err(BitcoinBoundaryError::malformed(
                BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata,
            ));
        }
        _ => {}
    }

    if metadata.uses_checksigadd && !matches!(context, MiniscriptContext::TaprootScriptPath) {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::MiniscriptContextMismatch,
        ));
    }

    Ok(())
}

/// Validates a Miniscript handoff and returns only core-owned structural capabilities.
pub fn validate_miniscript_handoff(
    handoff: &MiniscriptHandoff,
) -> Result<MiniscriptHandoffAssessment, MiniscriptInvariantError> {
    if handoff.api_version != MINISCRIPT_HANDOFF_API_VERSION {
        return Err(BitcoinBoundaryError::unsupported(
            BitcoinBoundaryErrorCode::UnsupportedMiniscriptApiVersion,
        ));
    }

    validate_miniscript_policy_metadata(&handoff.context, &handoff.metadata)?;

    if !handoff
        .requested_capabilities
        .contains(&MiniscriptCapability::StaticMetadata)
    {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::MissingStaticMetadataCapability,
        ));
    }

    if !handoff
        .requested_capabilities
        .contains(&MiniscriptCapability::StructuralHandoff)
    {
        return Err(BitcoinBoundaryError::malformed(
            BitcoinBoundaryErrorCode::MissingStructuralHandoffCapability,
        ));
    }

    if handoff
        .requested_capabilities
        .iter()
        .any(|capability| !capability.is_core_supported())
    {
        return Err(BitcoinBoundaryError::downstream_owned(
            BitcoinBoundaryErrorCode::DownstreamOwnedMiniscriptCapability,
        ));
    }

    let downstream_owned_capabilities = MiniscriptCapability::all()
        .into_iter()
        .filter(|capability| !capability.is_core_supported())
        .collect();

    Ok(MiniscriptHandoffAssessment {
        api_version: handoff.api_version,
        context: handoff.context.clone(),
        accepted_capabilities: handoff.requested_capabilities.clone(),
        downstream_owned_capabilities,
        claim: ValidationClaim::StructuralOnly,
    })
}
