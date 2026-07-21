use lib_conxian_core::bitcoin::taproot::{
    classify_taproot_leaf_version, inspect_control_block, inspect_taproot_witness,
    validate_control_block, validate_key_path_signature, validate_miniscript_handoff,
    validate_miniscript_policy_metadata, validate_p2tr_witness_program, validate_taproot_witness,
    BitcoinBoundaryError, BitcoinBoundaryErrorCategory, BitcoinBoundaryErrorCode,
    MiniscriptCapability, MiniscriptContext, MiniscriptHandoff, MiniscriptPolicyKind,
    MiniscriptPolicyMetadata, TaprootLeafVersionSupport, TaprootParity,
    TaprootWitnessClassification, ValidationClaim, KEY_PATH_SIGNATURE_BYTES,
    KEY_PATH_SIGNATURE_WITH_SIGHASH_BYTES, MAX_TAPROOT_CONTROL_BLOCK_BYTES,
    MAX_TAPROOT_MERKLE_DEPTH, MINISCRIPT_HANDOFF_API_VERSION, P2TR_WITNESS_PROGRAM_BYTES,
    P2TR_WITNESS_VERSION, TAPROOT_CONTROL_BLOCK_BASE_BYTES, TAPROOT_MERKLE_PATH_NODE_BYTES,
};

fn control_block(control_byte: u8, depth: usize) -> Vec<u8> {
    let mut bytes = vec![control_byte; TAPROOT_CONTROL_BLOCK_BASE_BYTES];
    bytes.extend(std::iter::repeat_n(
        0x22,
        depth * TAPROOT_MERKLE_PATH_NODE_BYTES,
    ));
    bytes
}

#[test]
fn p2tr_v1_program_shape_accepts_exactly_32_bytes() {
    let program = [0x11; P2TR_WITNESS_PROGRAM_BYTES];
    let shape = validate_p2tr_witness_program(P2TR_WITNESS_VERSION, &program)
        .expect("32-byte v1 witness program is structurally valid");

    assert_eq!(shape.witness_version, P2TR_WITNESS_VERSION);
    assert_eq!(shape.program_length_bytes, 32);
    assert_eq!(shape.program, program);
    assert_eq!(shape.claim, ValidationClaim::StructuralOnly);
    assert!(!shape.claim.cryptographic_verification_performed());
    assert!(!shape.claim.runtime_execution_performed());
}

#[test]
fn p2tr_program_rejects_wrong_length_and_classifies_other_versions_as_unsupported() {
    let wrong_length = validate_p2tr_witness_program(P2TR_WITNESS_VERSION, &[0x11; 31])
        .expect_err("31-byte witness program must fail");
    assert_eq!(
        wrong_length.category,
        BitcoinBoundaryErrorCategory::Malformed
    );
    assert_eq!(
        wrong_length.code,
        BitcoinBoundaryErrorCode::WitnessProgramWrongLength
    );

    let wrong_version = validate_p2tr_witness_program(0, &[0x11; 32])
        .expect_err("v0 is not a P2TR witness program");
    assert_eq!(
        wrong_version.category,
        BitcoinBoundaryErrorCategory::Unsupported
    );
    assert_eq!(
        wrong_version.code,
        BitcoinBoundaryErrorCode::UnsupportedWitnessVersion
    );
}

#[test]
fn key_path_signature_vectors_are_shape_only() {
    let bare = validate_key_path_signature(&[0x42; KEY_PATH_SIGNATURE_BYTES])
        .expect("64-byte Schnorr signature shape is valid");
    assert_eq!(bare.length_bytes, 64);
    assert_eq!(bare.explicit_sighash_byte, None);

    let mut explicit = vec![0x42; KEY_PATH_SIGNATURE_WITH_SIGHASH_BYTES];
    explicit[64] = 0x01;
    let explicit_shape = validate_key_path_signature(&explicit)
        .expect("65-byte signature with non-zero sighash byte is valid");
    assert_eq!(explicit_shape.length_bytes, 65);
    assert_eq!(explicit_shape.explicit_sighash_byte, Some(0x01));

    let mut zero_sighash = explicit;
    zero_sighash[64] = 0;
    let error = validate_key_path_signature(&zero_sighash)
        .expect_err("explicit zero sighash byte is not a valid key-path shape");
    assert_eq!(error.category, BitcoinBoundaryErrorCategory::Malformed);
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::KeyPathSignatureZeroSighash
    );

    let error =
        validate_key_path_signature(&[0x42; 63]).expect_err("short signature must fail closed");
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::KeyPathSignatureWrongLength
    );
}

#[test]
fn control_block_vectors_cover_current_versions_bounds_and_malformed_lengths() {
    let even = control_block(0xc0, 0);
    let even_shape = validate_control_block(&even).expect("0xc0 control byte is current");
    assert_eq!(even_shape.parity, TaprootParity::Even);
    assert_eq!(even_shape.leaf_version, 0xc0);
    assert_eq!(even_shape.merkle_path_depth, 0);
    assert_eq!(even_shape.serialized_length_bytes, 33);
    assert_eq!(even_shape.internal_key, [0xc0; 32]);

    let odd = control_block(0xc1, 1);
    let odd_shape = validate_control_block(&odd).expect("0xc1 encodes current Tapscript");
    assert_eq!(odd_shape.parity, TaprootParity::Odd);
    assert_eq!(odd_shape.leaf_version, 0xc0);
    assert_eq!(odd_shape.merkle_path_depth, 1);

    let maximum = control_block(0xc0, MAX_TAPROOT_MERKLE_DEPTH);
    let maximum_shape = validate_control_block(&maximum).expect("BIP-341 depth 128 is valid");
    assert_eq!(
        maximum_shape.serialized_length_bytes as usize,
        MAX_TAPROOT_CONTROL_BLOCK_BYTES
    );
    assert_eq!(maximum_shape.merkle_path_depth, 128);

    let too_short = inspect_control_block(&[0xc0; 32]).expect_err("control block needs 33 bytes");
    assert_eq!(
        too_short.code,
        BitcoinBoundaryErrorCode::ControlBlockTooShort
    );

    let misaligned = inspect_control_block(&[0xc0; 34]).expect_err("path must be 32-byte aligned");
    assert_eq!(
        misaligned.code,
        BitcoinBoundaryErrorCode::ControlBlockLengthMisaligned
    );

    let too_deep = control_block(0xc0, MAX_TAPROOT_MERKLE_DEPTH + 1);
    let too_deep_error = inspect_control_block(&too_deep).expect_err("depth 129 is invalid");
    assert_eq!(
        too_deep_error.code,
        BitcoinBoundaryErrorCode::ControlBlockDepthExceeded
    );
}

#[test]
fn future_leaf_versions_are_downstream_owned_not_malformed() {
    assert_eq!(
        classify_taproot_leaf_version(0xc0),
        TaprootLeafVersionSupport::CurrentTapscript
    );
    assert_eq!(
        classify_taproot_leaf_version(0xc1),
        TaprootLeafVersionSupport::CurrentTapscript
    );

    let future = control_block(0xe0, 0);
    let inspected = inspect_control_block(&future).expect("future shape remains inspectable");
    assert_eq!(
        inspected.leaf_version_support,
        TaprootLeafVersionSupport::FutureOrUnknown { leaf_version: 0xe0 }
    );

    let error = validate_control_block(&future).expect_err("future leaf version must fail closed");
    assert_eq!(
        error.category,
        BitcoinBoundaryErrorCategory::DownstreamOwned
    );
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::UnknownTaprootLeafVersion
    );
}

#[test]
fn witness_classification_identifies_key_path_annex_and_script_path_positions() {
    let key_path = vec![vec![0x42; 64]];
    let key_classification = validate_taproot_witness(&key_path).expect("key path is valid shape");
    assert!(matches!(
        key_classification,
        TaprootWitnessClassification::KeyPath {
            annex: None,
            signature: _,
            claim: ValidationClaim::StructuralOnly,
        }
    ));

    let key_path_with_annex = vec![vec![0x42; 64], vec![0x50, 0xaa]];
    let annex_classification = inspect_taproot_witness(&key_path_with_annex)
        .expect("annex is identified by the last element's 0x50 prefix");
    match annex_classification {
        TaprootWitnessClassification::KeyPath { annex, .. } => {
            assert_eq!(annex.expect("annex is present").position, 1);
            assert_eq!(annex.expect("annex is present").size_bytes, 2);
        }
        TaprootWitnessClassification::ScriptPath { .. } => panic!("expected key path"),
    }

    let script_path = vec![
        vec![0x01],
        vec![0x51],
        control_block(0xc1, 2),
        vec![0x50, 0xbb],
    ];
    let classification =
        validate_taproot_witness(&script_path).expect("script path shape is valid");
    match classification {
        TaprootWitnessClassification::ScriptPath { classification } => {
            assert_eq!(classification.annex.expect("annex is present").position, 3);
            assert_eq!(classification.script_leaf.position, 1);
            assert_eq!(classification.script_leaf.size_bytes, 1);
            assert_eq!(classification.control_block.parity, TaprootParity::Odd);
            assert_eq!(classification.control_block.merkle_path_depth, 2);
            assert_eq!(classification.stack_argument_count, 1);
            assert_eq!(
                classification.leaf_version_support,
                TaprootLeafVersionSupport::CurrentTapscript
            );
            assert!(!classification.claim.cryptographic_verification_performed());
            assert!(!classification.claim.runtime_execution_performed());
        }
        TaprootWitnessClassification::KeyPath { .. } => panic!("expected script path"),
    }

    let empty = inspect_taproot_witness(&[]).expect_err("empty witness must fail closed");
    assert_eq!(empty.code, BitcoinBoundaryErrorCode::EmptyWitness);
}

#[test]
fn future_script_path_leaf_is_inspectable_but_validation_is_downstream_owned() {
    let witness = vec![vec![0x01], vec![0x51], control_block(0xe1, 0)];
    let inspected = inspect_taproot_witness(&witness).expect("future leaf shape is inspectable");
    match inspected {
        TaprootWitnessClassification::ScriptPath { classification } => assert_eq!(
            classification.leaf_version_support,
            TaprootLeafVersionSupport::FutureOrUnknown { leaf_version: 0xe0 }
        ),
        TaprootWitnessClassification::KeyPath { .. } => panic!("expected script path"),
    }

    let error = validate_taproot_witness(&witness).expect_err("future leaf must fail closed");
    assert_eq!(
        error.category,
        BitcoinBoundaryErrorCategory::DownstreamOwned
    );
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::UnknownTaprootLeafVersion
    );
}

fn threshold_metadata() -> MiniscriptPolicyMetadata {
    MiniscriptPolicyMetadata {
        policy_kind: MiniscriptPolicyKind::Threshold,
        required_signatures: 2,
        candidate_signers: 3,
        max_satisfaction_elements: 2,
        uses_timelock: false,
        uses_hashlock: false,
        uses_checksigadd: true,
    }
}

fn single_key_metadata() -> MiniscriptPolicyMetadata {
    MiniscriptPolicyMetadata {
        policy_kind: MiniscriptPolicyKind::SingleKey,
        required_signatures: 1,
        candidate_signers: 1,
        max_satisfaction_elements: 1,
        uses_timelock: false,
        uses_hashlock: false,
        uses_checksigadd: false,
    }
}

fn timelock_metadata() -> MiniscriptPolicyMetadata {
    MiniscriptPolicyMetadata {
        policy_kind: MiniscriptPolicyKind::Timelock,
        required_signatures: 0,
        candidate_signers: 0,
        max_satisfaction_elements: 0,
        uses_timelock: true,
        uses_hashlock: false,
        uses_checksigadd: false,
    }
}

fn assert_invalid_miniscript_metadata_error(error: BitcoinBoundaryError) {
    assert_eq!(error.category, BitcoinBoundaryErrorCategory::Malformed);
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata
    );
    assert_eq!(error.category_code(), "malformed");
    assert_eq!(error.code_str(), "invalid_miniscript_metadata");
    assert_eq!(error.to_string(), "malformed: invalid_miniscript_metadata");
    assert_eq!(
        serde_json::to_string(&error).expect("metadata error serializes"),
        r#"{"category":"malformed","code":"invalid_miniscript_metadata"}"#
    );
}

#[test]
fn miniscript_metadata_handoff_accepts_public_threshold_constraints_only() {
    let handoff = MiniscriptHandoff {
        api_version: MINISCRIPT_HANDOFF_API_VERSION,
        context: MiniscriptContext::TaprootScriptPath,
        metadata: threshold_metadata(),
        requested_capabilities: vec![
            MiniscriptCapability::StaticMetadata,
            MiniscriptCapability::StructuralHandoff,
        ],
    };

    let assessment = validate_miniscript_handoff(&handoff).expect("core handoff is supported");
    assert_eq!(
        assessment.accepted_capabilities,
        handoff.requested_capabilities
    );
    assert_eq!(
        assessment.downstream_owned_capabilities,
        vec![
            MiniscriptCapability::Compilation,
            MiniscriptCapability::Satisfaction,
            MiniscriptCapability::Execution,
            MiniscriptCapability::CryptographicVerification,
        ]
    );
    assert_eq!(assessment.claim, ValidationClaim::StructuralOnly);
    assert!(!assessment.claim.cryptographic_verification_performed());
    assert!(!assessment.claim.runtime_execution_performed());

    let encoded = serde_json::to_string(&handoff).expect("handoff serializes");
    let decoded: MiniscriptHandoff = serde_json::from_str(&encoded).expect("handoff round trips");
    assert_eq!(decoded, handoff);
}

#[test]
fn miniscript_metadata_rejects_inconsistent_constraints_and_downstream_capabilities() {
    let mut invalid = threshold_metadata();
    invalid.required_signatures = 4;
    let error =
        validate_miniscript_policy_metadata(&MiniscriptContext::TaprootScriptPath, &invalid)
            .expect_err("required signatures cannot exceed candidate signers");
    assert_eq!(error.category, BitcoinBoundaryErrorCategory::Malformed);
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::InvalidMiniscriptMetadata
    );

    let mut context_mismatch = threshold_metadata();
    context_mismatch.uses_checksigadd = true;
    let error =
        validate_miniscript_policy_metadata(&MiniscriptContext::SegwitV0, &context_mismatch)
            .expect_err("CHECKSIGADD is Tapscript-context metadata");
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::MiniscriptContextMismatch
    );

    let downstream_handoff = MiniscriptHandoff {
        api_version: MINISCRIPT_HANDOFF_API_VERSION,
        context: MiniscriptContext::TaprootScriptPath,
        metadata: threshold_metadata(),
        requested_capabilities: vec![
            MiniscriptCapability::StaticMetadata,
            MiniscriptCapability::StructuralHandoff,
            MiniscriptCapability::Compilation,
        ],
    };
    let error = validate_miniscript_handoff(&downstream_handoff)
        .expect_err("compilation remains downstream-owned");
    assert_eq!(
        error.category,
        BitcoinBoundaryErrorCategory::DownstreamOwned
    );
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::DownstreamOwnedMiniscriptCapability
    );

    let unsupported_context = MiniscriptHandoff {
        api_version: MINISCRIPT_HANDOFF_API_VERSION,
        context: MiniscriptContext::Other("future_context".to_string()),
        metadata: threshold_metadata(),
        requested_capabilities: vec![
            MiniscriptCapability::StaticMetadata,
            MiniscriptCapability::StructuralHandoff,
        ],
    };
    let error = validate_miniscript_handoff(&unsupported_context)
        .expect_err("unknown context must be downstream-owned");
    assert_eq!(
        error.category,
        BitcoinBoundaryErrorCategory::DownstreamOwned
    );
    assert_eq!(
        error.code,
        BitcoinBoundaryErrorCode::UnsupportedMiniscriptContext
    );
}

#[test]
fn miniscript_metadata_rejects_undersized_satisfaction_bounds_for_threshold_and_single_key() {
    let mut threshold = threshold_metadata();
    validate_miniscript_policy_metadata(&MiniscriptContext::TaprootScriptPath, &threshold)
        .expect("threshold metadata with a sufficient bound remains valid");
    threshold.max_satisfaction_elements = 1;
    let error =
        validate_miniscript_policy_metadata(&MiniscriptContext::TaprootScriptPath, &threshold)
            .expect_err("threshold bound cannot be smaller than required signatures");
    assert_invalid_miniscript_metadata_error(error);

    let mut single_key = single_key_metadata();
    validate_miniscript_policy_metadata(&MiniscriptContext::TaprootScriptPath, &single_key)
        .expect("single-key metadata with a sufficient bound remains valid");
    single_key.max_satisfaction_elements = 0;
    let error =
        validate_miniscript_policy_metadata(&MiniscriptContext::TaprootScriptPath, &single_key)
            .expect_err("single-key bound cannot be smaller than required signatures");
    assert_invalid_miniscript_metadata_error(error);
}

#[test]
fn miniscript_timelock_metadata_requires_zero_signers_and_satisfaction_elements() {
    let mut nonzero_required_signatures = timelock_metadata();
    nonzero_required_signatures.required_signatures = 1;
    nonzero_required_signatures.candidate_signers = 1;
    nonzero_required_signatures.max_satisfaction_elements = 1;
    let error = validate_miniscript_policy_metadata(
        &MiniscriptContext::SegwitV0,
        &nonzero_required_signatures,
    )
    .expect_err("timelock-only metadata cannot require signatures");
    assert_invalid_miniscript_metadata_error(error);

    let mut nonzero_candidate_signers = timelock_metadata();
    nonzero_candidate_signers.candidate_signers = 1;
    let error = validate_miniscript_policy_metadata(
        &MiniscriptContext::SegwitV0,
        &nonzero_candidate_signers,
    )
    .expect_err("timelock-only metadata cannot advertise candidate signers");
    assert_invalid_miniscript_metadata_error(error);

    let mut nonzero_satisfaction_elements = timelock_metadata();
    nonzero_satisfaction_elements.max_satisfaction_elements = 1;
    let error = validate_miniscript_policy_metadata(
        &MiniscriptContext::SegwitV0,
        &nonzero_satisfaction_elements,
    )
    .expect_err("timelock-only metadata cannot require satisfaction elements");
    assert_invalid_miniscript_metadata_error(error);

    validate_miniscript_policy_metadata(&MiniscriptContext::SegwitV0, &timelock_metadata())
        .expect("zero-signer, zero-element timelock-only metadata is valid");
}

#[test]
fn boundary_errors_and_shared_types_have_stable_serde_shapes() {
    let error = BitcoinBoundaryError {
        category: BitcoinBoundaryErrorCategory::Malformed,
        code: BitcoinBoundaryErrorCode::KeyPathSignatureZeroSighash,
    };
    let error_json = serde_json::to_string(&error).expect("error serializes");
    assert_eq!(
        error_json,
        r#"{"category":"malformed","code":"key_path_signature_zero_sighash"}"#
    );
    let decoded_error: lib_conxian_core::bitcoin::taproot::BitcoinBoundaryError =
        serde_json::from_str(&error_json).expect("error round trips");
    assert_eq!(decoded_error, error);

    let claim_json =
        serde_json::to_string(&ValidationClaim::StructuralOnly).expect("claim serializes");
    assert_eq!(claim_json, r#""structural_only""#);

    let policy = timelock_metadata();
    validate_miniscript_policy_metadata(&MiniscriptContext::SegwitV0, &policy)
        .expect("timelock-only metadata is structurally valid");
    let policy_json = serde_json::to_string(&policy).expect("policy metadata serializes");
    let decoded_policy: MiniscriptPolicyMetadata =
        serde_json::from_str(&policy_json).expect("policy metadata round trips");
    assert_eq!(decoded_policy, policy);
}
