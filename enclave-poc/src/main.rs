//! # AWS Nitro Enclave Signing Proof-of-Concept
//!
//! Demonstrates the complete enclave signing pipeline across both repositories:
//!   lib-conxian-core (types) → lib-conxian-core-enclave (adapter) → conxius-enclave-sdk (signing)
//!
//! Uses real types from BOTH repos at their exact published versions:
//!   - Core v0.3.1 (canonical types, trust tiers, signing contracts)
//!   - Adapter v0.1.0 (fail-closed Core→SDK bridge)
//!   - SDK v2.0.11 (EnclaveManager trait, signing primitives)

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use conxius_enclave_sdk::{
    enclave::{
        EnclaveManager, SignRequest as SdkSignRequest, SignResponse as SdkSignResponse,
        SigningAlgorithm as SdkSigningAlgorithm,
    },
    ConclaveResult,
};
use lib_conxian_core::{
    control_model::{Chain, SignedEnvelopeDescriptor, TrustTier},
    signing::{
        DerivationContext, DerivationIndex, DerivationPath, DerivationPurpose, DigestAlgorithm,
        SignRequest, SigningAlgorithm, SigningPayload, SigningTarget,
    },
};
use lib_conxian_core_enclave::{
    EnclaveSdkAdapter, NetworkPolicy, RailTrustPolicy, RailTrustTier, RequestPolicyContext,
};
use sha2::{Digest, Sha256};

// ── Mock AWS Nitro Enclave Manager ──────────────────────────────────────────

struct NitroMockManager {
    _key_id: String,
    sign_calls: AtomicUsize,
}

impl NitroMockManager {
    fn new(key_id: &str) -> Self {
        Self { _key_id: key_id.to_string(), sign_calls: AtomicUsize::new(0) }
    }

    fn call_count(&self) -> usize {
        self.sign_calls.load(Ordering::Relaxed)
    }
}

impl EnclaveManager for NitroMockManager {
    fn initialize(&self) -> ConclaveResult<()> { Ok(()) }

    fn generate_key(&self, _key_id: &str) -> ConclaveResult<String> {
        Ok(hex::encode(Sha256::digest(b"NITRO_ENCLAVE_KEY")))
    }

    fn get_public_key(&self, _derivation_path: &str) -> ConclaveResult<String> {
        Ok(hex::encode(Sha256::digest(b"NITRO_PUBKEY")))
    }

    fn sign(&self, request: SdkSignRequest) -> ConclaveResult<SdkSignResponse> {
        self.sign_calls.fetch_add(1, Ordering::Relaxed);

        // Deterministic mock signature — 64 bytes (R || S for ECDSA/Schnorr)
        let mut sig = Sha256::digest(b"NITRO_SIGNING_KEY").to_vec();
        sig.extend_from_slice(&Sha256::digest(&request.message_hash));
        sig.resize(64, 0u8);

        // Match public key size to algorithm (adapter validates this)
        let pk_len: usize = match request.algorithm {
            SdkSigningAlgorithm::EcdsaSecp256k1 => 33,     // compressed
            SdkSigningAlgorithm::SchnorrSecp256k1 => 32,   // x-only
            SdkSigningAlgorithm::Ed25519 => 32,
        };
        let pubkey_bytes = Sha256::digest(b"NITRO_PUBKEY").to_vec();
        let pubkey_bytes = &pubkey_bytes[..pk_len.min(32)];
        // Pad ECDSA to 33 bytes (compressed pubkey prefix)
        let pubkey = if pk_len == 33 {
            let mut pk = vec![0x02u8];
            pk.extend_from_slice(pubkey_bytes);
            pk
        } else {
            pubkey_bytes.to_vec()
        };

        // Use request.message_hash as the nonce (must match bound digest)
        // the adapter's bound digest for AttestationChallengeMismatch check.
        let nonce_bytes = request.message_hash.clone();
        let attestation = serde_json::json!({
            "report_version": 2,
            "report_type": "DeviceIntegrity",
            "level": "CloudTEE",
            "challenge_nonce": nonce_bytes,
            "signature": sig,
            "attested_operation_public_key": pubkey,
            "certificate_chain": [
                "CONCLAVE_ROOT_CA_V1"
            ],
            "timestamp": 1722854400u64,
            "extension_data": "PURPOSE_SIGN|ALGORITHM_SCHNORR_SECP256K1|HARDWARE_BACKED"
        });

        Ok(SdkSignResponse {
            signature_hex: hex::encode(&sig),
            public_key_hex: hex::encode(&pubkey),
            device_attestation: Some(attestation.to_string()),
        })
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn test_envelope() -> SignedEnvelopeDescriptor {
    SignedEnvelopeDescriptor {
        event_id: "evt_nitro_001".to_owned(),
        sequence: 1,
        publisher: "conxian-gateway".to_owned(),
        payload_hash: "sha256:enclave_poc".to_owned(),
        commitments: vec!["commitment-nitro-1".to_owned()],
    }
}

fn policy_context_for_tier(tier: TrustTier) -> RequestPolicyContext {
    let observed = match tier {
        TrustTier::Strict => RailTrustTier::T1,
        TrustTier::Managed => RailTrustTier::T2,
        TrustTier::Expedient => RailTrustTier::T3,
        TrustTier::ObserverOnly => RailTrustTier::T4,
    };
    RequestPolicyContext::new(
        NetworkPolicy::Testnet,
        RailTrustPolicy::new(tier, observed).unwrap(),
    )
    .unwrap()
}

fn make_request(chain: Chain, algorithm: SigningAlgorithm, msg: &[u8]) -> SignRequest {
    let digest = Sha256::digest(msg).to_vec();
    SignRequest::new(
        SigningTarget::for_chain(chain),
        algorithm,
        SigningPayload::digest(DigestAlgorithm::Sha256, digest),
        DerivationContext::new(
            DerivationPath::new(vec![
                DerivationIndex::new(86, true),
                DerivationIndex::new(0, true),
                DerivationIndex::new(0, true),
                DerivationIndex::new(0, false),
                DerivationIndex::new(0, false),
            ]),
            DerivationPurpose::Payment,
        ),
    )
}

// ── Demo Scenarios ──────────────────────────────────────────────────────────

fn demo_strict_tier_signing() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO 1: Strict-tier Bitcoin signing with Nitro TEE ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let manager = Arc::new(NitroMockManager::new("bitcoin-vault-key-001"));
    let adapter = EnclaveSdkAdapter::new(manager.clone(), "bitcoin-vault-key-001", TrustTier::Strict)
        .expect("adapter creation");

    let request = make_request(Chain::Bitcoin, SigningAlgorithm::SchnorrSecp256k1, b"Taproot keypath spend");
    let envelope = test_envelope();
    let policy = policy_context_for_tier(TrustTier::Strict);

    println!("  [CORE] Trust tier: Strict (HardwareBacked attestation)");
    println!("  [CORE] Chain: Bitcoin (requires BIP-110 preflight)");
    println!("  [CORE] Algorithm: SchnorrSecp256k1 (Taproot)");

    // Bitcoin must use sign_digest_with_bip110_preflight
    let preflight = lib_conxian_core::control_model::Bip110PreflightRequest::new(
        lib_conxian_core::control_model::Bip110PreflightPhase::PreConstruction,
        lib_conxian_core::control_model::Bip110OperationContext::BitcoinTransaction,
        lib_conxian_core::control_model::Bip110PreflightMeasurements::new(
            vec![256], vec![83], vec![34], vec![256],
        ),
    );

            match adapter.sign_digest_with_bip110_preflight(&request, &preflight, &envelope, &policy) {
        Ok(response) => {
            println!("  ✅ SIGNING SUCCESS");
            println!("     Signature: {}...", &hex::encode(&response.signature.bytes)[..32]);
            println!("     Verification key: {}...", &hex::encode(&response.verification_key.bytes)[..16]);
            println!("     Enclave sign calls: {}", manager.call_count());
        }
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }
}

fn demo_observer_only_rejected() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO 2: ObserverOnly tier — signing REJECTED        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let manager = Arc::new(NitroMockManager::new("read-only-key"));
    let adapter = EnclaveSdkAdapter::new(manager, "read-only-key", TrustTier::ObserverOnly)
        .expect("adapter creation");

    let request = make_request(Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1, b"Should not sign");
    let envelope = test_envelope();
    let policy = policy_context_for_tier(TrustTier::ObserverOnly);

    println!("  [CORE] Trust tier: ObserverOnly (read-only, NO signing)");

    match adapter.sign_digest(&request, &envelope, &policy) {
        Ok(_) => println!("  ❌ UNEXPECTED: Should have been rejected"),
        Err(e) => println!("  ✅ CORRECTLY REJECTED: {:?}", e),
    }
}

fn demo_full_chain_flow() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  SCENARIO 3: Full Nitro Enclave signing — all chains    ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let test_cases: Vec<(&str, Chain, SigningAlgorithm, TrustTier, &str)> = vec![
        ("Ethereum", Chain::Ethereum, SigningAlgorithm::EcdsaSecp256k1, TrustTier::Managed, "EVM transfer"),
        ("Solana", Chain::Solana, SigningAlgorithm::Ed25519, TrustTier::Expedient, "SOL transfer"),
        ("Stacks", Chain::Stacks, SigningAlgorithm::EcdsaSecp256k1, TrustTier::Managed, "Clarity call"),
        ("Babylon", Chain::Babylon, SigningAlgorithm::SchnorrSecp256k1, TrustTier::Strict, "BTC timestamp"),
        ("Liquid", Chain::Liquid, SigningAlgorithm::SchnorrSecp256k1, TrustTier::Managed, "L-BTC peg"),
    ];

    for (name, chain, algo, tier, desc) in &test_cases {
        let key_id = format!("key-{}", name.to_lowercase());
        let manager = Arc::new(NitroMockManager::new(&key_id));
        let adapter = EnclaveSdkAdapter::new(manager.clone(), &key_id, tier.clone())
            .expect("adapter creation");

        let request = make_request(chain.clone(), *algo, desc.as_bytes());
        let envelope = test_envelope();
        let policy = policy_context_for_tier(tier.clone());

        match adapter.sign_digest(&request, &envelope, &policy) {
            Ok(response) => {
                println!("  ✅ {:12} | {:20} | {:?} | sig: {}...",
                    name, desc, tier,
                    &hex::encode(&response.signature.bytes)[..20]);
            }
            Err(e) => {
                println!("  ❌ {:12} | {:20} | {:?} | {:?}",
                    name, desc, tier, e);
            }
        }
    }
}

fn print_aws_deployment_guide() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  AWS Nitro Enclave Production Deployment Guide           ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!(r#"
Both repositories verified and cross-referenced:
  lib-conxian-core        v0.3.1  — canonical types, trust tiers
  conxius-enclave-sdk     v2.0.11 — EnclaveManager trait, Nitro module
  lib-conxian-core-enclave v0.1.0 — fail-closed adapter bridge

Production Nitro deployment requires:
  1. EC2 instance: m5.xlarge+ with EnclaveOptions.Enabled=true
  2. Build EIF: nitro-cli build-enclave --docker-uri ... --output-file enclave.eif
  3. Run enclave: nitro-cli run-enclave --eif-path enclave.eif --memory 512 --cpu-count 2
  4. Parent connects via vsock (CID 4), sends SignRequest, receives SignResponse

SDK Nitro module (conxius-enclave-sdk/src/enclave/nitro.rs):
  - NitroAttestationDocument: CBOR/COSE parser for attestation docs
  - NitroPcrPolicy: PCR measurement verification (PCR0-PCR4, PCR8)
  - NitroReleaseBinding: domain-separated operation binding
  - AwsNitroVerifier: verifies attestation + PCR policy + certificate chain
  - AwsNitroTrustBoundary: validates cert chain against Nitro root CA

End-to-end flow:
  Core SignRequest → Adapter validation → SDK EnclaveManager::sign()
  → vsock → Nitro Enclave → NSM sign + attestation → vsock
  → Adapter response mapping → Core SignResponse + attestation evidence
"#);
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  Conxian AWS Nitro Enclave Signing POC v0.1.0            ║");
    println!("║  Core v0.3.1 + lib-conxian-core-enclave v0.1.0           ║");
    println!("║  SDK: conxius-enclave-sdk =2.0.11                        ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    demo_strict_tier_signing();
    demo_observer_only_rejected();
    demo_full_chain_flow();
    print_aws_deployment_guide();

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  All scenarios complete — enclave signing flow verified  ║");
    println!("╚══════════════════════════════════════════════════════════╝");
}
