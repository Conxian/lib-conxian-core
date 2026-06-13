# Control Plane Modules Roadmap (CON-773)

This document outlines the implementation plan and architectural boundaries for the first tranche of control-plane modules within the Conxian private operations surface.

## 1. Module Overview

### 1.1 Release Governance
- **Purpose:** Track, review, and approve protocol-level releases across the ecosystem.
- **Key Data Models:** `Release`, `ReleaseTrack`, `ReleaseStatus`.
- **Dependency:** `lib-conxian-core` (Changelog/Version), `conxian-gateway` (Deployment status).

### 1.2 Audit Dashboard
- **Purpose:** Provide operational visibility and security auditing for internal events.
- **Key Data Models:** `AuditEvent`, `AuditCategory`, `AuditSeverity`.
- **Dependency:** `conxian-gateway` (Event stream), `conxian-nexus` (System logs).

### 1.3 Policy Approval Queue
- **Purpose:** Formal change control for protocol parameters and sensitive operational settings.
- **Key Data Models:** `PolicyChangeRequest`, `PolicyApprovalStatus`.
- **Dependency:** `VaultSDK` (Policy enforcement), `conxian-nexus` (Config application).

### 1.4 Environment & Config Registry
- **Purpose:** Centralized registry for private environment endpoints and non-secret configuration.
- **Key Data Models:** `EnvironmentRegistryEntry`, `ConfigRegistryEntry`, `EnvironmentType`.
- **Dependency:** `GCP Infrastructure` (Live endpoints).

## 2. Dependency Map

| Module | Consumes | Informs |
|--------|----------|---------|
| Release Governance | Git/CI Metrics | Deployment Queue |
| Audit Dashboard | Gateway Event Bus | Compliance Reports |
| Policy Approvals | VaultSDK Invariants | Nexus Configuration |
| Config Registry | Ops Metadata | Adapter Runtime |

## 3. Ownership & Boundaries

### What these modules OWN:
- Workflow state for internal operations.
- Approval records and audit trails.
- Configuration metadata for private environments.

### What these modules MUST NOT own:
- **Custody:** No possession of customer funds.
- **Secrets:** Private keys and sensitive credentials remain in Enclaves/Vaults.
- **Protocol Truth:** Canonical protocol state lives in the Bitcoin/UTXO layers, not the control plane.

## 4. Implementation Tranche 1

1. **Schema Finalization:** (Completed in `lib-conxian-core`).
2. **Admin API Contracts:** Define JSON-RPC/REST interfaces between the Control Plane and Nexus.
3. **Initial UI Route Map:**
   - `/ops/releases`
   - `/ops/audit`
   - `/ops/policies`
   - `/ops/registry`
