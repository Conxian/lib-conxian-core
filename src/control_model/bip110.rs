use serde::{Deserialize, Serialize};

pub use super::trust::{Bip110Compliance, Bip110ValidationResult, Bip110Violation};

pub use super::trust::bip110::{
    MAX_OP_RETURN_BYTES, MAX_PUSHDATA_BYTES, MAX_SCRIPT_PUBKEY_BYTES, MAX_WITNESS_ELEMENT_BYTES,
};

/// Canonical BIP-110 size limits used by the core protocol contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110Limits {
    /// Maximum size of one pushdata element in bytes.
    pub max_pushdata_bytes: usize,
    /// Maximum size of an OP_RETURN output in bytes.
    pub max_op_return_bytes: usize,
    /// Maximum size of a ScriptPubKey in bytes.
    pub max_script_pubkey_bytes: usize,
    /// Maximum size of one witness element in bytes.
    pub max_witness_element_bytes: usize,
}

impl Bip110Limits {
    /// Returns the canonical BIP-110 limits.
    pub const fn canonical() -> Self {
        Self {
            max_pushdata_bytes: MAX_PUSHDATA_BYTES,
            max_op_return_bytes: MAX_OP_RETURN_BYTES,
            max_script_pubkey_bytes: MAX_SCRIPT_PUBKEY_BYTES,
            max_witness_element_bytes: MAX_WITNESS_ELEMENT_BYTES,
        }
    }

    /// Returns the canonical BIP-110 limits.
    pub const fn new() -> Self {
        Self::canonical()
    }

    /// Validates a transaction shape through the existing compliance engine.
    pub fn validate_transaction(&self, shape: &Bip110TransactionShape) -> Bip110ValidationResult {
        Bip110Compliance::with_limits(*self).validate_transaction(
            &shape.pushdata_sizes_bytes,
            shape.op_return_size_bytes,
            shape.script_pubkey_size_bytes,
            &shape.witness_element_sizes_bytes,
        )
    }
}

impl Default for Bip110Limits {
    fn default() -> Self {
        Self::canonical()
    }
}

/// The size-bearing transaction metadata required by the core BIP-110 validator.
///
/// This is intentionally a shape, not a transaction parser. Downstream SDK and
/// Gateway adapters are responsible for measuring transaction components and
/// populating these fields before calling [`Self::validate`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bip110TransactionShape {
    /// Byte sizes of each pushdata element in the transaction.
    pub pushdata_sizes_bytes: Vec<usize>,
    /// Byte size of the OP_RETURN output, when one is present.
    pub op_return_size_bytes: Option<usize>,
    /// Byte size of the transaction's ScriptPubKey under validation.
    pub script_pubkey_size_bytes: usize,
    /// Byte sizes of each witness element in the transaction.
    pub witness_element_sizes_bytes: Vec<usize>,
}

impl Bip110TransactionShape {
    /// Creates a transaction shape from the inputs accepted by the aggregate validator.
    pub fn new(
        pushdata_sizes_bytes: Vec<usize>,
        op_return_size_bytes: Option<usize>,
        script_pubkey_size_bytes: usize,
        witness_element_sizes_bytes: Vec<usize>,
    ) -> Self {
        Self {
            pushdata_sizes_bytes,
            op_return_size_bytes,
            script_pubkey_size_bytes,
            witness_element_sizes_bytes,
        }
    }

    /// Validates this shape against the canonical BIP-110 limits.
    pub fn validate(&self) -> Bip110ValidationResult {
        self.validate_with(&Bip110Compliance::new())
    }

    /// Validates this shape through an existing compliance configuration.
    pub fn validate_with(&self, compliance: &Bip110Compliance) -> Bip110ValidationResult {
        compliance.validate_transaction(
            &self.pushdata_sizes_bytes,
            self.op_return_size_bytes,
            self.script_pubkey_size_bytes,
            &self.witness_element_sizes_bytes,
        )
    }

    /// Validates this shape through the supplied limits.
    pub fn validate_with_limits(&self, limits: &Bip110Limits) -> Bip110ValidationResult {
        limits.validate_transaction(self)
    }
}

impl Bip110Compliance {
    /// Validates a canonical transaction shape through this compliance configuration.
    pub fn validate_shape(&self, shape: &Bip110TransactionShape) -> Bip110ValidationResult {
        shape.validate_with(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_shape() -> Bip110TransactionShape {
        Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES],
            Some(MAX_OP_RETURN_BYTES),
            MAX_SCRIPT_PUBKEY_BYTES,
            vec![MAX_WITNESS_ELEMENT_BYTES],
        )
    }

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
    fn every_boundary_is_valid_at_the_limit_and_rejected_at_limit_plus_one() {
        let compliance = Bip110Compliance::new();

        assert!(
            compliance
                .validate_pushdata(MAX_PUSHDATA_BYTES)
                .is_compliant
        );
        assert!(
            !compliance
                .validate_pushdata(MAX_PUSHDATA_BYTES + 1)
                .is_compliant
        );

        assert!(
            compliance
                .validate_op_return(MAX_OP_RETURN_BYTES)
                .is_compliant
        );
        assert!(
            !compliance
                .validate_op_return(MAX_OP_RETURN_BYTES + 1)
                .is_compliant
        );

        assert!(
            compliance
                .validate_script_pubkey(MAX_SCRIPT_PUBKEY_BYTES)
                .is_compliant
        );
        assert!(
            !compliance
                .validate_script_pubkey(MAX_SCRIPT_PUBKEY_BYTES + 1)
                .is_compliant
        );

        assert!(
            compliance
                .validate_witness_element(MAX_WITNESS_ELEMENT_BYTES)
                .is_compliant
        );
        assert!(
            !compliance
                .validate_witness_element(MAX_WITNESS_ELEMENT_BYTES + 1)
                .is_compliant
        );

        let mut shape = valid_shape();
        assert!(shape.validate().is_compliant);

        shape.pushdata_sizes_bytes = vec![MAX_PUSHDATA_BYTES + 1];
        assert!(!shape.validate().is_compliant);
        shape.pushdata_sizes_bytes = vec![MAX_PUSHDATA_BYTES];

        shape.op_return_size_bytes = Some(MAX_OP_RETURN_BYTES + 1);
        assert!(!shape.validate().is_compliant);
        shape.op_return_size_bytes = Some(MAX_OP_RETURN_BYTES);

        shape.script_pubkey_size_bytes = MAX_SCRIPT_PUBKEY_BYTES + 1;
        assert!(!shape.validate().is_compliant);
        shape.script_pubkey_size_bytes = MAX_SCRIPT_PUBKEY_BYTES;

        shape.witness_element_sizes_bytes = vec![MAX_WITNESS_ELEMENT_BYTES + 1];
        assert!(!shape.validate().is_compliant);
    }

    #[test]
    fn transaction_shape_round_trips_through_json() {
        let shape = Bip110TransactionShape::new(vec![32, 33], Some(40), 34, vec![32, 33]);
        let encoded = serde_json::to_string(&shape).expect("shape should serialize");
        let decoded: Bip110TransactionShape =
            serde_json::from_str(&encoded).expect("shape should deserialize");

        assert_eq!(decoded, shape);
    }

    #[test]
    fn aggregate_validation_preserves_all_violations() {
        let shape = Bip110TransactionShape::new(
            vec![MAX_PUSHDATA_BYTES + 1, MAX_PUSHDATA_BYTES + 2],
            Some(MAX_OP_RETURN_BYTES + 1),
            MAX_SCRIPT_PUBKEY_BYTES + 1,
            vec![MAX_WITNESS_ELEMENT_BYTES + 1, MAX_WITNESS_ELEMENT_BYTES + 2],
        );

        let result = shape.validate();

        assert!(!result.is_compliant);
        assert_eq!(result.violations.len(), 6);
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
                Bip110Violation::ScriptPubKeyExceedsLimit {
                    size: MAX_SCRIPT_PUBKEY_BYTES + 1,
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
    fn shape_and_limits_facades_delegate_to_existing_compliance_validation() {
        let shape = valid_shape();
        let compliance = Bip110Compliance::new();
        let expected = compliance.validate_transaction(
            &shape.pushdata_sizes_bytes,
            shape.op_return_size_bytes,
            shape.script_pubkey_size_bytes,
            &shape.witness_element_sizes_bytes,
        );

        assert_eq!(shape.validate(), expected);
        assert_eq!(shape.validate_with(&compliance), expected);
        assert_eq!(
            shape.validate_with_limits(&Bip110Limits::canonical()),
            expected
        );
        assert_eq!(
            Bip110Limits::canonical().validate_transaction(&shape),
            expected
        );
        assert_eq!(compliance.validate_shape(&shape), expected);
    }

    #[test]
    fn explicit_limits_are_used_by_the_same_compliance_engine() {
        let limits = Bip110Limits {
            max_pushdata_bytes: 1,
            max_op_return_bytes: 2,
            max_script_pubkey_bytes: 3,
            max_witness_element_bytes: 4,
        };
        let shape = Bip110TransactionShape::new(vec![2], Some(3), 4, vec![5]);

        let result = shape.validate_with_limits(&limits);

        assert_eq!(result.violations.len(), 4);
        assert!(result.violations.iter().all(|violation| match violation {
            Bip110Violation::PushdataExceedsLimit { max, .. }
            | Bip110Violation::OpReturnExceedsLimit { max, .. }
            | Bip110Violation::ScriptPubKeyExceedsLimit { max, .. }
            | Bip110Violation::WitnessElementExceedsLimit { max, .. } => *max <= 4,
        }));
    }
}
