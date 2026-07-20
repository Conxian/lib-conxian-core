//! Versioned, fail-closed BIP-110 transaction preflight contract.
//!
//! This module owns the platform-neutral request, result, diagnostic, and operation-context
//! types that downstream transaction builders can use before signing or routing. It validates
//! caller-supplied classified byte measurements; it does not parse transactions, interpret
//! scripts, perform cryptographic validation, or observe network state.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{Bip110Compliance, Bip110TransactionShape, Bip110Violation};

/// Version of the serializable Core BIP-110 preflight contract.
pub const BIP110_PREFLIGHT_API_VERSION: u16 = 1;

/// BIP-110's maximum serialized Taproot control-block size.
pub const MAX_TAPROOT_CONTROL_BLOCK_BYTES: usize = 257;

/// Identifies whether a preflight request is made before construction or after serialization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightPhase {
    /// Validate measurements supplied while a transaction is being planned or constructed.
    PreConstruction,
    /// Validate measurements classified from the serialized transaction representation.
    PostSerialization,
}

impl Default for Bip110PreflightPhase {
    fn default() -> Self {
        Self::PreConstruction
    }
}

/// Identifies where the classified measurements came from.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110MeasurementSource {
    /// Measurements supplied by a caller before a complete transaction is serialized.
    CallerClassified,
    /// Measurements obtained by classifying the serialized transaction representation.
    SerializedTransaction,
}

impl Default for Bip110MeasurementSource {
    fn default() -> Self {
        Self::CallerClassified
    }
}

impl Bip110MeasurementSource {
    fn phase(self) -> Bip110PreflightPhase {
        match self {
            Self::CallerClassified => Bip110PreflightPhase::PreConstruction,
            Self::SerializedTransaction => Bip110PreflightPhase::PostSerialization,
        }
    }
}

/// Operation contexts understood by the preflight contract.
///
/// Only the ordinary Bitcoin size surfaces and an explicitly classified Taproot control block
/// are supported by version 1. Taproot execution contexts, Miniscript, DLC roles, and
/// Bitcoin-anchored protocol contexts are represented so that callers can be explicit, but they
/// fail closed until a downstream parser/classifier supplies a contract version that can validate
/// their additional semantics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110OperationContext {
    /// A complete ordinary Bitcoin transaction shape containing all four ordinary surfaces.
    OrdinaryTransaction,
    /// Ordinary output script data, including classified output pushdata and output ScriptPubKeys.
    OrdinaryOutput,
    /// An ordinary pushdata payload, excluding its opcode and length prefix.
    Pushdata,
    /// An OP_RETURN output with a full serialized ScriptPubKey measurement.
    OpReturn,
    /// A non-OP_RETURN output with a full serialized ScriptPubKey measurement.
    NonOpReturn,
    /// An applicable script-argument witness item, excluding its item-length prefix.
    WitnessScriptArgument,
    /// A Taproot key-path spend. Unsupported in version 1.
    TaprootKeyPath,
    /// A Taproot script-path spend. Unsupported in version 1.
    TaprootScriptPath,
    /// A separately classified serialized Taproot control block.
    TaprootControlBlock,
    /// A Taproot leaf script. Unsupported in version 1.
    Tapleaf,
    /// A Tapscript execution context. Unsupported in version 1.
    Tapscript,
    /// A Miniscript policy or satisfaction context. Unsupported in version 1.
    Miniscript,
    /// A DLC funding transaction. Unsupported in version 1.
    DlcFunding,
    /// A DLC refund transaction. Unsupported in version 1.
    DlcRefund,
    /// A DLC contract execution transaction (CET). Unsupported in version 1.
    DlcCet,
    /// A Lightning commitment transaction. Unsupported in version 1.
    LightningCommitment,
    /// A Lightning mutual or unilateral closing transaction. Unsupported in version 1.
    LightningClosing,
    /// A Lightning justice transaction. Unsupported in version 1.
    LightningJustice,
    /// A Lightning HTLC timeout or success transaction. Unsupported in version 1.
    LightningHtlc,
    /// An RGB Bitcoin anchor. Unsupported in version 1.
    RgbAnchor,
    /// A Babylon staking transaction. Unsupported in version 1.
    BabylonStaking,
    /// A Babylon delegation transaction. Unsupported in version 1.
    BabylonDelegation,
    /// A Babylon unbonding transaction. Unsupported in version 1.
    BabylonUnbonding,
    /// A Babylon withdrawal transaction. Unsupported in version 1.
    BabylonWithdrawal,
    /// A Babylon checkpoint transaction. Unsupported in version 1.
    BabylonCheckpoint,
    /// A Stacks/sBTC peg-in transaction. Unsupported in version 1.
    StacksSbtcPegIn,
    /// A Stacks/sBTC peg-out transaction. Unsupported in version 1.
    StacksSbtcPegOut,
    /// A Stacks/sBTC mint transaction. Unsupported in version 1.
    StacksSbtcMint,
    /// A Stacks/sBTC burn transaction. Unsupported in version 1.
    StacksSbtcBurn,
    /// An explicitly named context not covered by this contract version.
    Other(String),
    /// An absent or unclassified operation context.
    Unknown,
}

impl Default for Bip110OperationContext {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Bip110OperationContext {
    fn is_malformed(&self) -> bool {
        matches!(self, Self::Other(name) if name.trim().is_empty())
    }

    fn is_supported(&self) -> bool {
        matches!(
            self,
            Self::OrdinaryTransaction
                | Self::OrdinaryOutput
                | Self::Pushdata
                | Self::OpReturn
                | Self::NonOpReturn
                | Self::WitnessScriptArgument
                | Self::TaprootControlBlock
        )
    }

    fn allows(&self, measurement: Bip110MeasurementKind) -> bool {
        match self {
            Self::OrdinaryTransaction => matches!(
                measurement,
                Bip110MeasurementKind::PushdataPayload
                    | Bip110MeasurementKind::OpReturnScriptPubKey
                    | Bip110MeasurementKind::NonOpReturnScriptPubKey
                    | Bip110MeasurementKind::WitnessScriptArgument
            ),
            Self::OrdinaryOutput => matches!(
                measurement,
                Bip110MeasurementKind::PushdataPayload
                    | Bip110MeasurementKind::OpReturnScriptPubKey
                    | Bip110MeasurementKind::NonOpReturnScriptPubKey
            ),
            Self::Pushdata => matches!(measurement, Bip110MeasurementKind::PushdataPayload),
            Self::OpReturn => matches!(
                measurement,
                Bip110MeasurementKind::PushdataPayload
                    | Bip110MeasurementKind::OpReturnScriptPubKey
            ),
            Self::NonOpReturn => matches!(
                measurement,
                Bip110MeasurementKind::PushdataPayload
                    | Bip110MeasurementKind::NonOpReturnScriptPubKey
            ),
            Self::WitnessScriptArgument => {
                matches!(measurement, Bip110MeasurementKind::WitnessScriptArgument)
            }
            Self::TaprootControlBlock => {
                matches!(measurement, Bip110MeasurementKind::TaprootControlBlock)
            }
            _ => false,
        }
    }
}

/// A classified measurement surface carried by a preflight request or diagnostic.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110MeasurementKind {
    /// Payload bytes of an applicable pushdata operation.
    PushdataPayload,
    /// Complete serialized ScriptPubKey bytes of an OP_RETURN output.
    OpReturnScriptPubKey,
    /// Complete serialized ScriptPubKey bytes of a non-OP_RETURN output.
    NonOpReturnScriptPubKey,
    /// Bytes in one applicable script-argument witness item.
    WitnessScriptArgument,
    /// Complete serialized bytes of a Taproot control block.
    TaprootControlBlock,
}

/// Classified byte measurements used by the preflight contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightMeasurements {
    /// Whether the measurements are planned values or values classified from serialization.
    pub source: Bip110MeasurementSource,
    /// Ordinary transaction size metadata. Control blocks are intentionally not stored here.
    pub shape: Bip110TransactionShape,
    /// Separately classified Taproot control-block sizes.
    pub taproot_control_block_sizes_bytes: Vec<usize>,
}

impl Bip110PreflightMeasurements {
    /// Creates measurements with an explicit source and optional control-block sizes.
    pub fn new(
        source: Bip110MeasurementSource,
        shape: Bip110TransactionShape,
        taproot_control_block_sizes_bytes: Vec<usize>,
    ) -> Self {
        Self {
            source,
            shape,
            taproot_control_block_sizes_bytes,
        }
    }

    /// Creates pre-construction measurements for ordinary Bitcoin surfaces.
    pub fn pre_construction(shape: Bip110TransactionShape) -> Self {
        Self::new(Bip110MeasurementSource::CallerClassified, shape, Vec::new())
    }

    /// Creates pre-construction measurements including separately classified control blocks.
    pub fn pre_construction_with_control_blocks(
        shape: Bip110TransactionShape,
        taproot_control_block_sizes_bytes: Vec<usize>,
    ) -> Self {
        Self::new(
            Bip110MeasurementSource::CallerClassified,
            shape,
            taproot_control_block_sizes_bytes,
        )
    }

    /// Creates post-serialization measurements for ordinary Bitcoin surfaces.
    pub fn post_serialization(shape: Bip110TransactionShape) -> Self {
        Self::new(
            Bip110MeasurementSource::SerializedTransaction,
            shape,
            Vec::new(),
        )
    }

    /// Creates post-serialization measurements including separately classified control blocks.
    pub fn post_serialization_with_control_blocks(
        shape: Bip110TransactionShape,
        taproot_control_block_sizes_bytes: Vec<usize>,
    ) -> Self {
        Self::new(
            Bip110MeasurementSource::SerializedTransaction,
            shape,
            taproot_control_block_sizes_bytes,
        )
    }
}

/// Versioned request for Core-side BIP-110 preflight.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightRequest {
    /// Version of the request contract.
    pub api_version: u16,
    /// Whether the caller is checking planned or serialized measurements.
    pub phase: Bip110PreflightPhase,
    /// Explicit operation context for the supplied measurements.
    pub context: Bip110OperationContext,
    /// Classified measurements. `None` is distinct from an intentionally empty vector.
    pub measurements: Option<Bip110PreflightMeasurements>,
}

impl Bip110PreflightRequest {
    /// Creates a request for the current contract version.
    pub fn new(
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
        measurements: Bip110PreflightMeasurements,
    ) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            phase,
            context,
            measurements: Some(measurements),
        }
    }

    /// Creates a request that intentionally omits measurements, useful for error handling tests
    /// and adapters that have not yet produced classified byte inputs.
    pub fn without_measurements(
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
    ) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            phase,
            context,
            measurements: None,
        }
    }

    /// Returns a copy of this request using another API version.
    pub fn with_api_version(mut self, api_version: u16) -> Self {
        self.api_version = api_version;
        self
    }
}

/// Stable codes for contract-level preflight failures.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightErrorCode {
    /// The request version is not supported by this Core contract.
    ApiVersionMismatch,
    /// The request shape is malformed before measurement validation can begin.
    MalformedRequest,
    /// The request phase and measurement source disagree.
    PhaseMismatch,
    /// The operation context is not supported by this contract version.
    UnsupportedContext,
    /// The request did not contain classified measurements.
    MissingMeasurementData,
    /// Measurements were supplied for a surface not allowed by the context.
    InvalidMeasurementData,
    /// Enforcement was explicitly disabled, so no compliant result is emitted.
    EnforcementDisabled,
}

/// A serializable, deterministic Core preflight error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightError {
    /// Version of the error contract.
    pub api_version: u16,
    /// Stable error code.
    pub code: Bip110PreflightErrorCode,
    /// Context involved in the failed request, when available.
    pub context: Option<Bip110OperationContext>,
    /// Phase involved in the failed request, when available.
    pub phase: Option<Bip110PreflightPhase>,
    /// Measurement surface involved in the failed request, when available.
    pub measurement_kind: Option<Bip110MeasurementKind>,
    /// Expected API version for an API-version mismatch.
    pub expected_api_version: Option<u16>,
    /// Received API version for an API-version mismatch.
    pub received_api_version: Option<u16>,
    /// Expected phase for a phase mismatch.
    pub expected_phase: Option<Bip110PreflightPhase>,
    /// Measurement source phase for a phase mismatch.
    pub received_phase: Option<Bip110PreflightPhase>,
}

impl Bip110PreflightError {
    fn api_version_mismatch(received_api_version: u16) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::ApiVersionMismatch,
            context: None,
            phase: None,
            measurement_kind: None,
            expected_api_version: Some(BIP110_PREFLIGHT_API_VERSION),
            received_api_version: Some(received_api_version),
            expected_phase: None,
            received_phase: None,
        }
    }

    fn malformed(context: Bip110OperationContext, phase: Bip110PreflightPhase) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::MalformedRequest,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: None,
            expected_api_version: None,
            received_api_version: None,
            expected_phase: None,
            received_phase: None,
        }
    }

    fn phase_mismatch(
        context: Bip110OperationContext,
        phase: Bip110PreflightPhase,
        received_phase: Bip110PreflightPhase,
    ) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::PhaseMismatch,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: None,
            expected_api_version: None,
            received_api_version: None,
            expected_phase: Some(phase),
            received_phase: Some(received_phase),
        }
    }

    fn unsupported(context: Bip110OperationContext, phase: Bip110PreflightPhase) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::UnsupportedContext,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: None,
            expected_api_version: None,
            received_api_version: None,
            expected_phase: None,
            received_phase: None,
        }
    }

    fn missing(context: Bip110OperationContext, phase: Bip110PreflightPhase) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::MissingMeasurementData,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: None,
            expected_api_version: None,
            received_api_version: None,
            expected_phase: None,
            received_phase: None,
        }
    }

    fn invalid(
        context: Bip110OperationContext,
        phase: Bip110PreflightPhase,
        measurement_kind: Bip110MeasurementKind,
    ) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::InvalidMeasurementData,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: Some(measurement_kind),
            expected_api_version: None,
            received_api_version: None,
            expected_phase: None,
            received_phase: None,
        }
    }

    fn disabled(context: Bip110OperationContext, phase: Bip110PreflightPhase) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            code: Bip110PreflightErrorCode::EnforcementDisabled,
            context: Some(context),
            phase: Some(phase),
            measurement_kind: None,
            expected_api_version: None,
            received_api_version: None,
            expected_phase: None,
            received_phase: None,
        }
    }
}

impl fmt::Display for Bip110PreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Bip110PreflightErrorCode::ApiVersionMismatch => write!(
                f,
                "BIP-110 preflight API version mismatch: received {}, expected {}",
                self.received_api_version.unwrap_or_default(),
                self.expected_api_version
                    .unwrap_or(BIP110_PREFLIGHT_API_VERSION)
            ),
            Bip110PreflightErrorCode::MalformedRequest => {
                write!(f, "malformed BIP-110 preflight request")
            }
            Bip110PreflightErrorCode::PhaseMismatch => write!(
                f,
                "BIP-110 preflight phase mismatch: request is {:?}, measurements are {:?}",
                self.expected_phase, self.received_phase
            ),
            Bip110PreflightErrorCode::UnsupportedContext => write!(
                f,
                "unsupported BIP-110 preflight operation context: {:?}",
                self.context
            ),
            Bip110PreflightErrorCode::MissingMeasurementData => {
                write!(f, "missing BIP-110 preflight measurement data")
            }
            Bip110PreflightErrorCode::InvalidMeasurementData => write!(
                f,
                "invalid BIP-110 preflight measurement data for {:?}",
                self.measurement_kind
            ),
            Bip110PreflightErrorCode::EnforcementDisabled => {
                write!(f, "BIP-110 preflight enforcement is disabled")
            }
        }
    }
}

impl std::error::Error for Bip110PreflightError {}

/// Stable codes for ordinary BIP-110 size violations returned by preflight.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightViolationCode {
    /// An applicable pushdata payload is larger than the configured limit.
    PushdataExceedsLimit,
    /// An OP_RETURN ScriptPubKey is larger than the configured limit.
    OpReturnExceedsLimit,
    /// A non-OP_RETURN ScriptPubKey is larger than the configured limit.
    NonOpReturnScriptPubKeyExceedsLimit,
    /// An applicable witness script argument is larger than the configured limit.
    WitnessScriptArgumentExceedsLimit,
    /// A Taproot control block is larger than the BIP-110 limit.
    TaprootControlBlockExceedsLimit,
}

/// A deterministic, serializable BIP-110 compliance violation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightViolation {
    /// Stable violation code.
    pub code: Bip110PreflightViolationCode,
    /// Authoritative byte measurement for the violating surface.
    pub size_bytes: usize,
    /// Inclusive maximum allowed size.
    pub max_bytes: usize,
}

impl Bip110PreflightViolation {
    fn from_existing(violation: Bip110Violation) -> Self {
        match violation {
            Bip110Violation::PushdataExceedsLimit { size, max } => Self {
                code: Bip110PreflightViolationCode::PushdataExceedsLimit,
                size_bytes: size,
                max_bytes: max,
            },
            Bip110Violation::OpReturnExceedsLimit { size, max } => Self {
                code: Bip110PreflightViolationCode::OpReturnExceedsLimit,
                size_bytes: size,
                max_bytes: max,
            },
            Bip110Violation::ScriptPubKeyExceedsLimit { size, max } => Self {
                code: Bip110PreflightViolationCode::NonOpReturnScriptPubKeyExceedsLimit,
                size_bytes: size,
                max_bytes: max,
            },
            Bip110Violation::WitnessElementExceedsLimit { size, max } => Self {
                code: Bip110PreflightViolationCode::WitnessScriptArgumentExceedsLimit,
                size_bytes: size,
                max_bytes: max,
            },
        }
    }

    fn taproot_control_block(size: usize) -> Self {
        Self {
            code: Bip110PreflightViolationCode::TaprootControlBlockExceedsLimit,
            size_bytes: size,
            max_bytes: MAX_TAPROOT_CONTROL_BLOCK_BYTES,
        }
    }
}

impl fmt::Display for Bip110PreflightViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BIP-110 preflight violation {:?}: {} bytes exceeds {} bytes",
            self.code, self.size_bytes, self.max_bytes
        )
    }
}

/// Successful request evaluation, which may be compliant or explicitly non-compliant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightResult {
    /// Version of the result contract.
    pub api_version: u16,
    /// Phase used for the evaluation.
    pub phase: Bip110PreflightPhase,
    /// Explicit context used for the evaluation.
    pub context: Bip110OperationContext,
    /// Source of the classified measurements.
    pub measurement_source: Bip110MeasurementSource,
    /// True only when all applicable checks pass and no violations are present.
    pub is_compliant: bool,
    /// All ordinary size violations, in deterministic surface order.
    pub violations: Vec<Bip110PreflightViolation>,
}

impl Bip110PreflightResult {
    /// Returns whether this result is a hard rejection rather than a warning-only outcome.
    pub fn is_rejected(&self) -> bool {
        !self.is_compliant
    }
}

/// Core-side BIP-110 preflight validator.
///
/// The validator composes the existing [`Bip110Compliance`] and
/// [`Bip110TransactionShape`] checks. The default `Bip110Compliance` remains disabled for
/// compatibility; callers must construct this validator with `Bip110Compliance::new()` or an
/// enabled custom-limits configuration to receive a compliance result.
#[derive(Debug, Clone)]
pub struct Bip110Preflight {
    compliance: Bip110Compliance,
}

impl Bip110Preflight {
    /// Creates a preflight validator from an existing compliance configuration.
    pub fn new(compliance: Bip110Compliance) -> Self {
        Self { compliance }
    }

    /// Creates an explicitly enabled validator using canonical BIP-110 limits.
    pub fn enabled() -> Self {
        Self::new(Bip110Compliance::new())
    }

    /// Creates an explicitly disabled validator. Requests fail with
    /// [`Bip110PreflightErrorCode::EnforcementDisabled`] rather than producing a compliant result.
    pub fn disabled() -> Self {
        Self::new(Bip110Compliance::disabled())
    }

    /// Returns the underlying compliance configuration.
    pub fn compliance(&self) -> &Bip110Compliance {
        &self.compliance
    }

    /// Returns whether this preflight validator is enforcing BIP-110 limits.
    pub fn is_enabled(&self) -> bool {
        self.compliance.is_enabled()
    }

    /// Evaluates one versioned request.
    pub fn preflight(
        &self,
        request: &Bip110PreflightRequest,
    ) -> Result<Bip110PreflightResult, Bip110PreflightError> {
        if request.api_version != BIP110_PREFLIGHT_API_VERSION {
            return Err(Bip110PreflightError::api_version_mismatch(
                request.api_version,
            ));
        }

        if request.context.is_malformed() {
            return Err(Bip110PreflightError::malformed(
                request.context.clone(),
                request.phase,
            ));
        }

        let measurements = request
            .measurements
            .as_ref()
            .ok_or_else(|| Bip110PreflightError::missing(request.context.clone(), request.phase))?;

        let received_phase = measurements.source.phase();
        if received_phase != request.phase {
            return Err(Bip110PreflightError::phase_mismatch(
                request.context.clone(),
                request.phase,
                received_phase,
            ));
        }

        if !request.context.is_supported() {
            return Err(Bip110PreflightError::unsupported(
                request.context.clone(),
                request.phase,
            ));
        }

        if !self.compliance.is_enabled() {
            return Err(Bip110PreflightError::disabled(
                request.context.clone(),
                request.phase,
            ));
        }

        self.validate_measurement_scope(request, measurements)?;

        let ordinary_result = self.compliance.validate_shape(&measurements.shape);
        let mut violations = ordinary_result
            .violations
            .into_iter()
            .map(Bip110PreflightViolation::from_existing)
            .collect::<Vec<_>>();

        for size in measurements
            .taproot_control_block_sizes_bytes
            .iter()
            .copied()
        {
            if size > MAX_TAPROOT_CONTROL_BLOCK_BYTES {
                violations.push(Bip110PreflightViolation::taproot_control_block(size));
            }
        }

        Ok(Bip110PreflightResult {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            phase: request.phase,
            context: request.context.clone(),
            measurement_source: measurements.source,
            is_compliant: violations.is_empty(),
            violations,
        })
    }

    /// Alias for [`Self::preflight`] for callers that use validation terminology.
    pub fn validate(
        &self,
        request: &Bip110PreflightRequest,
    ) -> Result<Bip110PreflightResult, Bip110PreflightError> {
        self.preflight(request)
    }

    fn validate_measurement_scope(
        &self,
        request: &Bip110PreflightRequest,
        measurements: &Bip110PreflightMeasurements,
    ) -> Result<(), Bip110PreflightError> {
        let populated = [
            (
                Bip110MeasurementKind::PushdataPayload,
                !measurements.shape.pushdata_sizes_bytes.is_empty(),
            ),
            (
                Bip110MeasurementKind::OpReturnScriptPubKey,
                !measurements
                    .shape
                    .op_return_script_pubkey_sizes_bytes
                    .is_empty(),
            ),
            (
                Bip110MeasurementKind::NonOpReturnScriptPubKey,
                !measurements
                    .shape
                    .non_op_return_script_pubkey_sizes_bytes
                    .is_empty(),
            ),
            (
                Bip110MeasurementKind::WitnessScriptArgument,
                !measurements.shape.witness_element_sizes_bytes.is_empty(),
            ),
            (
                Bip110MeasurementKind::TaprootControlBlock,
                !measurements.taproot_control_block_sizes_bytes.is_empty(),
            ),
        ];

        for (kind, is_populated) in populated {
            if is_populated && !request.context.allows(kind) {
                return Err(Bip110PreflightError::invalid(
                    request.context.clone(),
                    request.phase,
                    kind,
                ));
            }
        }

        Ok(())
    }
}

/// Performs BIP-110 preflight with an existing compliance configuration.
pub fn preflight(
    request: &Bip110PreflightRequest,
    compliance: &Bip110Compliance,
) -> Result<Bip110PreflightResult, Bip110PreflightError> {
    Bip110Preflight::new(compliance.clone()).preflight(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_model::bip110::{
        MAX_OP_RETURN_BYTES, MAX_PUSHDATA_BYTES, MAX_SCRIPT_PUBKEY_BYTES, MAX_WITNESS_ELEMENT_BYTES,
    };

    fn ordinary_shape() -> Bip110TransactionShape {
        Bip110TransactionShape::new(vec![], vec![], vec![], vec![])
    }

    fn make_request(
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
        measurements: Bip110PreflightMeasurements,
    ) -> Bip110PreflightRequest {
        Bip110PreflightRequest::new(phase, context, measurements)
    }

    #[test]
    fn ordinary_preflight_accepts_exact_boundaries_and_zero_or_empty_vectors() {
        let shape = Bip110TransactionShape::new(
            vec![0, MAX_PUSHDATA_BYTES],
            vec![0, MAX_OP_RETURN_BYTES],
            vec![0, MAX_SCRIPT_PUBKEY_BYTES],
            vec![0, MAX_WITNESS_ELEMENT_BYTES],
        );
        let request = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(shape),
        );

        let result = Bip110Preflight::enabled()
            .preflight(&request)
            .expect("exact boundaries should pass");

        assert!(result.is_compliant);
        assert!(result.violations.is_empty());

        let empty_request = make_request(
            Bip110PreflightPhase::PostSerialization,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::post_serialization(ordinary_shape()),
        );
        let empty_result = Bip110Preflight::enabled()
            .preflight(&empty_request)
            .expect("empty vectors should represent no classified occurrences");
        assert!(empty_result.is_compliant);
    }

    #[test]
    fn ordinary_preflight_retains_deterministic_multiple_violation_order() {
        let shape = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES + 1, MAX_PUSHDATA_BYTES + 2],
            vec![MAX_OP_RETURN_BYTES + 1],
            vec![MAX_SCRIPT_PUBKEY_BYTES + 1],
            vec![MAX_WITNESS_ELEMENT_BYTES + 1],
        );
        let request = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(shape),
        );

        let result = Bip110Preflight::enabled()
            .preflight(&request)
            .expect("size violations are a valid preflight result");

        assert!(!result.is_compliant);
        assert!(result.is_rejected());
        assert_eq!(
            result
                .violations
                .iter()
                .map(|violation| violation.code)
                .collect::<Vec<_>>(),
            vec![
                Bip110PreflightViolationCode::PushdataExceedsLimit,
                Bip110PreflightViolationCode::PushdataExceedsLimit,
                Bip110PreflightViolationCode::OpReturnExceedsLimit,
                Bip110PreflightViolationCode::NonOpReturnScriptPubKeyExceedsLimit,
                Bip110PreflightViolationCode::WitnessScriptArgumentExceedsLimit,
            ]
        );
    }

    #[test]
    fn control_block_boundary_is_inclusive_and_separate_from_witness_arguments() {
        let exact_request = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::TaprootControlBlock,
            Bip110PreflightMeasurements::pre_construction_with_control_blocks(
                ordinary_shape(),
                vec![MAX_TAPROOT_CONTROL_BLOCK_BYTES],
            ),
        );
        let exact_result = Bip110Preflight::enabled()
            .preflight(&exact_request)
            .expect("257-byte control block should pass");
        assert!(exact_result.is_compliant);

        let oversized_request = make_request(
            Bip110PreflightPhase::PostSerialization,
            Bip110OperationContext::TaprootControlBlock,
            Bip110PreflightMeasurements::post_serialization_with_control_blocks(
                ordinary_shape(),
                vec![MAX_TAPROOT_CONTROL_BLOCK_BYTES + 1],
            ),
        );
        let oversized_result = Bip110Preflight::enabled()
            .preflight(&oversized_request)
            .expect("oversized control block should be a violation result");
        assert_eq!(
            oversized_result.violations,
            vec![Bip110PreflightViolation {
                code: Bip110PreflightViolationCode::TaprootControlBlockExceedsLimit,
                size_bytes: 258,
                max_bytes: 257,
            }]
        );
    }

    #[test]
    fn unsupported_contexts_fail_closed_even_when_shape_sizes_pass() {
        let contexts = [
            Bip110OperationContext::TaprootKeyPath,
            Bip110OperationContext::TaprootScriptPath,
            Bip110OperationContext::Tapleaf,
            Bip110OperationContext::Tapscript,
            Bip110OperationContext::Miniscript,
            Bip110OperationContext::DlcFunding,
            Bip110OperationContext::DlcRefund,
            Bip110OperationContext::DlcCet,
            Bip110OperationContext::LightningCommitment,
            Bip110OperationContext::RgbAnchor,
            Bip110OperationContext::BabylonStaking,
            Bip110OperationContext::StacksSbtcPegIn,
            Bip110OperationContext::Other("future_context".to_owned()),
            Bip110OperationContext::Unknown,
        ];

        for context in contexts {
            let request = make_request(
                Bip110PreflightPhase::PreConstruction,
                context,
                Bip110PreflightMeasurements::pre_construction(ordinary_shape()),
            );
            let error = Bip110Preflight::enabled()
                .preflight(&request)
                .expect_err("unsupported contexts must not pass");
            assert_eq!(error.code, Bip110PreflightErrorCode::UnsupportedContext);
        }
    }

    #[test]
    fn errors_distinguish_missing_invalid_phase_version_and_disabled_requests() {
        let missing = Bip110PreflightRequest::without_measurements(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
        );
        assert_eq!(
            Bip110Preflight::enabled()
                .preflight(&missing)
                .expect_err("missing measurements must fail")
                .code,
            Bip110PreflightErrorCode::MissingMeasurementData
        );

        let malformed = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::Other("  ".to_owned()),
            Bip110PreflightMeasurements::pre_construction(ordinary_shape()),
        );
        assert_eq!(
            Bip110Preflight::enabled()
                .preflight(&malformed)
                .expect_err("blank custom contexts must fail as malformed")
                .code,
            Bip110PreflightErrorCode::MalformedRequest
        );

        let invalid = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::Pushdata,
            Bip110PreflightMeasurements::pre_construction(Bip110TransactionShape::new(
                vec![],
                vec![1],
                vec![],
                vec![],
            )),
        );
        assert_eq!(
            Bip110Preflight::enabled()
                .preflight(&invalid)
                .expect_err("context-inconsistent measurements must fail")
                .code,
            Bip110PreflightErrorCode::InvalidMeasurementData
        );

        let phase_mismatch = make_request(
            Bip110PreflightPhase::PostSerialization,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(ordinary_shape()),
        );
        assert_eq!(
            Bip110Preflight::enabled()
                .preflight(&phase_mismatch)
                .expect_err("phase/source mismatch must fail")
                .code,
            Bip110PreflightErrorCode::PhaseMismatch
        );

        let version_mismatch = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(ordinary_shape()),
        )
        .with_api_version(BIP110_PREFLIGHT_API_VERSION + 1);
        assert_eq!(
            Bip110Preflight::enabled()
                .preflight(&version_mismatch)
                .expect_err("unknown API version must fail")
                .code,
            Bip110PreflightErrorCode::ApiVersionMismatch
        );

        let disabled = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(Bip110TransactionShape::new(
                vec![usize::MAX],
                vec![],
                vec![],
                vec![],
            )),
        );
        assert_eq!(
            Bip110Preflight::disabled()
                .preflight(&disabled)
                .expect_err("disabled enforcement must not emit a compliant result")
                .code,
            Bip110PreflightErrorCode::EnforcementDisabled
        );
    }

    #[test]
    fn request_result_error_and_measurements_round_trip_through_json() {
        let request = make_request(
            Bip110PreflightPhase::PostSerialization,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::post_serialization(Bip110TransactionShape::new(
                vec![MAX_PUSHDATA_BYTES + 1],
                vec![],
                vec![],
                vec![],
            )),
        );
        let encoded_request = serde_json::to_string(&request).expect("request serializes");
        let decoded_request: Bip110PreflightRequest =
            serde_json::from_str(&encoded_request).expect("request deserializes");
        assert_eq!(decoded_request, request);

        let result = Bip110Preflight::enabled()
            .preflight(&request)
            .expect("oversized data should return a result");
        let encoded_result = serde_json::to_string(&result).expect("result serializes");
        let decoded_result: Bip110PreflightResult =
            serde_json::from_str(&encoded_result).expect("result deserializes");
        assert_eq!(decoded_result, result);

        let error = Bip110Preflight::enabled()
            .preflight(&Bip110PreflightRequest::without_measurements(
                Bip110PreflightPhase::PreConstruction,
                Bip110OperationContext::OrdinaryTransaction,
            ))
            .expect_err("missing measurements should produce an error");
        let encoded_error = serde_json::to_string(&error).expect("error serializes");
        let decoded_error: Bip110PreflightError =
            serde_json::from_str(&encoded_error).expect("error deserializes");
        assert_eq!(decoded_error, error);
    }

    #[test]
    fn compatibility_with_existing_shape_and_compliance_types_is_preserved() {
        let shape =
            Bip110TransactionShape::new(vec![MAX_PUSHDATA_BYTES + 1], vec![], vec![], vec![]);
        let existing = Bip110Compliance::new().validate_shape(&shape);
        assert!(!existing.is_compliant);
        assert_eq!(existing.violations.len(), 1);

        let request = make_request(
            Bip110PreflightPhase::PreConstruction,
            Bip110OperationContext::OrdinaryTransaction,
            Bip110PreflightMeasurements::pre_construction(shape),
        );
        let preflight_result = Bip110Preflight::enabled()
            .preflight(&request)
            .expect("preflight should compose existing validation");
        assert_eq!(preflight_result.violations.len(), existing.violations.len());

        let default_compliance = Bip110Compliance::default();
        assert!(!default_compliance.is_enabled());
        assert!(
            default_compliance
                .validate_shape(&Bip110TransactionShape::new(
                    vec![usize::MAX],
                    vec![],
                    vec![],
                    vec![],
                ))
                .is_compliant
        );
    }
}
