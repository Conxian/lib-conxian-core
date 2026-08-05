//! # SDK Capability Re-exports
//!
//! Comprehensive re-exports of all publicly-accessible conxius-enclave-sdk modules,
//! organized by category. Each category is gated behind a feature flag.
//!
//! ## Module Map (Rust 1.94.1-compatible SDK v2.0.12)
//!
//! | Category | Feature | Modules | Count |
//! |----------|---------|---------|:-----:|
//! | Blockchain | `sdk-blockchain` | ark, asset, bip110, bip322, bitcoin, bitvm, bitvm2, cctp, covenant, credit, dlc, ethereum, fiat, frost, lightning, mmr, musig2, sidl, solana, stacks, swap_router | 21 |
//! | Cross-cutting | `sdk-cross-cutting` | a2p, account_abstraction, business, chain_abstraction, control_model_adapter, economy, identity, intent, job_card, opportunity, settlement, settlement_service, solver, stablecoin_orchestrator, zkml | 15 |
//! | Nexus | `sdk-nexus` | nexus::fedimint | 1 |
//! | Infrastructure | `sdk-infrastructure` | config, state, telemetry, wasm_support | 4 |
//! | Signing | `sdk-signing` | Reserved until the workspace adopts an SDK/Rust floor that exposes signing modules | 0 |
//! | Enclave | `enclave` | android_authorization, android_strongbox*, attestation, cloud*, durable_replay, nitro, proof, proofs, replay_guard, trust, trust_contracts | 11 |
//! | Rails | `sdk-rails` | (none — all `pub(crate)` in SDK) | 0 |
//!
//! \* = gated behind `development-simulators` in SDK
//!
//! **Blocked modules:**
//! - Rails (6): `pub(crate)` in SDK — cannot re-export
//! - `babylon`, `rgb`, `statechain`, `nexus::roast`, `serde_big_array`, and signing modules: introduced after SDK v2.0.12
//! - `frost_crypto`: `#[cfg(feature = "frost-crypto")]` in SDK
//! - `wasm_bindings`: `#[cfg(target_arch = "wasm32")]` in SDK
//! - `android_strongbox`, `cloud`: `#[cfg(any(test, feature = "development-simulators"))]` in SDK

// ── Full SDK re-export (always available when any sdk feature is enabled) ──

#[cfg(any(
    feature = "enclave",
    feature = "sdk-blockchain",
    feature = "sdk-cross-cutting",
    feature = "sdk-rails",
    feature = "sdk-nexus",
    feature = "sdk-infrastructure",
    feature = "sdk-signing",
))]
pub use conxius_enclave_sdk;

// ── Convenience re-exports organized by category ──

#[cfg(feature = "sdk-blockchain")]
pub mod blockchain {
    pub use conxius_enclave_sdk::protocol::ark;
    pub use conxius_enclave_sdk::protocol::asset;
    pub use conxius_enclave_sdk::protocol::bip110;
    pub use conxius_enclave_sdk::protocol::bip322;
    pub use conxius_enclave_sdk::protocol::bitcoin;
    pub use conxius_enclave_sdk::protocol::bitvm;
    pub use conxius_enclave_sdk::protocol::bitvm2;
    pub use conxius_enclave_sdk::protocol::cctp;
    pub use conxius_enclave_sdk::protocol::covenant;
    pub use conxius_enclave_sdk::protocol::credit;
    pub use conxius_enclave_sdk::protocol::dlc;
    pub use conxius_enclave_sdk::protocol::ethereum;
    pub use conxius_enclave_sdk::protocol::fiat;
    pub use conxius_enclave_sdk::protocol::frost;
    pub use conxius_enclave_sdk::protocol::lightning;
    pub use conxius_enclave_sdk::protocol::mmr;
    pub use conxius_enclave_sdk::protocol::musig2;
    pub use conxius_enclave_sdk::protocol::sidl;
    pub use conxius_enclave_sdk::protocol::solana;
    pub use conxius_enclave_sdk::protocol::stacks;
    pub use conxius_enclave_sdk::protocol::swap_router;
    // #[cfg(feature = "frost-crypto")] in SDK:
    // pub use conxius_enclave_sdk::protocol::frost_crypto;
}

#[cfg(feature = "sdk-cross-cutting")]
pub mod cross_cutting {
    pub use conxius_enclave_sdk::protocol::a2p;
    pub use conxius_enclave_sdk::protocol::account_abstraction;
    pub use conxius_enclave_sdk::protocol::business;
    pub use conxius_enclave_sdk::protocol::chain_abstraction;
    pub use conxius_enclave_sdk::protocol::control_model_adapter;
    pub use conxius_enclave_sdk::protocol::economy;
    pub use conxius_enclave_sdk::protocol::identity;
    pub use conxius_enclave_sdk::protocol::intent;
    pub use conxius_enclave_sdk::protocol::job_card;
    pub use conxius_enclave_sdk::protocol::opportunity;
    pub use conxius_enclave_sdk::protocol::settlement;
    pub use conxius_enclave_sdk::protocol::settlement_service;
    pub use conxius_enclave_sdk::protocol::solver;
    pub use conxius_enclave_sdk::protocol::stablecoin_orchestrator;
    pub use conxius_enclave_sdk::protocol::zkml;
}

// Rails modules are `pub(crate)` in SDK — cannot re-export.
// #[cfg(feature = "sdk-rails")]
// pub mod rails { ... }

#[cfg(feature = "sdk-nexus")]
pub mod nexus {
    pub use conxius_enclave_sdk::protocol::nexus::fedimint;
}

#[cfg(feature = "sdk-infrastructure")]
pub mod infrastructure {
    pub use conxius_enclave_sdk::config;
    pub use conxius_enclave_sdk::state;
    pub use conxius_enclave_sdk::telemetry;
    pub use conxius_enclave_sdk::wasm_support;
    // #[cfg(target_arch = "wasm32")] in SDK:
    // pub use conxius_enclave_sdk::wasm_bindings;
}

// ── Enclave module (re-exported at crate root for backward compat) ──

#[cfg(feature = "enclave")]
pub mod enclave_sdk {
    pub use conxius_enclave_sdk::enclave::android_authorization;
    // #[cfg(any(test, feature = "development-simulators"))] in SDK:
    // pub use conxius_enclave_sdk::enclave::android_strongbox;
    // pub use conxius_enclave_sdk::enclave::cloud;
    pub use conxius_enclave_sdk::enclave::attestation;
    pub use conxius_enclave_sdk::enclave::durable_replay;
    pub use conxius_enclave_sdk::enclave::nitro;
    pub use conxius_enclave_sdk::enclave::proof;
    pub use conxius_enclave_sdk::enclave::proofs;
    pub use conxius_enclave_sdk::enclave::replay_guard;
    pub use conxius_enclave_sdk::enclave::trust;
    pub use conxius_enclave_sdk::enclave::trust_contracts;
}
