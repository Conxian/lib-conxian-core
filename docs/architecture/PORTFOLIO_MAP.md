# Conxian Portfolio Map & Review Standards (CON-468)

This document classifies every repository in the Conxian-Labs stack by layer, role, and evaluation standard, ensuring a consistent security and quality posture across the full Bitcoin-native system.

## 1. Portfolio Classification (May 2026 SDK Pivot)

| Layer | Repository | Role | Evaluation Standard |
| :--- | :--- | :--- | :--- |
| **Protocol Core** | `lib-conxian-core` | Canonical protocol primitives, control contracts, and invariants | **P0 - Hardened**: BIP-aligned, fail-closed, no mocks, full test coverage. |
| **Secure Enclave SDK** | `conxius-enclave-sdk` | Production hardware-backed signing, attestation, and policy flows | **P0 - Hardened**: provider-backed, fail-closed, security-reviewed. |
| **Core/SDK Adapter** | `lib-conxian-core-enclave` | Narrow compatibility adapter between Core contracts and the SDK | **P0 - Hardened**: typed boundary, no provider/runtime ownership. |
| **Supporting Infra** | `conxian-gateway`  | Unified API & Protocol Routing | **P0 - Hardened**: TEE-anchored execution, ZSE compliant, audit-ready. |
| **User & Application** | `conxius-wallet` | Reference Asset Management Client | **P1 - Standard**: StrongBox/TEE signing, Passkey auth, zero-PII persistence. |
| **User & Application** | `Conxian_UI` | Product Dashboards & Landing | **P1 - Standard**: High-contrast theme, responsive, type-safe, CI-badge mandatory. |
| **Shared Runtime** | `conxius-platform` | Workflow Orchestration | **P1 - Standard**: Fail-closed orchestrations, mainnet-only release paths. |
| **Governance & OS** | `conxian-business` | Institutional Strategy & Rules | **P2 - Strategic**: ZSE compliant, ZK-Data Room integration, Linear-native strategy. |

## 2. Review Standards

### 2.1 P0 - Hardened (Critical Floor)
- **Security**: Mandatory TEE/HSM anchoring for signing. No credentials in code.
- **Resilience**: Must implement "Handoff Limbo" and "Fail-Closed" patterns.
- **Verification**: 100% pass on all cryptographic and state transition tests.
- **Branch Policy**: `main` is mainnet-only. Direct promotion from `staged` only.

### 2.2 P1 - Standard (Operational Surface)
- **Hygiene**: No generated artifacts (`node_modules`, `target`) in Git.
- **Clarity**: Purpose, status, and audience must be defined in README.
- **CI/CD**: Required status checks, linting, and formatting on every PR.
- **Release**: Tagged releases with versioned changelogs.

### 2.3 P2 - Strategic (Institutional Brain)
- **Privacy**: No public exposure of strategic, legal, or unvetted operational content.
- **Traceability**: All strategic decisions must be linked to Linear ExCo records.
- **Sanitization**: Regular audits to move strategic detail to Linear Virtual Office.

## 3. Dependency Relationships
- **lib-conxian-core** is the shared protocol foundation for all integrators; it
  does not provide production Vault SDK, hardware, provider, or runtime behavior.
- **conxius-enclave-sdk** owns production signing, attestation, and policy flows.
- **lib-conxian-core-enclave** provides the narrow Core/SDK compatibility boundary
  used by downstream applications.
- **conxian-gateway** owns runtime orchestration and protocol routing.
- **conxius-wallet** is the reference application consuming the SDK boundary and
  Core protocol contracts.

## 4. Maintenance
This map is reviewed during the Weekly Launch Review (CON-229). Any repo addition or role shift requires an update here.
