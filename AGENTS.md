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
4. **BIP-110 Alignment:** Ensure all Bitcoin-related code supports reduced-data softfork principles

## BIP-110 (Reduced Data Temporary Softfork)

BIP-110 is a consensus proposal that temporarily limits data embedding in Bitcoin to refocus on monetary use. All Bitcoin-related code should:

- **Support BIP-110 rules**: Max 256-byte pushdata, 83-byte OP_RETURN, 34-byte ScriptPubKey
- **Prefer monetary transactions**: Design for peer-to-peer cash, not data storage
- **Optimize for clean blocks**: Reduce inscription/ordinal noise in fee estimation
- **Document compliance**: Use `Bip110Compliance` struct for validation in `control_model`

The `Bip110Compliance` struct provides:
- `validate_pushdata(size)` - Max 256-byte pushdata
- `validate_op_return(size)` - Max 83-byte OP_RETURN
- `validate_script_pubkey(size)` - Max 34-byte ScriptPubKey
- `validate_witness_element(size)` - Max 256-byte witness
- `validate_transaction(...)` - Full transaction validation

See `docs/BIP110_ALIGNMENT.md` for full guidance.

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

## Cross-Repository Alignment

| Repository | Bitcoin Layer | BIP-110 Priority |
|------------|-------------|-----------------|
| `conxius-enclave-sdk` | Core signing | HIGH |
| `conxius-wallet` | L1 wallet | HIGH |
| `conxian-nexus` | Observation | MEDIUM |
| `lib-conxian-core` | Protocol | MEDIUM |

## Crate Publishing
This repository is published as `lib-conxian-core` on crates.io.
The Vault SDK is published separately as `conxius-enclave-sdk`.

## Workflow Instructions
- **Verification:** Always run `cargo test --workspace` to verify changes.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **BIP-110 Check:** Run `cargo clippy` to ensure no deprecated data embedding patterns.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.

## Contact
- Support: support@conxian-labs.com
- Security: security@conxian-labs.com
- Labs: https://www.conxian-labs.com
