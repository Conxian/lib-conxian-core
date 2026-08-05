//! # SDK Capability Re-exports (Session 52 → Session 57)
//!
//! Comprehensive re-exports of all publicly-accessible conxius-enclave-sdk modules,
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
//! ## Module Map (57 public modules → 6 categories; Session 57 ground-truth from v2.0.12)
//!
//! | Category | Feature | Modules | Status |
//! |----------|---------|---------|:------:|
//! | Blockchain | `sdk-blockchain` | ark, asset, bip110, bip322, bitcoin, bitvm, bitvm2, cctp, covenant, credit, dlc, ethereum, fiat, frost, lightning, mmr, musig2, sidl, solana, stacks, swap_router | ✅ |
//! | Cross-cutting | `sdk-cross-cutting` | a2p, account_abstraction, business, chain_abstraction, control_model_adapter, economy, identity, intent, job_card, opportunity, settlement, settlement_service, solver, stablecoin_orchestrator, zkml | ✅ |
//! | Nexus | `sdk-nexus` | nexus::fedimint | ✅ |
//! | Infrastructure | `sdk-infrastructure` | config, state, telemetry, wasm_support | ✅ |
//! | Enclave | `enclave` | android_authorization, android_strongbox, attestation, cloud, durable_replay, nitro, proof, proofs, replay_guard, trust, trust_contracts | ✅ |
//!
//! **Not re-exported:**
//! - Rails modules (`bisq`, `boltz`, `changelly`, `ntt`, `wormhole`, `x402`): `pub(crate)` in SDK — internal only
//! - `frost_crypto`, `bip110_compliant`: behind SDK feature gates
//! - `babylon`, `rgb`, `statechain`, `roast`, `signing/*`, `serde_big_array`: added to SDK main after v2.0.12 (pending v2.0.13 release)
//! - `wasm_bindings`: `cfg(wasm32)` gated
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
    feature = "sdk-nexus",
    feature = "sdk-infrastructure",
))]
pub use conxius_enclave_sdk;

// ── Convenience re-exports organized by category ──
// These mirror the actual SDK module structure as confirmed by CI build.

#[cfg(feature = "sdk-blockchain")]
pub mod blockchain {
    pub use conxius_enclave_sdk::protocol::ark;
    pub use conxius_enclave_sdk::protocol::asset;
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
    // v2.0.13+ (on main, not yet in any release):
    // pub use conxius_enclave_sdk::protocol::babylon;
    // pub use conxius_enclave_sdk::protocol::rgb;
    // pub use conxius_enclave_sdk::protocol::statechain;
    // Feature-gated in SDK: cfg(frost-crypto) and cfg(bip110_compliant)
    // pub use conxius_enclave_sdk::protocol::frost_crypto;
    // pub use conxius_enclave_sdk::protocol::bip110;
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

// Rails modules are `pub(crate)` in SDK v2.0.12 — cannot re-export.
// Revisit after SDK v2.0.13 makes them public.
//
// #[cfg(feature = "sdk-rails")]
// pub mod rails { ... }

// Nexus modules are nested under protocol::nexus::
#[cfg(feature = "sdk-nexus")]
pub mod nexus {
    pub use conxius_enclave_sdk::protocol::nexus::fedimint;
    // v2.0.13+ (on main, not yet in any release):
    // pub use conxius_enclave_sdk::protocol::nexus::roast;
}

#[cfg(feature = "sdk-infrastructure")]
pub mod infrastructure {
    pub use conxius_enclave_sdk::config;
    pub use conxius_enclave_sdk::state;
    pub use conxius_enclave_sdk::telemetry;
    pub use conxius_enclave_sdk::wasm_support;
    // cfg(wasm32) gated:
    // pub use conxius_enclave_sdk::wasm_bindings;
    // v2.0.13+ (on main, not yet in any release):
    // pub use conxius_enclave_sdk::serde_big_array;
}

// ── Signing module (Session 57 — does not exist in SDK v2.0.12) ──
//
// The `signing/` crate was added to SDK main after v2.0.12.
// Will be re-exported here after v2.0.13 release:
//
// #[cfg(feature = "sdk-signing")]
// pub mod signing {
//     pub use conxius_enclave_sdk::signing::musig2_signing;
//     pub use conxius_enclave_sdk::signing::bip322_signing;
//     pub use conxius_enclave_sdk::signing::taproot;
//     pub use conxius_enclave_sdk::signing::ucs;
//     ...
// }

// ── Enclave module (re-exported at crate root for backward compat) ──

#[cfg(feature = "enclave")]
pub mod enclave_sdk {
    pub use conxius_enclave_sdk::enclave::android_authorization;
    // Gated behind `development-simulators` in SDK:
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
