# Product Requirements Document (PRD): lib-conxian-core / Vault SDK v0.2.10

## 1. Executive Summary
Conxian builds native application infrastructure for Bitcoin. The `lib-conxian-core` (Vault SDK) is the foundational library for secure signing, policy enforcement, and protocol-aware transaction coordination. It provides the essential primitives consumed by the standalone Conxian Gateway and other ecosystem applications.

## 2. Strategic Focus: Vault SDK
The primary commercial and technical boundary of this repository is the **Vault SDK**.
- **Hardware-Backed Signing:** Secure interface for TEE-anchored key management.
- **Policy Enforcement:** Programmable guardrails for Bitcoin-anchored intents.
- **Protocol Primitives:** Canonical implementations of MuSig2, BitVM2, and chain adapters.

## 3. System Architecture
### 3.1. Shared Primitives
This library defines the common language for the Conxian ecosystem:
- **Control Models:** Unified types for state proposals, partner intake, and trust tiers.
- **Anchoring:** Decentralized state root persistence models (Tableland/On-chain).
- **Protocol Risk:** Multi-factor assessment models for L2s and sidechains.

### 3.2. Integration Boundaries
- **Gateway:** Consumes this library for routing, compliance, and metrics. Implementation lives in `conxian-gateway` repository.
- **Wallet:** Relies on SDK primitives for enclave-anchored signing and intent validation.

## 4. Supported Services & Layers
The SDK provides the models and verification logic for:
- **Sovereign Services:** Bisq, RGB, BitVM2, Changelly.
- **Bitcoin Layers:** Stacks, Lightning, Liquid, Rootstock, Babylon, BOB, Merlin, Botanix, and more.

## 5. Security & Auditing
- **Memory Safety:** Rust-based implementation for high-performance security.
- **MuSig2:** BIP327-compliant deterministic key aggregation (CON-145).
- **Trust Policy:** Explicit enforcement of approved bridge/messaging trust tiers (CON-791).
- **Zero Secret Egress (ZSE):** Core logic is audited for safe state transitions and credential handling.

## 6. Economic Alignment
The SDK implements the programmatic hooks for the Conxian creator-first economy, ensuring that value creation is verifiable and reward logic is transparent.
