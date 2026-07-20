//! Versioned, platform-neutral BIP-110 transaction preflight contract.
//!
//! This module validates byte measurements supplied by a transaction-aware adapter. It does not
//! parse or construct Bitcoin transactions, execute scripts, select fees, or perform network I/O.
//! The request carries fixed-width `u64` measurements so it can cross platform and language
//! boundaries without exposing Rust's platform-sized `usize` fields. Validation converts those
//! measurements to [`Bip110TransactionShape`] with checked conversions before composing with the
//! canonical [`Bip110Compliance`] validator.

use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::bip110::Bip110TransactionShape;
use super::trust::{Bip110Compliance, Bip110ValidationResult, Bip110Violation};

/// Initial version of the serialized BIP-110 preflight contract.
pub const BIP110_PREFLIGHT_API_VERSION: u16 = 1;

/// Inclusive BIP-110 maximum for one explicitly classified Taproot control block.
pub const MAX_TAPROOT_CONTROL_BLOCK_BYTES: u64 = 257;

/// The point in the transaction lifecycle at which measurements are supplied.
///
/// Both phases use the same byte units and inclusive limits. In the pre-construction phase,
/// measurements describe the intended serialized transaction surfaces before final bytes exist.
/// In the post-serialization phase, measurements must be taken from the finalized serialized
/// transaction. Core does not construct or serialize the transaction in either phase.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightPhase {
    /// Validate measurements before a transaction is finalized or serialized.
    #[default]
    PreConstruction,
    /// Validate measurements taken from a finalized serialized transaction.
    PostSerialization,
}

impl Bip110PreflightPhase {
    /// Returns the measurement source that is valid for this lifecycle phase.
    pub const fn expected_measurement_source(self) -> Bip110MeasurementSource {
        match self {
            Self::PreConstruction => Bip110MeasurementSource::CallerClassified,
            Self::PostSerialization => Bip110MeasurementSource::SerializedTransaction,
        }
    }
}

/// Provenance of the classified byte measurements in a preflight request.
///
/// `CallerClassified` is intentionally weaker than serialized validation: it identifies
/// measurements supplied by a transaction-aware caller while construction is still in progress.
/// `SerializedTransaction` identifies measurements obtained from a finalized serialized
/// transaction. The validator rejects a request whose phase and source disagree.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Bip110MeasurementSource {
    /// Measurements classified by the caller before final serialization.
    #[default]
    #[serde(alias = "pre_construction")]
    CallerClassified,
    /// Measurements classified from the finalized serialized transaction.
    #[serde(alias = "post_serialization")]
    SerializedTransaction,
}

impl Bip110MeasurementSource {
    /// Returns the lifecycle phase that this provenance can validly support.
    pub const fn phase(self) -> Bip110PreflightPhase {
        match self {
            Self::CallerClassified => Bip110PreflightPhase::PreConstruction,
            Self::SerializedTransaction => Bip110PreflightPhase::PostSerialization,
        }
    }

    /// Returns whether this provenance is valid for the supplied lifecycle phase.
    pub const fn matches_phase(self, phase: Bip110PreflightPhase) -> bool {
        matches!(
            (self, phase),
            (
                Self::CallerClassified,
                Bip110PreflightPhase::PreConstruction
            ) | (
                Self::SerializedTransaction,
                Bip110PreflightPhase::PostSerialization
            )
        )
    }
}

/// Compatibility alias for callers that use provenance terminology.
pub type Bip110MeasurementProvenance = Bip110MeasurementSource;

/// Operation context attached to a BIP-110 preflight request.
///
/// Only [`Self::BitcoinTransaction`] is currently supported. Protocol-specific contexts are
/// represented explicitly so callers cannot accidentally treat Taproot, Miniscript, DLC, or
/// other context-sensitive measurements as a generic fully-classified transaction. Unknown wire
/// strings are retained by [`Self::Unknown`] and fail closed during validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Bip110OperationContext {
    /// Generic Bitcoin transaction context with all ordinary vectors classified by the caller and
    /// any Taproot control blocks supplied separately.
    BitcoinTransaction,
    /// Taproot context; its owning context contract is not defined here.
    Taproot,
    /// Tapscript context; its owning context contract is not defined here.
    Tapscript,
    /// Taproot script-path context; its owning context contract is not defined here.
    TaprootScriptPath,
    /// Taproot key-path context; its owning context contract is not defined here.
    TaprootKeyPath,
    /// Taproot leaf/Tapscript bytes; its owning context contract is not defined here.
    Tapleaf,
    /// Miniscript context; its owning context contract is not defined here.
    Miniscript,
    /// DLC context; its owning context contract is not defined here.
    Dlc,
    /// DLC funding context; its owning context contract is not defined here.
    DlcFunding,
    /// DLC refund context; its owning context contract is not defined here.
    DlcRefund,
    /// DLC contract execution context; its owning context contract is not defined here.
    DlcCet,
    /// Lightning context; its owning context contract is not defined here.
    Lightning,
    /// Lightning commitment context; its owning context contract is not defined here.
    LightningCommitment,
    /// Lightning closing context; its owning context contract is not defined here.
    LightningClosing,
    /// Lightning justice context; its owning context contract is not defined here.
    LightningJustice,
    /// Lightning HTLC context; its owning context contract is not defined here.
    LightningHtlc,
    /// RGB context; its owning context contract is not defined here.
    Rgb,
    /// RGB anchor context; its owning context contract is not defined here.
    RgbAnchor,
    /// Babylon context; its owning context contract is not defined here.
    Babylon,
    /// Babylon staking context; its owning context contract is not defined here.
    BabylonStaking,
    /// Babylon delegation context; its owning context contract is not defined here.
    BabylonDelegation,
    /// Babylon unbonding context; its owning context contract is not defined here.
    BabylonUnbonding,
    /// Babylon withdrawal context; its owning context contract is not defined here.
    BabylonWithdrawal,
    /// Babylon checkpoint context; its owning context contract is not defined here.
    BabylonCheckpoint,
    /// Fedimint context; its owning context contract is not defined here.
    Fedimint,
    /// Stacks or sBTC context; its owning context contract is not defined here.
    Stacks,
    /// Stacks/sBTC peg-in context; its owning context contract is not defined here.
    StacksSbtcPegIn,
    /// Stacks/sBTC peg-out context; its owning context contract is not defined here.
    StacksSbtcPegOut,
    /// Stacks/sBTC mint context; its owning context contract is not defined here.
    StacksSbtcMint,
    /// Stacks/sBTC burn context; its owning context contract is not defined here.
    StacksSbtcBurn,
    /// Liquid or Elements context; its owning context contract is not defined here.
    Liquid,
    /// A context name not known by this contract version.
    Unknown(String),
}

impl Bip110OperationContext {
    /// Returns the stable snake-case wire value for this context.
    pub fn as_str(&self) -> &str {
        match self {
            Self::BitcoinTransaction => "bitcoin_transaction",
            Self::Taproot => "taproot",
            Self::Tapscript => "tapscript",
            Self::TaprootScriptPath => "taproot_script_path",
            Self::TaprootKeyPath => "taproot_key_path",
            Self::Tapleaf => "tapleaf",
            Self::Miniscript => "miniscript",
            Self::Dlc => "dlc",
            Self::DlcFunding => "dlc_funding",
            Self::DlcRefund => "dlc_refund",
            Self::DlcCet => "dlc_cet",
            Self::Lightning => "lightning",
            Self::LightningCommitment => "lightning_commitment",
            Self::LightningClosing => "lightning_closing",
            Self::LightningJustice => "lightning_justice",
            Self::LightningHtlc => "lightning_htlc",
            Self::Rgb => "rgb",
            Self::RgbAnchor => "rgb_anchor",
            Self::Babylon => "babylon",
            Self::BabylonStaking => "babylon_staking",
            Self::BabylonDelegation => "babylon_delegation",
            Self::BabylonUnbonding => "babylon_unbonding",
            Self::BabylonWithdrawal => "babylon_withdrawal",
            Self::BabylonCheckpoint => "babylon_checkpoint",
            Self::Fedimint => "fedimint",
            Self::Stacks => "stacks",
            Self::StacksSbtcPegIn => "stacks_sbtc_peg_in",
            Self::StacksSbtcPegOut => "stacks_sbtc_peg_out",
            Self::StacksSbtcMint => "stacks_sbtc_mint",
            Self::StacksSbtcBurn => "stacks_sbtc_burn",
            Self::Liquid => "liquid",
            Self::Unknown(value) => value,
        }
    }

    /// Returns whether this context is supported by the current preflight contract.
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::BitcoinTransaction)
    }

    fn from_wire_value(value: String) -> Self {
        match value.as_str() {
            "bitcoin_transaction" => Self::BitcoinTransaction,
            "taproot" => Self::Taproot,
            "tapscript" => Self::Tapscript,
            "taproot_script_path" => Self::TaprootScriptPath,
            "taproot_key_path" => Self::TaprootKeyPath,
            "tapleaf" => Self::Tapleaf,
            "miniscript" => Self::Miniscript,
            "dlc" => Self::Dlc,
            "dlc_funding" => Self::DlcFunding,
            "dlc_refund" => Self::DlcRefund,
            "dlc_cet" => Self::DlcCet,
            "lightning" => Self::Lightning,
            "lightning_commitment" => Self::LightningCommitment,
            "lightning_closing" => Self::LightningClosing,
            "lightning_justice" => Self::LightningJustice,
            "lightning_htlc" => Self::LightningHtlc,
            "rgb" => Self::Rgb,
            "rgb_anchor" => Self::RgbAnchor,
            "babylon" => Self::Babylon,
            "babylon_staking" => Self::BabylonStaking,
            "babylon_delegation" => Self::BabylonDelegation,
            "babylon_unbonding" => Self::BabylonUnbonding,
            "babylon_withdrawal" => Self::BabylonWithdrawal,
            "babylon_checkpoint" => Self::BabylonCheckpoint,
            "fedimint" => Self::Fedimint,
            "stacks" => Self::Stacks,
            "stacks_sbtc_peg_in" => Self::StacksSbtcPegIn,
            "stacks_sbtc_peg_out" => Self::StacksSbtcPegOut,
            "stacks_sbtc_mint" => Self::StacksSbtcMint,
            "stacks_sbtc_burn" => Self::StacksSbtcBurn,
            "liquid" => Self::Liquid,
            _ => Self::Unknown(value),
        }
    }
}

impl Serialize for Bip110OperationContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

struct OperationContextVisitor;

impl<'de> Visitor<'de> for OperationContextVisitor {
    type Value = Bip110OperationContext;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a snake_case BIP-110 operation context string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bip110OperationContext::from_wire_value(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Bip110OperationContext::from_wire_value(value))
    }
}

impl<'de> Deserialize<'de> for Bip110OperationContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(OperationContextVisitor)
    }
}

impl fmt::Display for Bip110OperationContext {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A fixed-width measurement category in a preflight request or violation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Bip110MeasurementField {
    /// Payload bytes carried by one applicable pushdata operation.
    Pushdata,
    /// Complete serialized OP_RETURN output ScriptPubKey bytes.
    OpReturnScriptPubkey,
    /// Complete serialized non-OP_RETURN output ScriptPubKey bytes.
    NonOpReturnScriptPubkey,
    /// Bytes in one applicable script-argument witness element.
    WitnessElement,
    /// Complete serialized bytes of one explicitly classified Taproot control block.
    TaprootControlBlock,
}

impl Bip110MeasurementField {
    /// Returns the stable machine-readable field value.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pushdata => "pushdata",
            Self::OpReturnScriptPubkey => "op_return_script_pubkey",
            Self::NonOpReturnScriptPubkey => "non_op_return_script_pubkey",
            Self::WitnessElement => "witness_element",
            Self::TaprootControlBlock => "taproot_control_block",
        }
    }
}

/// Fixed-width byte measurements supplied by a transaction-aware adapter.
///
/// Each vector preserves occurrence order. The fields use `u64` on the wire rather than `usize`:
/// pushdata values are payload bytes only; ScriptPubKey values are complete serialized scripts;
/// witness values are individual applicable script-argument elements, not total witness
/// serialization lengths; and control-block values are complete serialized control-block witness
/// items kept separate from ordinary witness arguments.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightMeasurements {
    /// Provenance of these classified measurements.
    #[serde(default)]
    pub source: Bip110MeasurementSource,
    /// Payload byte sizes for applicable pushdata occurrences.
    pub pushdata_sizes_bytes: Vec<u64>,
    /// Complete serialized OP_RETURN ScriptPubKey byte sizes.
    pub op_return_script_pubkey_sizes_bytes: Vec<u64>,
    /// Complete serialized non-OP_RETURN ScriptPubKey byte sizes.
    pub non_op_return_script_pubkey_sizes_bytes: Vec<u64>,
    /// Byte sizes for applicable script-argument witness elements.
    pub witness_element_sizes_bytes: Vec<u64>,
    /// Complete serialized Taproot control-block sizes, kept separate from witness arguments.
    #[serde(default, alias = "control_block_sizes_bytes")]
    pub taproot_control_block_sizes_bytes: Vec<u64>,
}

impl Bip110PreflightMeasurements {
    /// Creates fixed-width measurements in canonical category order.
    pub fn new(
        pushdata_sizes_bytes: Vec<u64>,
        op_return_script_pubkey_sizes_bytes: Vec<u64>,
        non_op_return_script_pubkey_sizes_bytes: Vec<u64>,
        witness_element_sizes_bytes: Vec<u64>,
    ) -> Self {
        Self {
            source: Bip110MeasurementSource::default(),
            pushdata_sizes_bytes,
            op_return_script_pubkey_sizes_bytes,
            non_op_return_script_pubkey_sizes_bytes,
            witness_element_sizes_bytes,
            taproot_control_block_sizes_bytes: Vec::new(),
        }
    }

    /// Creates fixed-width measurements with an explicit measurement source and control blocks.
    pub fn new_with_source(
        source: Bip110MeasurementSource,
        pushdata_sizes_bytes: Vec<u64>,
        op_return_script_pubkey_sizes_bytes: Vec<u64>,
        non_op_return_script_pubkey_sizes_bytes: Vec<u64>,
        witness_element_sizes_bytes: Vec<u64>,
        taproot_control_block_sizes_bytes: Vec<u64>,
    ) -> Self {
        Self {
            source,
            pushdata_sizes_bytes,
            op_return_script_pubkey_sizes_bytes,
            non_op_return_script_pubkey_sizes_bytes,
            witness_element_sizes_bytes,
            taproot_control_block_sizes_bytes,
        }
    }

    /// Creates fixed-width measurements with a separate Taproot control-block vector.
    pub fn new_with_control_block_sizes(
        pushdata_sizes_bytes: Vec<u64>,
        op_return_script_pubkey_sizes_bytes: Vec<u64>,
        non_op_return_script_pubkey_sizes_bytes: Vec<u64>,
        witness_element_sizes_bytes: Vec<u64>,
        taproot_control_block_sizes_bytes: Vec<u64>,
    ) -> Self {
        Self::new_with_source(
            Bip110MeasurementSource::default(),
            pushdata_sizes_bytes,
            op_return_script_pubkey_sizes_bytes,
            non_op_return_script_pubkey_sizes_bytes,
            witness_element_sizes_bytes,
            taproot_control_block_sizes_bytes,
        )
    }

    /// Returns a copy with the supplied measurement provenance.
    pub fn with_source(mut self, source: Bip110MeasurementSource) -> Self {
        self.source = source;
        self
    }

    /// Returns a copy with the supplied Taproot control-block measurements.
    pub fn with_control_block_sizes(mut self, sizes: Vec<u64>) -> Self {
        self.taproot_control_block_sizes_bytes = sizes;
        self
    }

    /// Converts these wire measurements to the existing `usize`-based transaction shape.
    ///
    /// The conversion is checked and returns the first deterministic overflow error. The
    /// preflight validator uses the same checked conversion while accumulating every overflow
    /// finding before attempting size validation.
    pub fn try_into_transaction_shape(
        &self,
    ) -> Result<Bip110TransactionShape, Bip110PreflightError> {
        let (shape, errors) = self.to_transaction_shape_with_errors();
        match errors.into_iter().next() {
            Some(error) => Err(error),
            None => Ok(shape),
        }
    }

    fn to_transaction_shape_with_errors(
        &self,
    ) -> (Bip110TransactionShape, Vec<Bip110PreflightError>) {
        let (pushdata_sizes_bytes, mut errors) =
            checked_sizes(Bip110MeasurementField::Pushdata, &self.pushdata_sizes_bytes);
        let (op_return_script_pubkey_sizes_bytes, op_return_errors) = checked_sizes(
            Bip110MeasurementField::OpReturnScriptPubkey,
            &self.op_return_script_pubkey_sizes_bytes,
        );
        errors.extend(op_return_errors);
        let (non_op_return_script_pubkey_sizes_bytes, non_op_return_errors) = checked_sizes(
            Bip110MeasurementField::NonOpReturnScriptPubkey,
            &self.non_op_return_script_pubkey_sizes_bytes,
        );
        errors.extend(non_op_return_errors);
        let (witness_element_sizes_bytes, witness_errors) = checked_sizes(
            Bip110MeasurementField::WitnessElement,
            &self.witness_element_sizes_bytes,
        );
        errors.extend(witness_errors);

        (
            Bip110TransactionShape::new(
                pushdata_sizes_bytes,
                op_return_script_pubkey_sizes_bytes,
                non_op_return_script_pubkey_sizes_bytes,
                witness_element_sizes_bytes,
            ),
            errors,
        )
    }
}

/// A versioned request for BIP-110 preflight validation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Bip110PreflightRequest {
    /// Serialized contract version requested by the caller.
    pub api_version: u16,
    /// Lifecycle phase represented by the measurements.
    pub phase: Bip110PreflightPhase,
    /// Transaction/script context supplied by the caller.
    pub context: Bip110OperationContext,
    /// Classified fixed-width measurements for the request.
    pub measurements: Bip110PreflightMeasurements,
    /// Whether the caller supplied classified measurements.
    ///
    /// This is separate from empty vectors: an explicitly present generic transaction with zero
    /// constrained occurrences is valid, while missing measurement data always fails closed.
    #[serde(skip_serializing_if = "is_true")]
    pub measurements_present: bool,
}

fn is_true(value: &bool) -> bool {
    *value
}

#[derive(Debug, Deserialize)]
struct Bip110PreflightMeasurementsWire {
    #[serde(default)]
    source: Option<Bip110MeasurementSource>,
    pushdata_sizes_bytes: Vec<u64>,
    op_return_script_pubkey_sizes_bytes: Vec<u64>,
    non_op_return_script_pubkey_sizes_bytes: Vec<u64>,
    witness_element_sizes_bytes: Vec<u64>,
    #[serde(default, alias = "control_block_sizes_bytes")]
    taproot_control_block_sizes_bytes: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct Bip110PreflightRequestWire {
    api_version: u16,
    phase: Bip110PreflightPhase,
    context: Bip110OperationContext,
    #[serde(default)]
    measurements: Option<Bip110PreflightMeasurementsWire>,
    #[serde(default)]
    measurements_present: Option<bool>,
    #[serde(default)]
    measurement_source: Option<Bip110MeasurementSource>,
}

impl<'de> Deserialize<'de> for Bip110PreflightRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = Bip110PreflightRequestWire::deserialize(deserializer)?;
        let measurements_present = wire
            .measurements_present
            .unwrap_or(wire.measurements.is_some())
            && wire.measurements.is_some();
        let measurements = match wire.measurements {
            Some(measurements) => Bip110PreflightMeasurements {
                source: measurements
                    .source
                    .or(wire.measurement_source)
                    .unwrap_or_else(|| wire.phase.expected_measurement_source()),
                pushdata_sizes_bytes: measurements.pushdata_sizes_bytes,
                op_return_script_pubkey_sizes_bytes: measurements
                    .op_return_script_pubkey_sizes_bytes,
                non_op_return_script_pubkey_sizes_bytes: measurements
                    .non_op_return_script_pubkey_sizes_bytes,
                witness_element_sizes_bytes: measurements.witness_element_sizes_bytes,
                taproot_control_block_sizes_bytes: measurements.taproot_control_block_sizes_bytes,
            },
            None => {
                Bip110PreflightMeasurements::new(Vec::new(), Vec::new(), Vec::new(), Vec::new())
                    .with_source(wire.phase.expected_measurement_source())
            }
        };

        Ok(Self {
            api_version: wire.api_version,
            phase: wire.phase,
            context: wire.context,
            measurements,
            measurements_present,
        })
    }
}

impl Bip110PreflightRequest {
    /// Creates a request for the current preflight API version.
    pub fn new(
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
        measurements: Bip110PreflightMeasurements,
    ) -> Self {
        Self::with_api_version_and_source(
            BIP110_PREFLIGHT_API_VERSION,
            phase,
            context,
            phase.expected_measurement_source(),
            measurements,
        )
    }

    /// Creates a request with explicit measurement provenance.
    ///
    /// This constructor is useful for testing and for adapters that need the validator to reject
    /// an accidentally mislabeled pre-construction or post-serialization measurement set.
    pub fn new_with_source(
        phase: Bip110PreflightPhase,
        source: Bip110MeasurementSource,
        context: Bip110OperationContext,
        measurements: Bip110PreflightMeasurements,
    ) -> Self {
        Self::with_api_version_and_source(
            BIP110_PREFLIGHT_API_VERSION,
            phase,
            context,
            source,
            measurements,
        )
    }

    /// Creates a request with an explicitly selected API version for negotiation or rejection
    /// tests.
    pub fn with_api_version(
        api_version: u16,
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
        measurements: Bip110PreflightMeasurements,
    ) -> Self {
        Self::with_api_version_and_source(
            api_version,
            phase,
            context,
            phase.expected_measurement_source(),
            measurements,
        )
    }

    /// Creates a request with an explicit API version and measurement provenance.
    pub fn with_api_version_and_source(
        api_version: u16,
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
        source: Bip110MeasurementSource,
        mut measurements: Bip110PreflightMeasurements,
    ) -> Self {
        measurements.source = source;
        Self {
            api_version,
            phase,
            context,
            measurements,
            measurements_present: true,
        }
    }

    /// Creates an explicit missing-measurement request that cannot produce compliant success.
    pub fn without_measurements(
        phase: Bip110PreflightPhase,
        context: Bip110OperationContext,
    ) -> Self {
        Self {
            api_version: BIP110_PREFLIGHT_API_VERSION,
            phase,
            context,
            measurements: Bip110PreflightMeasurements::new(
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .with_source(phase.expected_measurement_source()),
            measurements_present: false,
        }
    }

    /// Returns whether classified measurement data was explicitly supplied.
    pub const fn has_measurements(&self) -> bool {
        self.measurements_present
    }

    /// Returns a copy of this request using another API version.
    pub fn with_version(mut self, api_version: u16) -> Self {
        self.api_version = api_version;
        self
    }

    /// Validates this request with a fresh enabled canonical validator.
    pub fn validate(&self) -> Bip110PreflightResult {
        Bip110PreflightValidator::new().validate(self)
    }
}

/// Stable machine-readable error codes for structural or request-level failures.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightErrorCode {
    /// The request API version is not supported.
    UnsupportedApiVersion,
    /// The context string is not known by this contract version.
    UnknownContext,
    /// The context is known but has no owning preflight contract yet.
    UnsupportedContext,
    /// The request phase and measurement provenance disagree.
    PhaseMismatch,
    /// The request did not contain classified measurements.
    MissingMeasurementData,
    /// The request structure was malformed before size validation.
    MalformedRequest,
    /// A fixed-width wire measurement could not be represented as `usize`.
    IntegerOverflow,
    /// A vector index could not be represented as a fixed-width wire index.
    IndexOverflow,
    /// A disabled compliance instance was supplied to a validator constructor.
    ComplianceDisabled,
}

/// Structural or request-level BIP-110 preflight failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Bip110PreflightError {
    /// The request API version is not supported by this implementation.
    UnsupportedApiVersion { requested: u16, supported: u16 },
    /// The request carried an unknown context string.
    UnknownContext { context: String },
    /// The request carried a known but not-yet-supported context.
    UnsupportedContext { context: String },
    /// The request phase does not match the provenance of its measurements.
    PhaseMismatch {
        expected_phase: Bip110PreflightPhase,
        received_source: Bip110MeasurementSource,
    },
    /// The request omitted its classified measurement data.
    MissingMeasurementData,
    /// A request was structurally malformed before size validation could begin.
    MalformedRequest { reason: String },
    /// A `u64` wire measurement did not fit into the target platform's `usize`.
    IntegerOverflow {
        field: Bip110MeasurementField,
        index: u64,
        actual_bytes: u64,
    },
    /// A vector occurrence index did not fit into the fixed-width wire representation.
    IndexOverflow { field: Bip110MeasurementField },
    /// A disabled `Bip110Compliance` instance was rejected rather than treated as compliant.
    ComplianceDisabled,
}

impl Bip110PreflightError {
    /// Returns the stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedApiVersion { .. } => "unsupported_api_version",
            Self::UnknownContext { .. } => "unknown_context",
            Self::UnsupportedContext { .. } => "unsupported_context",
            Self::PhaseMismatch { .. } => "phase_mismatch",
            Self::MissingMeasurementData => "missing_measurement_data",
            Self::MalformedRequest { .. } => "malformed_request",
            Self::IntegerOverflow { .. } => "integer_overflow",
            Self::IndexOverflow { .. } => "index_overflow",
            Self::ComplianceDisabled => "compliance_disabled",
        }
    }

    /// Returns the typed error code value.
    pub const fn error_code(&self) -> Bip110PreflightErrorCode {
        match self {
            Self::UnsupportedApiVersion { .. } => Bip110PreflightErrorCode::UnsupportedApiVersion,
            Self::UnknownContext { .. } => Bip110PreflightErrorCode::UnknownContext,
            Self::UnsupportedContext { .. } => Bip110PreflightErrorCode::UnsupportedContext,
            Self::PhaseMismatch { .. } => Bip110PreflightErrorCode::PhaseMismatch,
            Self::MissingMeasurementData => Bip110PreflightErrorCode::MissingMeasurementData,
            Self::MalformedRequest { .. } => Bip110PreflightErrorCode::MalformedRequest,
            Self::IntegerOverflow { .. } => Bip110PreflightErrorCode::IntegerOverflow,
            Self::IndexOverflow { .. } => Bip110PreflightErrorCode::IndexOverflow,
            Self::ComplianceDisabled => Bip110PreflightErrorCode::ComplianceDisabled,
        }
    }
}

impl Bip110PreflightErrorCode {
    /// Returns the stable machine-readable code string for this error category.
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedApiVersion => "unsupported_api_version",
            Self::UnknownContext => "unknown_context",
            Self::UnsupportedContext => "unsupported_context",
            Self::PhaseMismatch => "phase_mismatch",
            Self::MissingMeasurementData => "missing_measurement_data",
            Self::MalformedRequest => "malformed_request",
            Self::IntegerOverflow => "integer_overflow",
            Self::IndexOverflow => "index_overflow",
            Self::ComplianceDisabled => "compliance_disabled",
        }
    }
}

impl fmt::Display for Bip110PreflightError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedApiVersion {
                requested,
                supported,
            } => write!(
                formatter,
                "unsupported BIP-110 preflight API version {requested}; supported version is {supported}"
            ),
            Self::UnknownContext { context } => {
                write!(formatter, "unknown BIP-110 preflight context {context:?}")
            }
            Self::UnsupportedContext { context } => {
                write!(formatter, "unsupported BIP-110 preflight context {context:?}")
            }
            Self::PhaseMismatch {
                expected_phase,
                received_source,
            } => write!(
                formatter,
                "BIP-110 preflight phase {expected_phase:?} does not match measurement source {received_source:?}"
            ),
            Self::MissingMeasurementData => {
                formatter.write_str("missing BIP-110 preflight measurement data")
            }
            Self::MalformedRequest { reason } => {
                write!(formatter, "malformed BIP-110 preflight request: {reason}")
            }
            Self::IntegerOverflow {
                field,
                index,
                actual_bytes,
            } => write!(
                formatter,
                "{field:?} occurrence {index} has {actual_bytes} bytes, which does not fit the target usize"
            ),
            Self::IndexOverflow { field } => {
                write!(formatter, "{field:?} occurrence index does not fit the wire format")
            }
            Self::ComplianceDisabled => {
                formatter.write_str("disabled BIP-110 compliance cannot be used for preflight")
            }
        }
    }
}

impl std::error::Error for Bip110PreflightError {}

/// Stable machine-readable violation codes for size findings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Bip110PreflightViolationCode {
    /// An applicable pushdata payload exceeded its limit.
    PushdataExceedsLimit,
    /// An OP_RETURN ScriptPubKey exceeded its limit.
    OpReturnExceedsLimit,
    /// A non-OP_RETURN ScriptPubKey exceeded its limit.
    ScriptPubkeyExceedsLimit,
    /// An applicable witness element exceeded its limit.
    WitnessElementExceedsLimit,
    /// A separately classified Taproot control block exceeded its limit.
    TaprootControlBlockExceedsLimit,
}

impl Bip110PreflightViolationCode {
    /// Returns the stable machine-readable violation code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::PushdataExceedsLimit => "pushdata_exceeds_limit",
            Self::OpReturnExceedsLimit => "op_return_exceeds_limit",
            Self::ScriptPubkeyExceedsLimit => "script_pubkey_exceeds_limit",
            Self::WitnessElementExceedsLimit => "witness_element_exceeds_limit",
            Self::TaprootControlBlockExceedsLimit => "taproot_control_block_exceeds_limit",
        }
    }
}

/// Indexed, fixed-width BIP-110 size violation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightViolation {
    /// Stable machine-readable violation code.
    pub code: Bip110PreflightViolationCode,
    /// Measurement category that produced the violation.
    pub field: Bip110MeasurementField,
    /// Zero-based occurrence index within the category vector.
    pub index: u64,
    /// Actual measured bytes.
    pub actual_bytes: u64,
    /// Canonical or configured maximum bytes.
    pub max_bytes: u64,
}

impl Bip110PreflightViolation {
    /// Returns the stable machine-readable violation code string.
    pub const fn code(&self) -> &'static str {
        self.code.code()
    }

    /// Compatibility alias for callers that prefer an explicitly named code accessor.
    pub const fn code_value(&self) -> &'static str {
        self.code()
    }
}

/// One ordered structural error or size violation returned by preflight.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum Bip110PreflightFinding {
    /// Structural or request-level failure.
    Error(Bip110PreflightError),
    /// Size-policy violation.
    Violation(Bip110PreflightViolation),
}

impl Bip110PreflightFinding {
    /// Returns the stable machine-readable code for this finding.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Error(error) => error.code(),
            Self::Violation(violation) => violation.code(),
        }
    }
}

/// Deterministic result from BIP-110 preflight validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110PreflightResult {
    /// API version used to evaluate the request.
    pub api_version: u16,
    /// Lifecycle phase copied from the request.
    pub phase: Bip110PreflightPhase,
    /// Context copied from the request, including unknown strings.
    pub context: Bip110OperationContext,
    /// Provenance copied from the classified measurements.
    #[serde(default)]
    pub measurement_source: Bip110MeasurementSource,
    /// `true` only when there are no structural errors or size violations.
    pub is_compliant: bool,
    /// Findings in deterministic structural-then-size order. Size findings are ordered as
    /// pushdata, OP_RETURN, non-OP_RETURN, witness, then Taproot control block.
    pub findings: Vec<Bip110PreflightFinding>,
}

impl Bip110PreflightResult {
    /// Returns all structural errors in their result order.
    pub fn errors(&self) -> impl Iterator<Item = &Bip110PreflightError> {
        self.findings.iter().filter_map(|finding| match finding {
            Bip110PreflightFinding::Error(error) => Some(error),
            Bip110PreflightFinding::Violation(_) => None,
        })
    }

    /// Returns all size violations in their result order.
    pub fn violations(&self) -> impl Iterator<Item = &Bip110PreflightViolation> {
        self.findings.iter().filter_map(|finding| match finding {
            Bip110PreflightFinding::Error(_) => None,
            Bip110PreflightFinding::Violation(violation) => Some(violation),
        })
    }

    /// Returns whether the result contains at least one structural error.
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|finding| matches!(finding, Bip110PreflightFinding::Error(_)))
    }
}

/// Enabled BIP-110 validator for preflight requests.
///
/// `new()` always creates an enabled canonical [`Bip110Compliance`]. Passing a disabled
/// compliance instance to [`Self::with_compliance`] returns an error, preventing the legacy
/// disabled `Bip110Compliance::default()` behavior from becoming a fail-open preflight path.
#[derive(Debug, Clone)]
pub struct Bip110PreflightValidator {
    compliance: Bip110Compliance,
}

impl Default for Bip110PreflightValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Bip110PreflightValidator {
    /// Creates an enabled validator using canonical BIP-110 limits.
    pub fn new() -> Self {
        Self {
            compliance: Bip110Compliance::new(),
        }
    }

    /// Creates a validator from an existing enabled compliance configuration.
    ///
    /// Disabled compliance is rejected as a structural configuration error rather than allowed
    /// to produce a compliant result.
    pub fn with_compliance(compliance: Bip110Compliance) -> Result<Self, Bip110PreflightError> {
        if !compliance.is_enabled() {
            return Err(Bip110PreflightError::ComplianceDisabled);
        }

        Ok(Self { compliance })
    }

    /// Returns the enabled compliance configuration used by this validator.
    pub fn compliance(&self) -> &Bip110Compliance {
        &self.compliance
    }

    /// Validates one request and accumulates deterministic findings.
    pub fn validate(&self, request: &Bip110PreflightRequest) -> Bip110PreflightResult {
        let mut findings = Vec::new();

        if request.api_version != BIP110_PREFLIGHT_API_VERSION {
            findings.push(Bip110PreflightFinding::Error(
                Bip110PreflightError::UnsupportedApiVersion {
                    requested: request.api_version,
                    supported: BIP110_PREFLIGHT_API_VERSION,
                },
            ));
        }

        if request.has_measurements() && !request.measurements.source.matches_phase(request.phase) {
            findings.push(Bip110PreflightFinding::Error(
                Bip110PreflightError::PhaseMismatch {
                    expected_phase: request.phase,
                    received_source: request.measurements.source,
                },
            ));
        }

        match &request.context {
            Bip110OperationContext::BitcoinTransaction => {}
            Bip110OperationContext::Unknown(context) => {
                findings.push(Bip110PreflightFinding::Error(
                    Bip110PreflightError::UnknownContext {
                        context: context.clone(),
                    },
                ));
            }
            context => {
                findings.push(Bip110PreflightFinding::Error(
                    Bip110PreflightError::UnsupportedContext {
                        context: context.as_str().to_owned(),
                    },
                ));
            }
        }

        if !request.has_measurements() {
            findings.push(Bip110PreflightFinding::Error(
                Bip110PreflightError::MissingMeasurementData,
            ));
        }

        if !findings.is_empty() {
            return result_for(request, findings);
        }

        let (shape, conversion_errors) = request.measurements.to_transaction_shape_with_errors();
        if !conversion_errors.is_empty() {
            findings.extend(
                conversion_errors
                    .into_iter()
                    .map(Bip110PreflightFinding::Error),
            );
            return result_for(request, findings);
        }

        let violations = match self.collect_violations(
            &shape,
            &request.measurements.taproot_control_block_sizes_bytes,
        ) {
            Ok(violations) => violations,
            Err(error) => {
                findings.push(Bip110PreflightFinding::Error(error));
                return result_for(request, findings);
            }
        };
        findings.extend(
            violations
                .into_iter()
                .map(Bip110PreflightFinding::Violation),
        );

        result_for(request, findings)
    }

    fn collect_violations(
        &self,
        shape: &Bip110TransactionShape,
        taproot_control_block_sizes: &[u64],
    ) -> Result<Vec<Bip110PreflightViolation>, Bip110PreflightError> {
        let mut violations = Vec::new();
        violations.extend(collect_category_violations(
            Bip110MeasurementField::Pushdata,
            &shape.pushdata_sizes_bytes,
            |size| self.compliance.validate_pushdata(size),
        )?);
        violations.extend(collect_category_violations(
            Bip110MeasurementField::OpReturnScriptPubkey,
            &shape.op_return_script_pubkey_sizes_bytes,
            |size| self.compliance.validate_op_return(size),
        )?);
        violations.extend(collect_category_violations(
            Bip110MeasurementField::NonOpReturnScriptPubkey,
            &shape.non_op_return_script_pubkey_sizes_bytes,
            |size| self.compliance.validate_script_pubkey(size),
        )?);
        violations.extend(collect_category_violations(
            Bip110MeasurementField::WitnessElement,
            &shape.witness_element_sizes_bytes,
            |size| self.compliance.validate_witness_element(size),
        )?);
        violations.extend(collect_control_block_violations(
            taproot_control_block_sizes,
        )?);
        Ok(violations)
    }
}

/// Validates a request with a fresh enabled canonical validator.
pub fn validate_bip110_preflight(request: &Bip110PreflightRequest) -> Bip110PreflightResult {
    Bip110PreflightValidator::new().validate(request)
}

fn result_for(
    request: &Bip110PreflightRequest,
    findings: Vec<Bip110PreflightFinding>,
) -> Bip110PreflightResult {
    Bip110PreflightResult {
        api_version: request.api_version,
        phase: request.phase,
        context: request.context.clone(),
        measurement_source: request.measurements.source,
        is_compliant: findings.is_empty(),
        findings,
    }
}

fn checked_sizes(
    field: Bip110MeasurementField,
    values: &[u64],
) -> (Vec<usize>, Vec<Bip110PreflightError>) {
    let mut converted = Vec::with_capacity(values.len());
    let mut errors = Vec::new();

    for (index, &actual_bytes) in values.iter().enumerate() {
        let index = match u64::try_from(index) {
            Ok(index) => index,
            Err(_) => {
                errors.push(Bip110PreflightError::IndexOverflow { field });
                continue;
            }
        };

        match usize::try_from(actual_bytes) {
            Ok(value) => converted.push(value),
            Err(_) => errors.push(Bip110PreflightError::IntegerOverflow {
                field,
                index,
                actual_bytes,
            }),
        }
    }

    (converted, errors)
}

fn collect_category_violations<F>(
    field: Bip110MeasurementField,
    sizes: &[usize],
    validate: F,
) -> Result<Vec<Bip110PreflightViolation>, Bip110PreflightError>
where
    F: Fn(usize) -> Bip110ValidationResult,
{
    let mut violations = Vec::new();

    for (index, &size) in sizes.iter().enumerate() {
        let index =
            u64::try_from(index).map_err(|_| Bip110PreflightError::IndexOverflow { field })?;

        for violation in validate(size).violations {
            let (code, actual, max) = violation_parts(violation);
            let actual_bytes =
                u64::try_from(actual).map_err(|_| Bip110PreflightError::IndexOverflow { field })?;
            let max_bytes =
                u64::try_from(max).map_err(|_| Bip110PreflightError::IndexOverflow { field })?;

            violations.push(Bip110PreflightViolation {
                code,
                field,
                index,
                actual_bytes,
                max_bytes,
            });
        }
    }

    Ok(violations)
}

fn collect_control_block_violations(
    sizes: &[u64],
) -> Result<Vec<Bip110PreflightViolation>, Bip110PreflightError> {
    let mut violations = Vec::new();

    for (index, &actual_bytes) in sizes.iter().enumerate() {
        let index = u64::try_from(index).map_err(|_| Bip110PreflightError::IndexOverflow {
            field: Bip110MeasurementField::TaprootControlBlock,
        })?;

        if actual_bytes > MAX_TAPROOT_CONTROL_BLOCK_BYTES {
            violations.push(Bip110PreflightViolation {
                code: Bip110PreflightViolationCode::TaprootControlBlockExceedsLimit,
                field: Bip110MeasurementField::TaprootControlBlock,
                index,
                actual_bytes,
                max_bytes: MAX_TAPROOT_CONTROL_BLOCK_BYTES,
            });
        }
    }

    Ok(violations)
}

fn violation_parts(violation: Bip110Violation) -> (Bip110PreflightViolationCode, usize, usize) {
    match violation {
        Bip110Violation::PushdataExceedsLimit { size, max } => (
            Bip110PreflightViolationCode::PushdataExceedsLimit,
            size,
            max,
        ),
        Bip110Violation::OpReturnExceedsLimit { size, max } => (
            Bip110PreflightViolationCode::OpReturnExceedsLimit,
            size,
            max,
        ),
        Bip110Violation::ScriptPubKeyExceedsLimit { size, max } => (
            Bip110PreflightViolationCode::ScriptPubkeyExceedsLimit,
            size,
            max,
        ),
        Bip110Violation::WitnessElementExceedsLimit { size, max } => (
            Bip110PreflightViolationCode::WitnessElementExceedsLimit,
            size,
            max,
        ),
    }
}
