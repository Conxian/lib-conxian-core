//! Cross-repository integration contract tests.
//!
//! Validates that core types stay compatible with interfaces expected by
//! conxian-gateway, conxian-nexus, and conxius-enclave-sdk consumers.

use chrono::Utc;
use lib_conxian_core::control_model::{
    Bip110Compliance, Bip110Limits, Chain, ChainFamily, FinalityClass, TrustTier,
    VerificationClass, VerificationStatus,
};
use lib_conxian_core::signing::{
    AddressFormat, ChainSigningCapability, SignRequest, SignResponse, SignerCapabilities,
    SigningAlgorithm, SigningOperation, SigningTarget,
};
use lib_conxian_core::verifier::{
    BlockHeader, ChainId, LatestVerifiedBlock, TransactionFinalityResult,
    TransactionFinalityStatus, VerificationProvenance,
};

// ── Gateway contracts ──

#[test]
fn gateway_bip110_limit_canonicals_are_stable() {
    let limits = Bip110Limits::canonical();
    assert_eq!(limits.max_pushdata_bytes, 256);
    assert_eq!(limits.max_op_return_bytes, 83);
    assert_eq!(limits.max_script_pubkey_bytes, 34);
    assert_eq!(limits.max_witness_element_bytes, 256);
}

#[test]
fn gateway_bip110_compliance_toggle() {
    assert!(Bip110Compliance::new().is_enabled());
    assert!(!Bip110Compliance::disabled().is_enabled());
}

#[test]
fn gateway_trust_tier_serde_round_trip() {
    let tiers = [
        TrustTier::Strict,
        TrustTier::Managed,
        TrustTier::Expedient,
        TrustTier::ObserverOnly,
    ];
    for tier in &tiers {
        let json = serde_json::to_value(tier).expect("TrustTier serializes");
        let decoded: TrustTier = serde_json::from_value(json).expect("TrustTier deserializes");
        assert_eq!(decoded, *tier);
    }
}

// ── Nexus contracts ──

#[test]
fn nexus_sign_request_json_round_trip() {
    let json = serde_json::json!({
        "target": {"chain": "bitcoin", "family": "bitcoin_utxo"},
        "algorithm": "ecdsa_secp256k1",
        "payload": {"message": {"bytes": [1, 2, 3, 4]}},
        "derivation": {
            "path": {"components": [{"index": 84, "hardened": true}]},
            "purpose": "payment"
        }
    });
    let req: SignRequest = serde_json::from_value(json).expect("SignRequest deserializes");
    assert_eq!(req.algorithm, SigningAlgorithm::EcdsaSecp256k1);
    let back: serde_json::Value = serde_json::to_value(&req).expect("SignRequest serializes");
    assert_eq!(back["algorithm"], "ecdsa_secp256k1");
}

#[test]
fn nexus_sign_response_json_round_trip() {
    let json = serde_json::json!({
        "signature": {
            "algorithm": "ecdsa_secp256k1",
            "encoding": "der",
            "bytes": [170, 170]
        },
        "verification_key": {
            "algorithm": "ecdsa_secp256k1",
            "bytes": [2, 3]
        },
        "address": {
            "chain": "bitcoin",
            "format": "bitcoin_bech32",
            "value": "bc1qtest"
        },
        "derivation": {
            "path": {"components": []},
            "purpose": "payment"
        }
    });
    let resp: SignResponse = serde_json::from_value(json).expect("SignResponse deserializes");
    assert_eq!(resp.address.value, "bc1qtest");

    let back: serde_json::Value = serde_json::to_value(&resp).expect("SignResponse serializes");
    assert_eq!(back["address"]["value"], "bc1qtest");
}

#[test]
fn nexus_signer_capabilities_serde_round_trip() {
    let caps = SignerCapabilities::new(
        1,
        vec![ChainSigningCapability::new(
            SigningTarget::new(Chain::Bitcoin, ChainFamily::BitcoinUtxo),
            vec![SigningAlgorithm::EcdsaSecp256k1],
            vec![SigningOperation::SignMessage],
            vec![AddressFormat::BitcoinBech32],
        )],
    );
    let json = serde_json::to_value(&caps).expect("SignerCapabilities serializes");
    let decoded: SignerCapabilities =
        serde_json::from_value(json).expect("SignerCapabilities deserializes");
    assert_eq!(decoded.api_version, 1);
    assert_eq!(decoded.chains.len(), 1);
}

// ── Verifier contracts ──

#[test]
fn verifier_chain_id_serde_round_trip() {
    let cid = ChainId::new(ChainFamily::BitcoinUtxo, "mainnet");
    let json = serde_json::to_value(&cid).expect("ChainId serializes");
    let decoded: ChainId = serde_json::from_value(json).expect("ChainId deserializes");
    assert_eq!(decoded.to_string(), "bitcoin_utxo:mainnet");
}

#[test]
fn verifier_finality_status_exhaustive_and_serde_round_trip() {
    let all = [
        TransactionFinalityStatus::Pending,
        TransactionFinalityStatus::Confirmed { confirmations: 1 },
        TransactionFinalityStatus::Finalized { confirmations: 6 },
        TransactionFinalityStatus::Reorged,
        TransactionFinalityStatus::Rejected,
    ];
    for status in &all {
        let json = serde_json::to_value(status).expect("finality status serializes");
        let decoded: TransactionFinalityStatus =
            serde_json::from_value(json).expect("finality status deserializes");
        assert_eq!(decoded, *status);
    }
}

#[test]
fn verifier_finality_result_serde_round_trip() {
    let chain = ChainId::new(ChainFamily::BitcoinUtxo, "mainnet");
    let result = TransactionFinalityResult {
        chain: chain.clone(),
        transaction_id: "tx-finality-1".to_string(),
        status: TransactionFinalityStatus::Confirmed { confirmations: 3 },
        finality_class: FinalityClass::Economic,
        required_confirmations: 6,
        observed_confirmations: 3,
        latest_block: Some(LatestVerifiedBlock {
            chain: chain.clone(),
            header: BlockHeader {
                hash: "abc123".to_string(),
                parent_hash: Some("parent123".to_string()),
                height: 850_000,
                timestamp: Utc::now(),
                state_root: Some("root123".to_string()),
            },
            finality_class: FinalityClass::Economic,
            confirmations: 3,
            verification_class: VerificationClass::NativeObservation,
            trust_tier: TrustTier::Managed,
            verification_status: VerificationStatus::Verified,
            provenance: VerificationProvenance {
                verifier_id: "test-verifier".to_string(),
                evidence_ref: Some("proof-1".to_string()),
                verified_at: Utc::now(),
            },
        }),
        verification_class: VerificationClass::NativeObservation,
        trust_tier: TrustTier::Managed,
        verification_status: VerificationStatus::Verified,
        provenance: VerificationProvenance {
            verifier_id: "test-verifier".to_string(),
            evidence_ref: None,
            verified_at: Utc::now(),
        },
    };

    let json = serde_json::to_value(&result).expect("TransactionFinalityResult serializes");
    let decoded: TransactionFinalityResult =
        serde_json::from_value(json).expect("TransactionFinalityResult deserializes");

    assert_eq!(decoded.transaction_id, "tx-finality-1");
    assert_eq!(decoded.observed_confirmations, 3);
    assert_eq!(decoded.required_confirmations, 6);
    assert_eq!(
        decoded.status,
        TransactionFinalityStatus::Confirmed { confirmations: 3 }
    );
    assert!(decoded.latest_block.is_some());
}
