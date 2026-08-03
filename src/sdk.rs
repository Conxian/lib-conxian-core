//! # SDK Capability Re-exports (Session 52)
//!
//! Comprehensive re-exports of all 50 conxius-enclave-sdk modules,
//! organized by category. Each category is gated behind a feature flag.
//!
//! Enable via Cargo.toml:
//! ```toml
//! lib-conxian-core = { version = "0.3", features = ["full-sdk"] }
//! ```
//!
//! Or enable individual categories:
//! ```toml
//! lib-conxian-core = { version = "0.3", features = ["sdk-blockchain", "sdk-cross-cutting"] }
//! ```
//!
//! ## Module Map (50 modules → 5 categories + enclave)
//!
//! | Category | Feature | Modules | Status |
//! |----------|---------|---------|:------:|
//! | Blockchain | `sdk-blockchain` | bitcoin, bip322, bitvm, bitvm2, dlc, frost, lightning, musig2, stacks, covenant, ark, cctp, mmr, ethereum, solana, statechain, sidl, credit, fiat, asset, bip110, frost_crypto | ✅ |
//! | Cross-cutting | `sdk-cross-cutting` | intent, settlement, settlement_service, swap_router, stablecoin_orchestrator, solver, chain_abstraction, account_abstraction, a2p, control_model_adapter, identity, economy, job_card, business, opportunity, zkml | ✅ |
//! | Rails | `sdk-rails` | rails::{bisq, boltz, changelly, wormhole, ntt, x402} | ✅ |
//! | Nexus | `sdk-nexus` | nexus::{fedimint, roast} | ✅ |
//! | Infrastructure | `sdk-infrastructure` | config, state, telemetry, wasm_bindings | ✅ |
//! | Enclave | `enclave` | attestation, android_strongbox, cloud, durable_replay, nitro, proof, proofs, replay_guard, trust, trust_contracts | ✅ |
//!
//! **Note:** SDK modules behind feature gates (frost_crypto, bip110, nitro, etc.)
//! are only available when those SDK features are active. Consumers should enable
//! the corresponding crate features on `conxius-enclave-sdk` if needed.

// ── Full SDK re-export (always available when any sdk feature is enabled) ──

/// Re-export the entire conxius-enclave-sdk crate.
/// Access via `conxian_core::sdk::conxius_enclave_sdk::protocol::bitcoin` etc.
#[cfg(any(
    feature = "enclave",
    feature = "sdk-blockchain",
    feature = "sdk-cross-cutting",
    feature = "sdk-rails",
    feature = "sdk-nexus",
    feature = "sdk-infrastructure",
))]
pub use conxius_enclave_sdk;

// ── Convenience re-exports organized by category ──
// These mirror the actual SDK module structure as confirmed by CI build.

#[cfg(feature = "sdk-blockchain")]
pub mod blockchain {
    pub use conxius_enclave_sdk::protocol::bitcoin;
    pub use conxius_enclave_sdk::protocol::bip322;
    pub use conxius_enclave_sdk::protocol::bitvm;
    pub use conxius_enclave_sdk::protocol::bitvm2;
    pub use conxius_enclave_sdk::protocol::dlc;
    pub use conxius_enclave_sdk::protocol::frost;
    pub use conxius_enclave_sdk::protocol::lightning;
    pub use conxius_enclave_sdk::protocol::musig2;
    pub use conxius_enclave_sdk::protocol::stacks;
    pub use conxius_enclave_sdk::protocol::covenant;
    pub use conxius_enclave_sdk::protocol::ark;
    pub use conxius_enclave_sdk::protocol::cctp;
    pub use conxius_enclave_sdk::protocol::mmr;
    pub use conxius_enclave_sdk::protocol::ethereum;
    pub use conxius_enclave_sdk::protocol::solana;
    // v2.0.12+: pub use conxius_enclave_sdk::protocol::statechain;
    pub use conxius_enclave_sdk::protocol::sidl;
    pub use conxius_enclave_sdk::protocol::credit;
    pub use conxius_enclave_sdk::protocol::fiat;
    pub use conxius_enclave_sdk::protocol::asset;
    // Feature-gated in SDK: cfg(frost-crypto) and cfg(bip110_compliant)
    // pub use conxius_enclave_sdk::protocol::frost_crypto;
    // pub use conxius_enclave_sdk::protocol::bip110;
}

#[cfg(feature = "sdk-cross-cutting")]
pub mod cross_cutting {
    pub use conxius_enclave_sdk::protocol::intent;
    pub use conxius_enclave_sdk::protocol::settlement;
    pub use conxius_enclave_sdk::protocol::settlement_service;
    pub use conxius_enclave_sdk::protocol::swap_router;
    pub use conxius_enclave_sdk::protocol::stablecoin_orchestrator;
    pub use conxius_enclave_sdk::protocol::solver;
    pub use conxius_enclave_sdk::protocol::chain_abstraction;
    pub use conxius_enclave_sdk::protocol::account_abstraction;
    pub use conxius_enclave_sdk::protocol::a2p;
    // v2.0.12+: pub use conxius_enclave_sdk::protocol::control_model_adapter;
    pub use conxius_enclave_sdk::protocol::identity;
    pub use conxius_enclave_sdk::protocol::economy;
    pub use conxius_enclave_sdk::protocol::job_card;
    pub use conxius_enclave_sdk::protocol::business;
    pub use conxius_enclave_sdk::protocol::opportunity;
    pub use conxius_enclave_sdk::protocol::zkml;
}

// Rails modules are nested under protocol::rails::
#[cfg(feature = "sdk-rails")]
pub mod rails {
    pub use conxius_enclave_sdk::protocol::rails::bisq;
    pub use conxius_enclave_sdk::protocol::rails::boltz;
    pub use conxius_enclave_sdk::protocol::rails::changelly;
    pub use conxius_enclave_sdk::protocol::rails::wormhole;
    pub use conxius_enclave_sdk::protocol::rails::ntt;
    pub use conxius_enclave_sdk::protocol::rails::x402;
}

// Nexus modules are nested under protocol::nexus::
#[cfg(feature = "sdk-nexus")]
pub mod nexus {
    pub use conxius_enclave_sdk::protocol::nexus::fedimint;
    pub use conxius_enclave_sdk::protocol::nexus::roast;
}

#[cfg(feature = "sdk-infrastructure")]
pub mod infrastructure {
    pub use conxius_enclave_sdk::config;
    pub use conxius_enclave_sdk::state;
    pub use conxius_enclave_sdk::telemetry;
    pub use conxius_enclave_sdk::wasm_bindings;
}

// ── Enclave module (re-exported at crate root for backward compat) ──

#[cfg(feature = "enclave")]
pub mod enclave_sdk {
    pub use conxius_enclave_sdk::enclave::attestation;
    pub use conxius_enclave_sdk::enclave::android_strongbox;
    pub use conxius_enclave_sdk::enclave::cloud;
    // v2.0.12+: pub use conxius_enclave_sdk::enclave::durable_replay;
    // Feature-gated in SDK: cfg(not(wasm32))
    // pub use conxius_enclave_sdk::enclave::nitro;
    pub use conxius_enclave_sdk::enclave::replay_guard;
    // v2.0.12+ or re-exported at enclave module level only:
    // pub use conxius_enclave_sdk::enclave::proof;
    // pub use conxius_enclave_sdk::enclave::proofs;
    // pub use conxius_enclave_sdk::enclave::trust;
    // pub use conxius_enclave_sdk::enclave::trust_contracts;
}
