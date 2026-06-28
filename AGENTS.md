# Conxian Agent Guidelines: lib-conxian-core

This repository is the canonical home of the **Vault SDK** and shared protocol primitives. It is a "protocol-first" library.

## Strategic Priority (Vault SDK)
Prioritize the development and hardening of the `VaultSDK` primitive (`src/sdk_primitive.rs`). This is the primary commercial interface for the Conxian ecosystem.

## Architectural Boundaries (CON-700)
- **Core (`src/`):** Ownership of canonical types, state machines, invariant validation, and interface contracts.
- **Gateway:** Runtime orchestration, persistence, and external side effects live in the standalone `conxian-gateway` repository.
- **Rule:** If a change needs network IO, database access, or environment-specific branching, it belongs in the Gateway, not here.

## Trust Policy Enforcement (CON-791)
Ensure all cross-domain bridge or messaging metadata aligns with the approved trust-tier taxonomy in `control_model.rs`:
- `Strict` (T1)
- `Managed` (T2)
- `Expedient` (T3)

## Workflow Instructions
- **Verification:** Always run the full CI suite locally before pushing:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo build --workspace --release
  cargo test --workspace
  ```
- **MSRV:** The Minimum Supported Rust Version is 1.82. Do not use features stabilized after Rust 1.82. If CI is not available, verify with `cargo +1.82 test`.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.
- **Gap Tracking:** All identified gaps, stubs, and improvement opportunities are tracked in `docs/GAP_ANALYSIS_AND_SCORING.md`. Update this file when resolving or discovering gaps.
- **Protocol Standards:** FROST = RFC 9591 (final), MuSig2 = BIP-327 (deployed), OP_CAT = BIP-347 (not activated on mainnet), ERC-7683 = still Draft status. Implement against final standards; mark speculative implementations clearly as stubs.
