# Conxian Repository & SDK Boundaries (CON-555)

This document clarifies the ownership and responsibility boundaries between the various core libraries and SDKs in the Conxian ecosystem.

## 1. Core Primitives (`lib-conxian-core`)

**Role**: Root dependency for all protocol-bearing components.

**Responsibilities**:
- Canonical data models (e.g., `StateProposal`, `PartnerLead`).
- Low-level cryptographic primitives (MuSig2, BitVM2 proof verification).
- Shared financial and yield metrics logic.
- Job Card Schema (CJCS) specifications.
- Shared wallet models and generic signing interfaces.

**Constraints**:
- Must NOT contain hardware-specific or enclave-specific implementation logic.
- Must remain platform-agnostic and audit-ready.

## 2. Secure Enclave SDK (`lib-conclave-sdk`)

**Role**: Implementation layer for secure execution environments and trusted hardware.

**Responsibilities**:
- TEE (Trusted Execution Environment) integration logic (e.g., StrongBox).
- Remote attestation verification and proof generation.
- Secure key management and enclave-bound signing.
- Biometric auth and Passkey (FIDO2) implementation.

**Constraints**:
- Depends on `lib-conxian-core` for data models and protocol rules.
- Contains the "How" of secure execution, while Core contains the "What".

## 3. Unified Gateway (`conxian-gateway`)

**Role**: Single entry point for all sovereign services and protocol routing.

**Responsibilities**:
- Unified REST API and MCP server.
- Protocol monitoring and TVL aggregation.
- Compliance and risk assessment engine.
- Routing requests to external Bitcoin layers and sidechains.

## 4. Interaction Map

1. **Gateway** uses **lib-conxian-core** for state and protocol models.
2. **Wallet** uses **lib-conclave-sdk** for enclave-anchored signing.
3. **lib-conclave-sdk** uses **lib-conxian-core** to ensure signed intents align with protocol rules.
