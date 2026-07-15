# Conxian Agent Guidelines: lib-conxian-core

This repository contains **shared protocol primitives** for the Conxian ecosystem.

## ⚠️ Important: Vault SDK Location

The **production Vault SDK** is now in the [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate (v2.0.11), NOT in this repository.

This repository provides:
- Protocol primitives (types, state machines, invariants)
- Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint)
- Control models (trust tiers per CON-791)

## Strategic Priorities
1. **Protocol Primitives:** Maintain canonical types and invariant validation
2. **Chain Adapters:** Keep adapters consistent across the ecosystem
3. **Control Models:** Trust tier taxonomy enforcement (CON-791)

## Architectural Boundaries (CON-700)
- **Core (`src/`):** Ownership of canonical types, state machines, invariant validation, and interface contracts.
- **Gateway:** Runtime orchestration, persistence, and external side effects live in the standalone `conxian-gateway` repository.
- **Vault SDK:** Hardware-backed signing, attestation, and policy flows are in `conxius-enclave-sdk`.
- **Rule:** If a change needs network IO, database access, or environment-specific branching, it belongs in the Gateway, not here. This is automatically enforced by `scripts/verify_contamination_guard.py`.

## Trust Policy Enforcement (CON-791)
Ensure all cross-domain bridge or messaging metadata aligns with the approved trust-tier taxonomy in `control_model.rs`:
- `Strict` (T1)
- `Managed` (T2)
- `Expedient` (T3)
- `ObserverOnly`

## Crate Publishing
This repository is published as `lib-conxian-core` on crates.io.
The Vault SDK is published separately as `conxius-enclave-sdk`.

## Workflow Instructions
- **Verification:** Always run `cargo test --workspace` to verify changes.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.

## Contact
- Support: support@conxian-labs.com
- Security: security@conxian-labs.com
- Labs: https://www.conxian-labs.com
