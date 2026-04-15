# Conxian Portfolio Map & Review Standards (CON-468)

This document classifies every repository in the Conxian-Labs stack by layer, role, and evaluation standard, ensuring a consistent security and quality posture across the full Bitcoin-native system.

## 1. Portfolio Classification

| Layer | Repository | Role | Evaluation Standard |
| :--- | :--- | :--- | :--- |
| **Decentralization-Critical** | `lib-conxian-core` | Cryptographic & Protocol Primitives | **P0 - Hardened**: BIP-aligned, fail-closed, no mocks, full test coverage. |
| **Decentralization-Critical** | `conxian-gateway` | Unified API & Protocol Routing | **P0 - Hardened**: TEE-anchored execution, ZSE compliant, audit-ready. |
| **User & Application Surface** | `conxius-wallet` | Sovereign Asset Management | **P0 - Hardened**: StrongBox/TEE signing, Passkey auth, zero-PII persistence. |
| **User & Application Surface** | `Conxian_UI` | Product Dashboards & Landing | **P1 - Standard**: High-contrast theme, responsive, type-safe, CI-badge mandatory. |
| **Shared Runtime & Infra** | `conxius-platform` | Workflow Orchestration | **P1 - Standard**: Fail-closed orchestrations, mainnet-only release paths. |
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
- **lib-conxian-core** is the root dependency for all protocol-bearing repos.
- **conxian-gateway** serves as the source of truth for protocol state to **conxius-platform**.
- **conxius-wallet** provides the execution authority for all user-initiated intents.

## 4. Maintenance
This map is reviewed during the Weekly Launch Review (CON-229). Any repo addition or role shift requires an update here.
