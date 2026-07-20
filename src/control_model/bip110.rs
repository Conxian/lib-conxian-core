use serde::{Deserialize, Serialize};

pub use super::trust::{Bip110Compliance, Bip110ValidationResult, Bip110Violation};

pub use super::trust::bip110::{
    MAX_OP_RETURN_BYTES, MAX_PUSHDATA_BYTES, MAX_SCRIPT_PUBKEY_BYTES, MAX_WITNESS_ELEMENT_BYTES,
};

/// Canonical BIP-110 size-policy limits used by the core contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110Limits {
    /// Maximum payload size of one applicable pushdata element in bytes.
    pub max_pushdata_bytes: usize,
    /// Maximum full output ScriptPubKey size for one OP_RETURN output in bytes.
    pub max_op_return_bytes: usize,
    /// Maximum ScriptPubKey size for one non-OP_RETURN output in bytes.
    pub max_script_pubkey_bytes: usize,
    /// Maximum size of one applicable witness stack element in bytes.
    pub max_witness_element_bytes: usize,
}

impl Bip110Limits {
    /// Returns the canonical BIP-110 size-policy limits.
    pub const fn canonical() -> Self {
        Self {
            max_pushdata_bytes: MAX_PUSHDATA_BYTES,
            max_op_return_bytes: MAX_OP_RETURN_BYTES,
            max_script_pubkey_bytes: MAX_SCRIPT_PUBKEY_BYTES,
            max_witness_element_bytes: MAX_WITNESS_ELEMENT_BYTES,
        }
    }

    /// Returns the canonical BIP-110 size-policy limits.
    pub const fn new() -> Self {
        Self::canonical()
    }

    /// Validates every occurrence represented by a transaction shape.
    pub fn validate_transaction(&self, shape: &Bip110TransactionShape) -> Bip110ValidationResult {
        Bip110Compliance::with_limits(*self).validate_transaction_shape(
            &shape.pushdata_sizes_bytes,
            &shape.op_return_script_pubkey_sizes_bytes,
            &shape.non_op_return_script_pubkey_sizes_bytes,
            &shape.witness_element_sizes_bytes,
        )
    }
}

impl Default for Bip110Limits {
    fn default() -> Self {
        Self::canonical()
    }
}

/// Size-bearing transaction metadata for the core BIP-110 size-policy contract.
///
/// This is intentionally a shape, not a transaction parser or script-context classifier.
/// Downstream SDK, wallet, and Gateway adapters must inspect the transaction, apply any
/// context-sensitive exceptions, classify constrained occurrences, and populate every relevant
/// vector before calling [`Self::validate`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110TransactionShape {
    /// Payload byte sizes of every pushdata element subject to the core pushdata limit.
    pub pushdata_sizes_bytes: Vec<usize>,
    /// Full output ScriptPubKey byte sizes for every OP_RETURN output subject to the core limit.
    pub op_return_script_pubkey_sizes_bytes: Vec<usize>,
    /// Full ScriptPubKey byte sizes for every non-OP_RETURN output subject to the core limit.
    pub non_op_return_script_pubkey_sizes_bytes: Vec<usize>,
    /// Byte sizes of every applicable witness stack element subject to the core witness limit.
    pub witness_element_sizes_bytes: Vec<usize>,
}

impl Bip110TransactionShape {
    /// Creates a transaction shape from all constrained occurrences supplied by an adapter.
    pub fn new(
        pushdata_sizes_bytes: Vec<usize>,
        op_return_script_pubkey_sizes_bytes: Vec<usize>,
        non_op_return_script_pubkey_sizes_bytes: Vec<usize>,
        witness_element_sizes_bytes: Vec<usize>,
    ) -> Self {
        Self {
            pushdata_sizes_bytes,
            op_return_script_pubkey_sizes_bytes,
            non_op_return_script_pubkey_sizes_bytes,
            witness_element_sizes_bytes,
        }
    }

    /// Validates this shape against the canonical BIP-110 size-policy limits.
    pub fn validate(&self) -> Bip110ValidationResult {
        self.validate_with(&Bip110Compliance::new())
    }

    /// Validates this shape through an existing compliance configuration.
    pub fn validate_with(&self, compliance: &Bip110Compliance) -> Bip110ValidationResult {
        compliance.validate_transaction_shape(
            &self.pushdata_sizes_bytes,
            &self.op_return_script_pubkey_sizes_bytes,
            &self.non_op_return_script_pubkey_sizes_bytes,
            &self.witness_element_sizes_bytes,
        )
    }

    /// Validates this shape through the supplied size-policy limits.
    pub fn validate_with_limits(&self, limits: &Bip110Limits) -> Bip110ValidationResult {
        limits.validate_transaction(self)
    }
}

impl Bip110Compliance {
    /// Validates a transaction shape through this compliance configuration.
    pub fn validate_shape(&self, shape: &Bip110TransactionShape) -> Bip110ValidationResult {
        shape.validate_with(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_limits_have_expected_defaults() {
        let expected = Bip110Limits {
            max_pushdata_bytes: 256,
            max_op_return_bytes: 83,
            max_script_pubkey_bytes: 34,
            max_witness_element_bytes: 256,
        };

        assert_eq!(Bip110Limits::default(), expected);
        assert_eq!(Bip110Limits::new(), expected);
        assert_eq!(Bip110Limits::canonical(), expected);
    }

    #[test]
    fn limits_round_trip_through_json() {
        let limits = Bip110Limits {
            max_pushdata_bytes: 17,
            max_op_return_bytes: 19,
            max_script_pubkey_bytes: 23,
            max_witness_element_bytes: 29,
        };
        let encoded = serde_json::to_string(&limits).expect("limits should serialize");
        let decoded: Bip110Limits =
            serde_json::from_str(&encoded).expect("limits should deserialize");

        assert_eq!(decoded, limits);
    }

    #[test]
    fn transaction_shape_round_trips_through_json() {
        let shape =
            Bip110TransactionShape::new(vec![32, 33], vec![40, 41], vec![22, 25], vec![32, 33]);
        let encoded = serde_json::to_string(&shape).expect("shape should serialize");
        let decoded: Bip110TransactionShape =
            serde_json::from_str(&encoded).expect("shape should deserialize");

        assert_eq!(decoded, shape);
    }

    #[test]
    fn vector_boundaries_accept_exact_limits_and_catch_later_oversized_entries() {
        let at_limits = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES - 1, MAX_PUSHDATA_BYTES],
            vec![MAX_OP_RETURN_BYTES - 1, MAX_OP_RETURN_BYTES],
            vec![MAX_SCRIPT_PUBKEY_BYTES - 1, MAX_SCRIPT_PUBKEY_BYTES],
            vec![MAX_WITNESS_ELEMENT_BYTES - 1, MAX_WITNESS_ELEMENT_BYTES],
        );
        assert_eq!(at_limits.validate(), Bip110ValidationResult::compliant());

        let later_oversized = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES, MAX_PUSHDATA_BYTES + 1],
            vec![MAX_OP_RETURN_BYTES, MAX_OP_RETURN_BYTES + 1],
            vec![MAX_SCRIPT_PUBKEY_BYTES, MAX_SCRIPT_PUBKEY_BYTES + 1],
            vec![MAX_WITNESS_ELEMENT_BYTES, MAX_WITNESS_ELEMENT_BYTES + 1],
        );
        let result = later_oversized.validate();

        assert_eq!(
            result,
            Bip110ValidationResult::non_compliant(vec![
                Bip110Violation::PushdataExceedsLimit {
                    size: MAX_PUSHDATA_BYTES + 1,
                    max: MAX_PUSHDATA_BYTES,
                },
                Bip110Violation::OpReturnExceedsLimit {
                    size: MAX_OP_RETURN_BYTES + 1,
                    max: MAX_OP_RETURN_BYTES,
                },
                Bip110Violation::ScriptPubKeyExceedsLimit {
                    size: MAX_SCRIPT_PUBKEY_BYTES + 1,
                    max: MAX_SCRIPT_PUBKEY_BYTES,
                },
                Bip110Violation::WitnessElementExceedsLimit {
                    size: MAX_WITNESS_ELEMENT_BYTES + 1,
                    max: MAX_WITNESS_ELEMENT_BYTES,
                },
            ])
        );
    }

    #[test]
    fn aggregate_validation_preserves_all_violations_across_all_vectors() {
        let shape = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES + 1, MAX_PUSHDATA_BYTES + 2],
            vec![MAX_OP_RETURN_BYTES + 1, MAX_OP_RETURN_BYTES + 2],
            vec![MAX_SCRIPT_PUBKEY_BYTES + 1, MAX_SCRIPT_PUBKEY_BYTES + 2],
            vec![MAX_WITNESS_ELEMENT_BYTES + 1, MAX_WITNESS_ELEMENT_BYTES + 2],
        );

        let result = shape.validate();

        assert!(!result.is_compliant);
        assert_eq!(result.violations.len(), 8);
        assert_eq!(
            result.violations,
            vec![
                Bip110Violation::PushdataExceedsLimit {
                    size: MAX_PUSHDATA_BYTES + 1,
                    max: MAX_PUSHDATA_BYTES,
                },
                Bip110Violation::PushdataExceedsLimit {
                    size: MAX_PUSHDATA_BYTES + 2,
                    max: MAX_PUSHDATA_BYTES,
                },
                Bip110Violation::OpReturnExceedsLimit {
                    size: MAX_OP_RETURN_BYTES + 1,
                    max: MAX_OP_RETURN_BYTES,
                },
                Bip110Violation::OpReturnExceedsLimit {
                    size: MAX_OP_RETURN_BYTES + 2,
                    max: MAX_OP_RETURN_BYTES,
                },
                Bip110Violation::ScriptPubKeyExceedsLimit {
                    size: MAX_SCRIPT_PUBKEY_BYTES + 1,
                    max: MAX_SCRIPT_PUBKEY_BYTES,
                },
                Bip110Violation::ScriptPubKeyExceedsLimit {
                    size: MAX_SCRIPT_PUBKEY_BYTES + 2,
                    max: MAX_SCRIPT_PUBKEY_BYTES,
                },
                Bip110Violation::WitnessElementExceedsLimit {
                    size: MAX_WITNESS_ELEMENT_BYTES + 1,
                    max: MAX_WITNESS_ELEMENT_BYTES,
                },
                Bip110Violation::WitnessElementExceedsLimit {
                    size: MAX_WITNESS_ELEMENT_BYTES + 2,
                    max: MAX_WITNESS_ELEMENT_BYTES,
                },
            ]
        );
    }

    #[test]
    fn custom_limits_map_each_shape_field_to_the_exact_violation_limit() {
        let limits = Bip110Limits {
            max_pushdata_bytes: 1,
            max_op_return_bytes: 2,
            max_script_pubkey_bytes: 3,
            max_witness_element_bytes: 4,
        };
        let shape = Bip110TransactionShape::new(vec![1, 2], vec![2, 3], vec![3, 4], vec![4, 5]);

        let result = shape.validate_with_limits(&limits);

        assert_eq!(
            result,
            Bip110ValidationResult::non_compliant(vec![
                Bip110Violation::PushdataExceedsLimit { size: 2, max: 1 },
                Bip110Violation::OpReturnExceedsLimit { size: 3, max: 2 },
                Bip110Violation::ScriptPubKeyExceedsLimit { size: 4, max: 3 },
                Bip110Violation::WitnessElementExceedsLimit { size: 5, max: 4 },
            ])
        );
    }

    #[test]
    fn facades_produce_the_expected_result_for_the_same_shape() {
        let shape = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES + 1],
            vec![MAX_OP_RETURN_BYTES + 1],
            vec![MAX_SCRIPT_PUBKEY_BYTES + 1],
            vec![MAX_WITNESS_ELEMENT_BYTES + 1],
        );
        let expected = Bip110ValidationResult::non_compliant(vec![
            Bip110Violation::PushdataExceedsLimit {
                size: MAX_PUSHDATA_BYTES + 1,
                max: MAX_PUSHDATA_BYTES,
            },
            Bip110Violation::OpReturnExceedsLimit {
                size: MAX_OP_RETURN_BYTES + 1,
                max: MAX_OP_RETURN_BYTES,
            },
            Bip110Violation::ScriptPubKeyExceedsLimit {
                size: MAX_SCRIPT_PUBKEY_BYTES + 1,
                max: MAX_SCRIPT_PUBKEY_BYTES,
            },
            Bip110Violation::WitnessElementExceedsLimit {
                size: MAX_WITNESS_ELEMENT_BYTES + 1,
                max: MAX_WITNESS_ELEMENT_BYTES,
            },
        ]);
        let compliance = Bip110Compliance::new();
        let limits = Bip110Limits::canonical();

        assert_eq!(shape.validate(), expected);
        assert_eq!(shape.validate_with(&compliance), expected);
        assert_eq!(shape.validate_with_limits(&limits), expected);
        assert_eq!(compliance.validate_shape(&shape), expected);
        assert_eq!(limits.validate_transaction(&shape), expected);
    }

    #[test]
    fn disabled_compliance_accepts_invalid_shape_without_violations() {
        let shape = Bip110TransactionShape::new(
            vec![usize::MAX],
            vec![usize::MAX],
            vec![usize::MAX],
            vec![usize::MAX],
        );

        let result = Bip110Compliance::disabled().validate_shape(&shape);

        assert!(result.is_compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn default_compliance_remains_disabled_with_canonical_limits() {
        let compliance = Bip110Compliance::default();

        assert!(!compliance.is_enabled());
        assert_eq!(compliance.limits(), &Bip110Limits::canonical());
        assert!(compliance.validate_pushdata(usize::MAX).is_compliant);
    }

    #[test]
    fn usize_max_is_compared_without_arithmetic_overflow() {
        let shape = Bip110TransactionShape::new(
            vec![usize::MAX],
            vec![usize::MAX],
            vec![usize::MAX],
            vec![usize::MAX],
        );
        let limits = Bip110Limits {
            max_pushdata_bytes: usize::MAX,
            max_op_return_bytes: usize::MAX,
            max_script_pubkey_bytes: usize::MAX,
            max_witness_element_bytes: usize::MAX,
        };

        assert!(shape.validate_with_limits(&limits).is_compliant);
    }
}
