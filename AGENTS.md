# Conxian Agent Guidelines: lib-conxian-core

This repository is the canonical home of the **Vault SDK** and shared protocol primitives. It is a "protocol-first" library.

## Strategic Priority (Vault SDK)
Prioritize the development and hardening of the `VaultSDK` primitive (`src/sdk_primitive.rs`). This is the primary commercial interface for the Conxian ecosystem.

## Architectural Boundaries (CON-700)
- **Core (`src/`):** Ownership of canonical types, state machines, invariant validation, and interface contracts.
- **Gateway:** Runtime orchestration, persistence, and external side effects live in the standalone `conxian-gateway` repository.
- **Rule:** If a change needs network IO, database access, or environment-specific branching, it belongs in the Gateway, not here. This is automatically enforced by `scripts/verify_contamination_guard.py`.

## Trust Policy Enforcement (CON-791)
Ensure all cross-domain bridge or messaging metadata aligns with the approved trust-tier taxonomy in `control_model.rs`:
- `Strict` (T1)
- `Managed` (T2)
- `Expedient` (T3)

## Workflow Instructions
- **Verification:** Always run `cargo test --workspace` to verify changes.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.
