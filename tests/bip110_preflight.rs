use lib_conxian_core::control_model::{
    Bip110Compliance, Bip110MeasurementField, Bip110OperationContext, Bip110PreflightError,
    Bip110PreflightFinding, Bip110PreflightMeasurements, Bip110PreflightPhase,
    Bip110PreflightRequest, Bip110PreflightValidator, Bip110PreflightViolationCode,
    Bip110TransactionShape, BIP110_PREFLIGHT_API_VERSION,
};

fn measurements(
    pushdata: Vec<u64>,
    op_return: Vec<u64>,
    non_op_return: Vec<u64>,
    witness: Vec<u64>,
) -> Bip110PreflightMeasurements {
    Bip110PreflightMeasurements::new(pushdata, op_return, non_op_return, witness)
}

fn request(
    phase: Bip110PreflightPhase,
    context: Bip110OperationContext,
    measurements: Bip110PreflightMeasurements,
) -> Bip110PreflightRequest {
    Bip110PreflightRequest::new(phase, context, measurements)
}

#[test]
fn exact_limits_pass_and_limit_plus_one_fails_for_all_fields() {
    let exact = request(
        Bip110PreflightPhase::PreConstruction,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![256], vec![83], vec![34], vec![256]),
    );
    assert!(exact.validate().is_compliant);

    let oversized = request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![257], vec![84], vec![35], vec![257]),
    );
    let result = oversized.validate();
    let violations: Vec<_> = result.violations().collect();

    assert!(!result.is_compliant);
    assert_eq!(violations.len(), 4);
    assert_eq!(
        violations
            .iter()
            .map(|violation| (
                violation.code,
                violation.field,
                violation.index,
                violation.actual_bytes,
                violation.max_bytes,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                Bip110PreflightViolationCode::PushdataExceedsLimit,
                Bip110MeasurementField::Pushdata,
                0,
                257,
                256,
            ),
            (
                Bip110PreflightViolationCode::OpReturnExceedsLimit,
                Bip110MeasurementField::OpReturnScriptPubkey,
                0,
                84,
                83,
            ),
            (
                Bip110PreflightViolationCode::ScriptPubkeyExceedsLimit,
                Bip110MeasurementField::NonOpReturnScriptPubkey,
                0,
                35,
                34,
            ),
            (
                Bip110PreflightViolationCode::WitnessElementExceedsLimit,
                Bip110MeasurementField::WitnessElement,
                0,
                257,
                256,
            ),
        ]
    );
}

#[test]
fn zero_values_and_empty_vectors_are_valid_for_supported_context() {
    let zeroes = request(
        Bip110PreflightPhase::PreConstruction,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![0], vec![0], vec![0], vec![0]),
    );
    assert!(zeroes.validate().is_compliant);

    let empty = request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::BitcoinTransaction,
        Bip110PreflightMeasurements::default(),
    );
    assert!(empty.validate().is_compliant);
}

#[test]
fn multiple_findings_preserve_category_and_occurrence_order() {
    let result = request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![257, 258], vec![84, 85], vec![35, 36], vec![257, 258]),
    )
    .validate();

    let findings: Vec<_> = result
        .findings
        .iter()
        .map(|finding| match finding {
            Bip110PreflightFinding::Violation(violation) => {
                (violation.field, violation.index, violation.actual_bytes)
            }
            Bip110PreflightFinding::Error(error) => panic!("unexpected structural error: {error}"),
        })
        .collect();

    assert_eq!(
        findings,
        vec![
            (Bip110MeasurementField::Pushdata, 0, 257),
            (Bip110MeasurementField::Pushdata, 1, 258),
            (Bip110MeasurementField::OpReturnScriptPubkey, 0, 84),
            (Bip110MeasurementField::OpReturnScriptPubkey, 1, 85),
            (Bip110MeasurementField::NonOpReturnScriptPubkey, 0, 35),
            (Bip110MeasurementField::NonOpReturnScriptPubkey, 1, 36),
            (Bip110MeasurementField::WitnessElement, 0, 257),
            (Bip110MeasurementField::WitnessElement, 1, 258),
        ]
    );
}

#[test]
fn structural_findings_precede_size_findings_and_fail_closed() {
    let request = Bip110PreflightRequest::with_api_version(
        BIP110_PREFLIGHT_API_VERSION + 1,
        Bip110PreflightPhase::PreConstruction,
        Bip110OperationContext::Unknown("future_context".to_owned()),
        measurements(vec![257], vec![84], vec![35], vec![257]),
    );
    let result = request.validate();

    assert!(!result.is_compliant);
    assert_eq!(result.findings.len(), 2);
    assert!(matches!(
        &result.findings[0],
        Bip110PreflightFinding::Error(Bip110PreflightError::UnsupportedApiVersion { .. })
    ));
    assert!(matches!(
        &result.findings[1],
        Bip110PreflightFinding::Error(Bip110PreflightError::UnknownContext { context })
            if context == "future_context"
    ));
}

#[test]
fn known_unsupported_context_with_empty_vectors_does_not_pass() {
    let request = request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::Taproot,
        Bip110PreflightMeasurements::default(),
    );
    let result = request.validate();

    assert!(!result.is_compliant);
    assert!(matches!(
        result.findings.first(),
        Some(Bip110PreflightFinding::Error(
            Bip110PreflightError::UnsupportedContext { context }
        )) if context == "taproot"
    ));
}

#[test]
fn unknown_context_strings_round_trip_and_fail_closed() {
    let encoded = r#"{
        "api_version": 1,
        "phase": "post_serialization",
        "context": "future_protocol_context",
        "measurements": {
            "pushdata_sizes_bytes": [],
            "op_return_script_pubkey_sizes_bytes": [],
            "non_op_return_script_pubkey_sizes_bytes": [],
            "witness_element_sizes_bytes": []
        }
    }"#;
    let request: Bip110PreflightRequest = serde_json::from_str(encoded).unwrap();
    let round_trip = serde_json::to_string(&request).unwrap();
    let decoded: Bip110PreflightRequest = serde_json::from_str(&round_trip).unwrap();

    assert_eq!(decoded, request);
    assert_eq!(decoded.context.as_str(), "future_protocol_context");
    assert!(!decoded.validate().is_compliant);
    assert!(matches!(
        decoded.validate().findings.first(),
        Some(Bip110PreflightFinding::Error(
            Bip110PreflightError::UnknownContext { context }
        )) if context == "future_protocol_context"
    ));
}

#[test]
fn known_context_wire_values_are_stable() {
    let contexts = [
        (
            Bip110OperationContext::BitcoinTransaction,
            "bitcoin_transaction",
        ),
        (Bip110OperationContext::Taproot, "taproot"),
        (Bip110OperationContext::Tapscript, "tapscript"),
        (
            Bip110OperationContext::TaprootScriptPath,
            "taproot_script_path",
        ),
        (Bip110OperationContext::TaprootKeyPath, "taproot_key_path"),
        (Bip110OperationContext::Miniscript, "miniscript"),
        (Bip110OperationContext::Dlc, "dlc"),
        (Bip110OperationContext::Lightning, "lightning"),
        (Bip110OperationContext::Rgb, "rgb"),
        (Bip110OperationContext::Babylon, "babylon"),
        (Bip110OperationContext::Fedimint, "fedimint"),
        (Bip110OperationContext::Stacks, "stacks"),
        (Bip110OperationContext::Liquid, "liquid"),
    ];

    for (context, expected_wire_value) in contexts {
        let encoded = serde_json::to_string(&context).unwrap();
        assert_eq!(encoded, format!("\"{expected_wire_value}\""));
        let decoded: Bip110OperationContext = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, context);
    }
}

#[test]
fn request_result_error_and_violation_round_trip_through_json() {
    let request = request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![257], vec![], vec![], vec![]),
    );
    let encoded_request = serde_json::to_string(&request).unwrap();
    let decoded_request: Bip110PreflightRequest = serde_json::from_str(&encoded_request).unwrap();
    assert_eq!(decoded_request, request);

    let result = request.validate();
    let encoded_result = serde_json::to_string(&result).unwrap();
    let decoded_result: lib_conxian_core::control_model::Bip110PreflightResult =
        serde_json::from_str(&encoded_result).unwrap();
    assert_eq!(decoded_result, result);

    let error = Bip110PreflightError::UnsupportedApiVersion {
        requested: 2,
        supported: 1,
    };
    let encoded_error = serde_json::to_string(&error).unwrap();
    let decoded_error: Bip110PreflightError = serde_json::from_str(&encoded_error).unwrap();
    assert_eq!(decoded_error, error);

    let violation = result.violations().next().unwrap();
    let encoded_violation = serde_json::to_string(violation).unwrap();
    let decoded_violation: lib_conxian_core::control_model::Bip110PreflightViolation =
        serde_json::from_str(&encoded_violation).unwrap();
    assert_eq!(decoded_violation, *violation);
}

#[test]
fn checked_measurement_conversion_never_truncates() {
    let measurements = Bip110PreflightMeasurements::new(vec![u64::MAX], vec![], vec![], vec![]);

    #[cfg(target_pointer_width = "32")]
    assert!(matches!(
        measurements.try_into_transaction_shape(),
        Err(Bip110PreflightError::IntegerOverflow {
            field: Bip110MeasurementField::Pushdata,
            index: 0,
            actual_bytes: u64::MAX,
        })
    ));

    #[cfg(not(target_pointer_width = "32"))]
    assert_eq!(
        measurements
            .try_into_transaction_shape()
            .unwrap()
            .pushdata_sizes_bytes,
        vec![usize::try_from(u64::MAX).unwrap()]
    );
}

#[test]
fn disabled_compliance_is_rejected_instead_of_becoming_fail_open() {
    let error = Bip110PreflightValidator::with_compliance(Bip110Compliance::default())
        .expect_err("default compliance is intentionally disabled");
    assert_eq!(error.code(), "compliance_disabled");

    let enabled = Bip110PreflightValidator::new();
    let result = enabled.validate(&request(
        Bip110PreflightPhase::PreConstruction,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![257], vec![], vec![], vec![]),
    ));
    assert!(!result.is_compliant);
}

#[test]
fn custom_enabled_compliance_is_composed_without_replacing_wire_types() {
    let compliance = Bip110Compliance::with_limits(lib_conxian_core::control_model::Bip110Limits {
        max_pushdata_bytes: 10,
        max_op_return_bytes: 20,
        max_script_pubkey_bytes: 30,
        max_witness_element_bytes: 40,
    });
    let validator = Bip110PreflightValidator::with_compliance(compliance).unwrap();
    let result = validator.validate(&request(
        Bip110PreflightPhase::PostSerialization,
        Bip110OperationContext::BitcoinTransaction,
        measurements(vec![11], vec![21], vec![31], vec![41]),
    ));

    assert_eq!(
        result
            .violations()
            .map(|violation| violation.max_bytes)
            .collect::<Vec<_>>(),
        vec![10, 20, 30, 40]
    );
}

#[test]
fn measurements_convert_to_the_existing_transaction_shape() {
    let measurements = measurements(vec![1, 2], vec![3], vec![4], vec![5, 6]);
    let shape = measurements.try_into_transaction_shape().unwrap();

    assert_eq!(
        shape,
        Bip110TransactionShape::new(vec![1, 2], vec![3], vec![4], vec![5, 6])
    );
}
