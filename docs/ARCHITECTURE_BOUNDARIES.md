# Conxian Repository & SDK Boundaries (CON-555 / CON-700)

This document clarifies ownership and responsibility boundaries between core libraries and service layers in the Conxian ecosystem.

## 1. Core Primitives (`lib-conxian-core`)

**Role**: Root dependency for protocol-bearing components.

**Responsibilities (managed in independent repository)**:
- Canonical data models (e.g., `StateProposal`, `PartnerLead`).
- Low-level cryptographic primitives (MuSig2, BitVM2 proof verification).
- Shared financial and yield metrics logic.
- Job Card Schema (CJCS) specifications.
- Shared wallet models and generic signing interfaces.
- Shared control-model primitives for:
  - wallet authority classes,
  - protected-action lifecycle states,
  - trigger and pending-action state modeling,
  - timelock/quorum invariant descriptors and validators,
  - signed envelope descriptors and replay/idempotency helpers,
  - session trust/security claim types,
  - adapter-facing traits for intent authorization and session issuance.

**Constraints**:
- Must NOT contain hardware-specific or enclave-specific implementation logic.
- Must NOT contain provider-specific standalone gateway runtime logic, transport clients, or persistence adapters.
- May define interfaces/traits for integration points, but runtime implementations stay outside core.
- Must remain platform-agnostic and audit-ready.
- Must avoid "dumping ground" growth: if behavior depends on environment, tenancy, provider APIs, or workflow orchestration, it belongs to standalone Conxian Gateway/Platform.

## 2. Secure Enclave SDK (`lib-conclave-sdk`)

**Role**: Implementation layer for secure execution environments and trusted hardware.

**Responsibilities (managed in independent repository)**:
- TEE (Trusted Execution Environment) integration logic (e.g., StrongBox).
- Remote attestation verification and proof generation.
- Secure key management and enclave-bound signing.
- Biometric auth and Passkey (FIDO2) implementation.

**Constraints**:
- Depends on `lib-conxian-core` for data models and protocol rules.
- Contains the "How" of secure execution, while Core contains the "What".

## 3. Unified standalone Conxian Gateway (`conxian-gateway`)

**Role**: Single entry point for sovereign services and protocol routing.

**Responsibilities (managed in independent repository)**:
- Unified REST API and MCP server.
- Protocol monitoring and TVL aggregation.
- Compliance and risk assessment engine.
- Runtime/provider implementations that satisfy core integration traits.
- Routing requests to external Bitcoin layers and sidechains.

## 4. Interaction Map

1. **standalone Conxian Gateway** uses **lib-conxian-core** for state and control-model types.
2. **standalone Conxian Gateway** implements runtime adapters and provider workflows against core traits.
3. **Wallet** uses **lib-conclave-sdk** for enclave-anchored signing.
4. **lib-conclave-sdk** uses **lib-conxian-core** to ensure signed intents align with protocol rules.

## 5. Core-vs-standalone Conxian Gateway guardrail (CON-700)

Use this decision rule when adding new capability:

- **Core (`lib-conxian-core`)**: canonical types, state machines, invariant validation, and interface contracts.
- **standalone Conxian Gateway (`conxian-gateway`)**: runtime orchestration, persistence, provider integrations, retries, observability, and external side effects.

If a change needs network IO, database access, deployment/environment configuration, or provider-specific branching, it should not land in core.
