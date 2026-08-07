//! BIP-110 transaction builder guard.
//!
//! Provides [`Bip110SizeGuard`], a builder-pattern validator that tracks
//! constrained byte surfaces during transaction construction and enforces
//! BIP-110 (Reduced Data Softfork) limits before finalization.
//!
//! Downstream transaction builders call `track_*` methods as they add
//! elements, then call [`validate`](Bip110SizeGuard::validate) before
//! signing. If validation fails, [`suggestions`](Bip110SizeGuard::suggestions)
//! returns human-readable optimization guidance.

use crate::control_model::bip110::{
    Bip110TransactionShape, MAX_OP_RETURN_BYTES, MAX_PUSHDATA_BYTES, MAX_SCRIPT_PUBKEY_BYTES,
    MAX_WITNESS_ELEMENT_BYTES,
};
use crate::control_model::trust::{Bip110Compliance, Bip110ValidationResult};

/// Tracks BIP-110-constrained byte surfaces during transaction construction.
///
/// # Usage
///
/// ```rust
/// use lib_conxian_core::bitcoin::bip110_builder::Bip110SizeGuard;
///
/// let mut guard = Bip110SizeGuard::new();
/// guard.track_pushdata(200);   // within 256-byte limit
/// guard.track_op_return(60);   // within 83-byte limit
/// guard.track_script_pubkey(34); // within 34-byte limit
///
/// let result = guard.validate();
/// assert!(result.is_compliant);
/// ```
#[derive(Debug, Clone)]
pub struct Bip110SizeGuard {
    compliance: Bip110Compliance,
    pushdata_sizes: Vec<usize>,
    op_return_sizes: Vec<usize>,
    script_pubkey_sizes: Vec<usize>,
    witness_sizes: Vec<usize>,
    /// Per-element pre-check results for optimization suggestions.
    warnings: Vec<SizeWarning>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeWarning {
    #[allow(clippy::enum_variant_names)]
    PushdataApproaching {
        size: usize,
        remaining: usize,
    },
    OpReturnApproaching {
        size: usize,
        remaining: usize,
    },
    ScriptPubKeyApproaching {
        size: usize,
        remaining: usize,
    },
    WitnessApproaching {
        size: usize,
        remaining: usize,
    },
}

impl Bip110SizeGuard {
    /// Create a new guard with canonical BIP-110 limits.
    pub fn new() -> Self {
        Self {
            compliance: Bip110Compliance::new(),
            pushdata_sizes: Vec::new(),
            op_return_sizes: Vec::new(),
            script_pubkey_sizes: Vec::new(),
            witness_sizes: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a guard with custom BIP-110 limits.
    pub fn with_limits(limits: crate::control_model::Bip110Limits) -> Self {
        Self {
            compliance: Bip110Compliance::with_limits(limits),
            pushdata_sizes: Vec::new(),
            op_return_sizes: Vec::new(),
            script_pubkey_sizes: Vec::new(),
            witness_sizes: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a disabled guard (no BIP-110 enforcement).
    pub fn disabled() -> Self {
        Self {
            compliance: Bip110Compliance::disabled(),
            pushdata_sizes: Vec::new(),
            op_return_sizes: Vec::new(),
            script_pubkey_sizes: Vec::new(),
            witness_sizes: Vec::new(),
            warnings: Vec::new(),
        }
    }

    // ── Tracking methods ──

    /// Track a pushdata element. Warns when within 20% of the 256-byte limit.
    pub fn track_pushdata(&mut self, size_bytes: usize) -> &mut Self {
        self.pushdata_sizes.push(size_bytes);
        if size_bytes > MAX_PUSHDATA_BYTES * 4 / 5 {
            self.warnings.push(SizeWarning::PushdataApproaching {
                size: size_bytes,
                remaining: MAX_PUSHDATA_BYTES.saturating_sub(size_bytes),
            });
        }
        self
    }

    /// Track an OP_RETURN script pubkey. Warns when within 20% of the 83-byte limit.
    pub fn track_op_return(&mut self, size_bytes: usize) -> &mut Self {
        self.op_return_sizes.push(size_bytes);
        if size_bytes > MAX_OP_RETURN_BYTES * 4 / 5 {
            self.warnings.push(SizeWarning::OpReturnApproaching {
                size: size_bytes,
                remaining: MAX_OP_RETURN_BYTES.saturating_sub(size_bytes),
            });
        }
        self
    }

    /// Track a non-OP_RETURN script pubkey. Warns when within 20% of the 34-byte limit.
    pub fn track_script_pubkey(&mut self, size_bytes: usize) -> &mut Self {
        self.script_pubkey_sizes.push(size_bytes);
        if size_bytes > MAX_SCRIPT_PUBKEY_BYTES * 4 / 5 {
            self.warnings.push(SizeWarning::ScriptPubKeyApproaching {
                size: size_bytes,
                remaining: MAX_SCRIPT_PUBKEY_BYTES.saturating_sub(size_bytes),
            });
        }
        self
    }

    /// Track a witness element. Warns when within 20% of the 256-byte limit.
    pub fn track_witness(&mut self, size_bytes: usize) -> &mut Self {
        self.witness_sizes.push(size_bytes);
        if size_bytes > MAX_WITNESS_ELEMENT_BYTES * 4 / 5 {
            self.warnings.push(SizeWarning::WitnessApproaching {
                size: size_bytes,
                remaining: MAX_WITNESS_ELEMENT_BYTES.saturating_sub(size_bytes),
            });
        }
        self
    }

    // ── Validation ──

    /// Validate all tracked elements against BIP-110 limits.
    ///
    /// Returns a [`Bip110ValidationResult`] with all violations. If the
    /// compliance is disabled, this always returns compliant.
    pub fn validate(&self) -> Bip110ValidationResult {
        self.compliance.validate_shape(&self.to_shape())
    }

    /// Check whether any tracked element approaches its limit.
    ///
    /// Returns `true` if pre-construction warnings were generated. These
    /// are advisory — the transaction may still be compliant even with
    /// warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Human-readable optimization suggestions for data-heavy flows.
    ///
    /// Returns an empty vector if no warnings were generated. Each string
    /// describes which element is approaching its limit and suggests a
    /// concrete mitigation.
    pub fn suggestions(&self) -> Vec<String> {
        self.warnings
            .iter()
            .map(|w| match w {
                SizeWarning::PushdataApproaching { size, remaining } => {
                    format!(
                        "pushdata element is {} bytes (only {} bytes remain before the {} byte BIP-110 limit); consider witness embedding, data compression, or splitting across outputs",
                        size, remaining, MAX_PUSHDATA_BYTES
                    )
                }
                SizeWarning::OpReturnApproaching { size, remaining } => {
                    format!(
                        "OP_RETURN output is {} bytes (only {} bytes remain before the {} byte limit); consider off-chain data storage with an on-chain commitment hash",
                        size, remaining, MAX_OP_RETURN_BYTES
                    )
                }
                SizeWarning::ScriptPubKeyApproaching { size, remaining } => {
                    format!(
                        "ScriptPubKey is {} bytes (only {} bytes remain before the {} byte limit); simplify the output script or use a P2TR commitment",
                        size, remaining, MAX_SCRIPT_PUBKEY_BYTES
                    )
                }
                SizeWarning::WitnessApproaching { size, remaining } => {
                    format!(
                        "witness element is {} bytes (only {} bytes remain before the {} byte limit); consider splitting across multiple witness stack items",
                        size, remaining, MAX_WITNESS_ELEMENT_BYTES
                    )
                }
            })
            .collect()
    }

    // ── Shape export ──

    /// Export the accumulated sizes as a [`Bip110TransactionShape`] for
    /// downstream validation or serialization.
    pub fn to_shape(&self) -> Bip110TransactionShape {
        Bip110TransactionShape::new(
            self.pushdata_sizes.clone(),
            self.op_return_sizes.clone(),
            self.script_pubkey_sizes.clone(),
            self.witness_sizes.clone(),
        )
    }

    /// Number of elements tracked across all categories.
    pub fn element_count(&self) -> usize {
        self.pushdata_sizes.len()
            + self.op_return_sizes.len()
            + self.script_pubkey_sizes.len()
            + self.witness_sizes.len()
    }

    /// Reset all tracked state, keeping the same compliance configuration.
    pub fn reset(&mut self) {
        self.pushdata_sizes.clear();
        self.op_return_sizes.clear();
        self.script_pubkey_sizes.clear();
        self.witness_sizes.clear();
        self.warnings.clear();
    }
}

impl Default for Bip110SizeGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control_model::trust::Bip110Violation;

    #[test]
    fn compliant_transaction_passes_validation() {
        let mut guard = Bip110SizeGuard::new();
        guard
            .track_pushdata(100)
            .track_pushdata(200)
            .track_op_return(60)
            .track_script_pubkey(34)
            .track_witness(128);

        let result = guard.validate();
        assert!(result.is_compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn oversized_pushdata_fails_validation() {
        let mut guard = Bip110SizeGuard::new();
        guard.track_pushdata(300); // exceeds 256

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert_eq!(result.violations.len(), 1);
        assert!(matches!(
            result.violations[0],
            Bip110Violation::PushdataExceedsLimit {
                size: 300,
                max: 256
            }
        ));
    }

    #[test]
    fn oversized_op_return_fails_validation() {
        let mut guard = Bip110SizeGuard::new();
        guard.track_op_return(100); // exceeds 83

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert!(matches!(
            result.violations[0],
            Bip110Violation::OpReturnExceedsLimit { size: 100, max: 83 }
        ));
    }

    #[test]
    fn oversized_script_pubkey_fails_validation() {
        let mut guard = Bip110SizeGuard::new();
        guard.track_script_pubkey(50); // exceeds 34

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert!(matches!(
            result.violations[0],
            Bip110Violation::ScriptPubKeyExceedsLimit { size: 50, max: 34 }
        ));
    }

    #[test]
    fn oversized_witness_fails_validation() {
        let mut guard = Bip110SizeGuard::new();
        guard.track_witness(300); // exceeds 256

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert!(matches!(
            result.violations[0],
            Bip110Violation::WitnessElementExceedsLimit {
                size: 300,
                max: 256
            }
        ));
    }

    #[test]
    fn multiple_violations_aggregated() {
        let mut guard = Bip110SizeGuard::new();
        guard
            .track_pushdata(300)
            .track_op_return(100)
            .track_witness(500);

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert_eq!(result.violations.len(), 3);
    }

    #[test]
    fn at_limit_values_pass() {
        let mut guard = Bip110SizeGuard::new();
        guard
            .track_pushdata(256)
            .track_op_return(83)
            .track_script_pubkey(34)
            .track_witness(256);

        let result = guard.validate();
        assert!(result.is_compliant);
    }

    #[test]
    fn warnings_generated_when_approaching_limits() {
        let mut guard = Bip110SizeGuard::new();
        // 220 is > 80% of 256 (204.8), so should generate warning
        guard.track_pushdata(220);
        // 70 is > 80% of 83 (66.4), so should generate warning
        guard.track_op_return(70);
        // 28 is > 80% of 34 (27.2), so should generate warning
        guard.track_script_pubkey(28);
        // 210 is > 80% of 256 (204.8), so should generate warning
        guard.track_witness(210);

        assert!(guard.has_warnings());
        let suggestions = guard.suggestions();
        assert_eq!(suggestions.len(), 4);
        assert!(suggestions[0].contains("pushdata"));
        assert!(suggestions[1].contains("OP_RETURN"));
        assert!(suggestions[2].contains("ScriptPubKey"));
        assert!(suggestions[3].contains("witness"));
    }

    #[test]
    fn small_elements_generate_no_warnings() {
        let mut guard = Bip110SizeGuard::new();
        guard
            .track_pushdata(32)
            .track_op_return(32)
            .track_script_pubkey(22)
            .track_witness(64);

        assert!(!guard.has_warnings());
        assert!(guard.suggestions().is_empty());
    }

    #[test]
    fn disabled_guard_always_compliant() {
        let mut guard = Bip110SizeGuard::disabled();
        guard.track_pushdata(10_000).track_op_return(10_000);

        let result = guard.validate();
        assert!(result.is_compliant);
    }

    #[test]
    fn custom_limits_respected() {
        use crate::control_model::Bip110Limits;

        let limits = Bip110Limits {
            max_pushdata_bytes: 128,
            max_op_return_bytes: 40,
            max_script_pubkey_bytes: 34,
            max_witness_element_bytes: 128,
        };
        let mut guard = Bip110SizeGuard::with_limits(limits);
        guard.track_pushdata(200); // exceeds custom 128

        let result = guard.validate();
        assert!(!result.is_compliant);
        assert!(matches!(
            result.violations[0],
            Bip110Violation::PushdataExceedsLimit {
                size: 200,
                max: 128
            }
        ));
    }

    #[test]
    fn reset_clears_all_state() {
        let mut guard = Bip110SizeGuard::new();
        guard.track_pushdata(300);
        assert!(!guard.validate().is_compliant);

        guard.reset();
        assert!(guard.validate().is_compliant);
        assert_eq!(guard.element_count(), 0);
        assert!(!guard.has_warnings());
    }

    #[test]
    fn to_shape_exports_correct_sizes() {
        let mut guard = Bip110SizeGuard::new();
        guard
            .track_pushdata(10)
            .track_pushdata(20)
            .track_op_return(30)
            .track_script_pubkey(22)
            .track_witness(64)
            .track_witness(128);

        let shape = guard.to_shape();
        assert_eq!(shape.pushdata_sizes_bytes, vec![10, 20]);
        assert_eq!(shape.op_return_script_pubkey_sizes_bytes, vec![30]);
        assert_eq!(shape.non_op_return_script_pubkey_sizes_bytes, vec![22]);
        assert_eq!(shape.witness_element_sizes_bytes, vec![64, 128]);
    }

    #[test]
    fn element_count_tracks_all_categories() {
        let mut guard = Bip110SizeGuard::new();
        assert_eq!(guard.element_count(), 0);

        guard
            .track_pushdata(1)
            .track_op_return(2)
            .track_script_pubkey(22)
            .track_witness(3);
        assert_eq!(guard.element_count(), 4);
    }

    #[test]
    fn builder_pattern_fluent_api() {
        let result = Bip110SizeGuard::new()
            .track_pushdata(100)
            .track_op_return(60)
            .track_script_pubkey(22)
            .track_witness(64)
            .validate();

        assert!(result.is_compliant);
    }
}
