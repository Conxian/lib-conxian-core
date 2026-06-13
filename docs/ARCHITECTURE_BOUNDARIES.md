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


## 6. SDK Ownership & Version Policy (CON-1178)

### 6.1. Canonical Ownership
- **Shared Core (`lib-conxian-core`):** Protocol-bearing primitives, canonical data models, and platform-agnostic crypto. Owned by the Protocol Team.
- **Secure Enclave (`lib-conclave-sdk`):** TEE-specific implementations and hardware-bound signing. Owned by the Security Team.
- **Gateway-Local:** Provider-specific orchestration and temporary integration shims. Owned by the Infrastructure Team.

### 6.2. Version Policy
- **Protocol Core:** SEMVER-compliant releases. Breaking changes to canonical models require a 2-week deprecation notice in `CHANGELOG.md`.
- **Enclave SDK:** Beta/RC dependencies are allowed but must be pinned to exact revisions.
- **Stacks JS / Clarinet:** Standardized on v7.3+ family across all production surfaces.

### 6.3. Consumption Guidance
- **Production Apps:** Must consume `lib-conxian-core` via crates.io or pinned Git tags.
- **Local Integrations:** Use repo-local code for experimental rails only. Once a rail reaches T2 (Managed) maturity, its models must be upstreamed to Core.

### 6.4. Release Posture
- **Reusable SDKs:** (Core, Enclave) must have tagged GitHub releases and maintained changelogs.
- **App-Layer:** (Gateway, Wallet, UI) are deployment-tracked. Main branch state determines production status.
