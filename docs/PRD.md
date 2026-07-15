# Product Requirements Document (PRD): lib-conxian-core v0.2.10

## 1. Executive Summary
Conxian builds native application infrastructure for Bitcoin. The `lib-conxian-core` is the foundational library for shared protocol primitives used across the Conxian ecosystem.

> ⚠️ **Note:** The production **Vault SDK** (hardware-backed signing, attestation, FROST DKG, BitVM2) is now in the [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate (v2.0.11).

## 2. Strategic Focus: Protocol Primitives
This repository's primary purpose is providing shared protocol primitives:
- **Control Models:** Unified types for state proposals, partner intake, and trust tiers.
- **Chain Adapters:** Reusable adapters for Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint.
- **Protocol Primitives:** Canonical implementations of MuSig2, BitVM2, and invariant validation.

## 3. System Architecture
### 3.1. Shared Primitives
This library defines the common language for the Conxian ecosystem:
- **Control Models:** Unified types for state proposals, partner intake, and trust tiers (CON-791).
- **Anchoring:** Decentralized state root persistence models.
- **Protocol Risk:** Multi-factor assessment models for L2s and sidechains.

### 3.2. Integration Boundaries
- **Vault SDK:** Hardware-backed signing lives in `conxius-enclave-sdk` (NOT this repository).
- **Gateway:** Consumes this library for routing, compliance, and metrics. Implementation lives in `conxian-gateway` repository.
- **Wallet:** Relies on `conxius-enclave-sdk` for enclave-anchored signing.

## 4. Supported Services & Layers
This library provides the models and verification logic for:
- **Sovereign Services:** RGB, BitVM2.
- **Bitcoin Layers:** Stacks, Lightning, Liquid, Rootstock, Babylon, BOB, Merlin, Botanix, and more.

## 5. Security & Auditing
- **Memory Safety:** Rust-based implementation for high-performance security.
- **MuSig2:** BIP327-compliant deterministic key aggregation (CON-145).
- **Trust Policy:** Explicit enforcement of approved bridge/messaging trust tiers (CON-791).
- **Zero Secret Egress (ZSE):** Core logic is audited for safe state transitions and credential handling.

## 6. Economic Alignment
The library implements the programmatic hooks for the Conxian creator-first economy, ensuring that value creation is verifiable and reward logic is transparent.

## 7. Crate Relationships
| Crate | Version | Purpose |
|-------|---------|---------|
| `conxius-enclave-sdk` | 2.0.11 | **Production Vault SDK** - Hardware signing, attestation |
| `lib-conxian-core` | 0.2.10 | **Protocol primitives** - Types, invariants, adapters |
| `conxian-gateway` | 0.1.4 | Runtime orchestration and middleware |

## 8. Contact
- **Conxian Labs:** https://www.conxian-labs.com
- **Support:** support@conxian-labs.com
- **Security:** security@conxian-labs.com
