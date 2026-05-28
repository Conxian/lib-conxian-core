//! # Audit Report: SDK Extraction Viability (CON-627)
//!
//! ## 1. Current Architecture Map
//! The core signing and policy logic is currently distributed across:
//! - `src/wallet.rs`: Basic secp256k1/k256 signing and key management.
//! - `src/sdk_primitive.rs`: High-level Vault SDK with policy enforcement (max amount, allowlist).
//! - `src/musig2.rs`: BIP327-compliant key aggregation.
//!
//! ## 2. Extraction Viability
//! ### 2.1 Hardware-Signing Isolation
//! The `Wallet` struct provides a clean interface for signing. It is currently backend-agnostic but prepared for StrongBox TEE integration.
//! - **Viability:** HIGH.
//!
//! ### 2.2 Policy Enforcement
//! The `SigningPolicy` and `VaultSDK` in `src/sdk_primitive.rs` are already decoupled from UI concerns.
//! - **Viability:** HIGH.
//!
//! ### 2.3 Chain Adapters
//! Chain-specific logic is isolated in `src/bitcoin/`, `src/stacks/`, and `src/lightning/`.
//! - **Viability:** MEDIUM. Some coupling exists in `gateway/src/engine/mod.rs` which should be moved to the SDK core if it's to be reusable.
//!
//! ## 3. Risks & Recommendations
//! - **Risk:** UI/UX logic is currently out of scope for this repository, which is good. However, the Gateway Engine (`gateway/src/engine/mod.rs`) contains significant orchestration logic that should be partially moved to the SDK to allow third-party integrators to use the same "Sovereign Handshake" flows.
//! - **Recommendation:** Formalize the `VaultSDK` as the primary entry point for all library consumers, deprecating direct use of `Wallet` where policy enforcement is required.
//!
//! ## 4. Conclusion
//! Extraction of a sellable SDK primitive is viable. The codebase already follows a modular pattern that separates protocol primitives from service routing.
