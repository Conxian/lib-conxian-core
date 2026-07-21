mod support;

use lib_conxian_core::adapters::{
    BitcoinAdapter, StateProofError, TxParams, UniversalChainAdapter,
};
use lib_conxian_core::babylon::{BabylonAdapter, StakingIntent};
use lib_conxian_core::bitcoin::liquid_adapter::LiquidAdapter;
use lib_conxian_core::protocol::dlc::{DlcIntent, DlcManager};
use lib_conxian_core::rgb::{RGBError, RGBExecutionMode, RGBRuntime, RGBSkeletonAdapter};
use lib_conxian_core::stacks::{SBTCBridge, SBTCIntent, StacksAdapter, StacksError};
use serde::de::DeserializeOwned;
use serde_json::Value;
use support::{assert_json_structural_round_trip, load_fixture_value};

fn bytes(value: &Value) -> Vec<u8> {
    serde_json::from_value(value.clone()).expect("fixture bytes are numeric arrays")
}

fn expected<T>(case: &Value, key: &str) -> T
where
    T: DeserializeOwned,
{
    serde_json::from_value(case["expected"][key].clone())
        .unwrap_or_else(|error| panic!("adapter fixture expected.{key} is invalid: {error}"))
}

fn assert_stacks_error(result: Result<SBTCIntent, StacksError>, expected_code: &str) {
    match expected_code {
        "invalid_transaction" => assert!(matches!(result, Err(StacksError::InvalidTransaction))),
        "invalid_address" => assert!(matches!(result, Err(StacksError::InvalidAddress))),
        other => panic!("unknown Stacks error expectation {other}"),
    }
}

#[test]
fn representative_adapter_dtos_round_trip_and_remain_structural_only() {
    let fixture = load_fixture_value("adapter_cases.json");
    assert_eq!(fixture["schema_version"], Value::from(1));
    assert_eq!(fixture["contract"], "adapter_structural_conformance");
    assert_eq!(fixture["contract_version"], Value::from(1));

    for case in fixture["cases"].as_array().expect("adapter cases") {
        match case["kind"].as_str().expect("adapter case kind") {
            "bitcoin_tx_params" => {
                let params: TxParams =
                    serde_json::from_value(case["value"].clone()).expect("TxParams fixture");
                assert_json_structural_round_trip(&params);
                let adapter = BitcoinAdapter;
                assert_eq!(adapter.family(), expected(case, "family"));
                assert_eq!(adapter.chain(), expected(case, "chain"));
                assert_eq!(adapter.trust_tier(), expected(case, "trust_tier"));
                assert!(adapter.validate_address("bc1qfixture").is_ok());
                let expected_fee: u64 = expected(case, "fee");
                assert_eq!(
                    adapter.estimate_fee(&params).expect("fixture fee estimate"),
                    expected_fee
                );
            }
            "babylon_staking_intent" => {
                let intent: StakingIntent =
                    serde_json::from_value(case["value"].clone()).expect("Babylon intent");
                assert_json_structural_round_trip(&intent);
                let adapter = BabylonAdapter;
                assert_eq!(adapter.family(), expected(case, "family"));
                assert_eq!(adapter.chain(), expected(case, "chain"));
                assert_eq!(adapter.trust_tier(), expected(case, "trust_tier"));
                assert!(adapter.validate_address("bc1qfixture").is_ok());
                let proof: String = expected(case, "proof");
                match adapter.verify_state_proof("fixture-root", &proof) {
                    Err(StateProofError::Unsupported { chain }) => assert_eq!(chain, "babylon"),
                    other => {
                        panic!("expected Babylon state proof to be unsupported, got {other:?}")
                    }
                }
            }
            "liquid_proof" => {
                let adapter = LiquidAdapter;
                assert_eq!(adapter.family(), expected(case, "family"));
                assert_eq!(adapter.chain(), expected(case, "chain"));
                assert_eq!(adapter.trust_tier(), expected(case, "trust_tier"));
                assert!(adapter
                    .validate_address(case["address"].as_str().expect("Liquid address"))
                    .is_ok());
                match adapter.verify_state_proof(
                    "fixture-root",
                    case["proof"].as_str().expect("Liquid proof"),
                ) {
                    Err(StateProofError::Unsupported { chain }) => assert_eq!(chain, "liquid"),
                    other => panic!("expected Liquid state proof to be unsupported, got {other:?}"),
                }
                assert!(adapter
                    .verify_state_proof(
                        "fixture-root",
                        case["malformed_proof"].as_str().expect("malformed proof")
                    )
                    .is_err());
            }
            "stacks_sbtc_intent" => {
                let intent: SBTCIntent =
                    serde_json::from_value(case["value"].clone()).expect("sBTC intent");
                assert_json_structural_round_trip(&intent);
                let expected_epoch: u64 = expected(case, "created_at_epoch");
                assert_eq!(intent.created_at_epoch, expected_epoch);

                let bridge = SBTCBridge::new();
                let invalid_peg_in: String = expected(case, "invalid_peg_in");
                assert_stacks_error(bridge.initiate_peg_in(1, ""), &invalid_peg_in);
                let invalid_peg_out: String = expected(case, "invalid_peg_out");
                assert_stacks_error(bridge.initiate_peg_out(1, ""), &invalid_peg_out);
                let generated = bridge
                    .initiate_peg_in(500_000, "fixture-btc-txid")
                    .expect("dummy sBTC peg-in intent");
                assert_eq!(generated.created_at_epoch, expected_epoch);
            }
            "rgb_rollout" => {
                for mode_case in case["cases"].as_array().expect("RGB rollout cases") {
                    let mode: RGBExecutionMode =
                        serde_json::from_value(mode_case["mode"].clone()).expect("RGB mode");
                    let runtime = RGBRuntime::new(mode, RGBSkeletonAdapter);
                    let result = runtime.validate_transition(
                        mode_case["transition_hex"]
                            .as_str()
                            .expect("RGB transition fixture"),
                    );
                    match mode_case["expected"].as_str().expect("RGB expectation") {
                        "gated_by_rollout_mode" => {
                            assert_eq!(result, Err(RGBError::GatedByRolloutMode));
                        }
                        "ok" => assert_eq!(result, Ok(true)),
                        "non_authoritative_shadow" => {
                            assert_eq!(result, Err(RGBError::NonAuthoritativeShadow));
                        }
                        "transition_validation_failed" => assert!(matches!(
                            result,
                            Err(RGBError::TransitionValidationFailed(_))
                        )),
                        other => panic!("unknown RGB expectation {other}"),
                    }
                }
            }
            "dlc_intent" => {
                let intent: DlcIntent =
                    serde_json::from_value(case["value"].clone()).expect("DLC intent");
                assert_json_structural_round_trip(&intent);
                let recreated = DlcManager::create_intent(
                    &intent.oracle_pubkey,
                    intent.collateral_sats,
                    intent.outcome_hash,
                    intent.expiry_block,
                );
                assert_eq!(
                    serde_json::to_value(recreated).expect("DLC intent serializes"),
                    case["value"]
                );

                let attestation = &case["attestation"];
                let valid = DlcManager::verify_oracle_attestation(
                    &intent.oracle_pubkey,
                    &bytes(&attestation["nonce_point"]),
                    &bytes(&attestation["outcome_msg"]),
                    &bytes(&attestation["signature_scalar"]),
                );
                assert_eq!(
                    valid,
                    case["expected_attestation"]
                        .as_bool()
                        .expect("DLC attestation expectation")
                );
            }
            other => panic!("unknown adapter fixture kind {other}"),
        }
    }
}
