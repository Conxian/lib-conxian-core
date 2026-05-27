# BOS Runtime Ownership Map (CON-413)

This document defines the canonical repository and runtime ownership for the Conxian Business Operating System (BOS). It ensures that every component has an explicit owner and that production paths remain mainnet-only and audit-ready.

## 1. Portfolio Taxonomy (May 2026 SDK-First GTM)

| Layer | Primary Responsibility | Representative Repositories |
| :--- | :--- | :--- |
| **Vault SDK & Core** | Signing primitives, policy enforcement, shared models. | `lib-conxian-core` |
| **Gateway & Routing** | Protocol monitoring, compliance, service ingress. | `conxian-gateway (Extracted)` |
| **Orchestration** | Automation, cross-service workflows, platform logic. | `conxius-platform` |
| **Reference Client** | User interface, biometric proof, reference signing. | `conxius-wallet`, `Conxian_UI` |
| **Governance & Ops** | Strategic specs, institutional alignment, business rules. | `conxian-business` |

## 2. Component Ownership Detail

### 2.1 Vault SDK (`lib-conxian-core`)
- **Vault SDK Primitive**: `src/sdk_primitive.rs` (Primary sellable primitive)
- **MuSig2 Key Aggregation**: `src/musig2.rs`
- **BitVM2 Proof Verification**: `src/bitvm2.rs`
- **Job Card Schema (CJCS)**: `src/cjcs.rs`
- **Shared Financial Models**: `src/lib.rs`

### 2.2 Supporting Services (`conxian-gateway (Extracted)`)
- **Sovereign Service Integration**: `extracted/conxian-gateway/src/engine/mod.rs` (Bisq, RGB, Changelly)
- **Bitcoin Layer Status**: `extracted/conxian-gateway/src/engine/mod.rs` (Stacks, Liquid, Rootstock, etc.)
- **Compliance & Risk**: `extracted/conxian-gateway/src/engine/remediation.rs`
- **MCP Integration**: `extracted/conxian-gateway/src/api/mcp_handler.rs`

### 2.3 External Platform Logic (`conxius-platform`)
- **Workflow Orchestration**: Cross-service state machine management.
- **Institutional Egress**: PAPSS, BRICS, and ISO 20022 signal normalization.

### 2.4 Reference Application (`conxius-wallet`)
- **Secure Signing**: StrongBox TEE integration.
- **Account Abstraction**: ERC-4337 biometric passkey flows.

## 3. Production Boundary Rules

1. **Mainnet-Only `main`**: No testnet principals, stubs, or mocks are permitted in the `main` branch of production repositories.
2. **Fail-Closed Execution**: All components must default to a secure, locked state if validation or connectivity fails.
3. **Additive Ingress**: External signals (e.g., ISO 20022) are triggers only and must generate a Proposal-only state update (CON-162).
4. **ZSE Compliance**: Zero Secret Egress is mandatory. Credentials must live in TEE or managed secret stores, never in Git.

## 4. Maintenance

This map is updated during weekly launch reviews (CON-229). Any architectural shift must be reflected here first before implementation.
