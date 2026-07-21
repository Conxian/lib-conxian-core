# BOS Runtime Ownership Map (CON-413)

This document defines the canonical repository and runtime ownership for the Conxian Business Operating System (BOS). It ensures that every component has an explicit owner and that production paths remain mainnet-only and audit-ready.

## 1. Portfolio Taxonomy (May 2026 SDK-First GTM)

| Layer | Primary Responsibility | Representative Repositories |
| :--- | :--- | :--- |
| **Protocol Core** | Canonical protocol primitives, control contracts, and invariant validation. | `lib-conxian-core` |
| **Secure Enclave SDK** | Production hardware-backed signing, attestation, and policy flows. | `conxius-enclave-sdk` |
| **Core/SDK Adapter** | Narrow typed compatibility boundary; no provider or runtime ownership. | `lib-conxian-core-enclave` |
| **Gateway & Routing** | Protocol monitoring, compliance, service ingress. | `conxian-gateway ` |
| **Orchestration** | Automation, cross-service workflows, platform logic. | `conxius-platform` |
| **Reference Client** | User interface, biometric proof, reference signing. | `conxius-wallet`, `Conxian_UI` |
| **Governance & Ops** | Strategic specs, institutional alignment, business rules. | `conxian-business` |

## 2. Component Ownership Detail

### 2.1 Protocol Core (`lib-conxian-core`)
- **Canonical protocol primitives**: control models, lifecycle state machines,
  chain adapters, and invariant validation.
- **Contract surfaces**: platform-neutral signing, verifier, and BIP-110
  preflight contracts.
- **Explicit non-ownership**: Core is not the production Vault SDK and does not
  own hardware, provider behavior, runtime orchestration, persistence, or
  external side effects. Production MuSig2 sessions and BitVM2 proof
  verification are SDK/downstream responsibilities.
- **Job Card Schema (CJCS)** and shared financial models remain protocol types.

### 2.2 Secure Enclave SDK (`conxius-enclave-sdk`)
- **Production signing**: hardware-backed key custody, derivation, and concrete
  signing implementations.
- **Security flows**: attestation and policy enforcement.
- **Provider/runtime boundary**: concrete hardware providers and SDK runtime
  behavior remain in this repository or its downstream integrations.

### 2.3 Core/SDK companion adapter (`lib-conxian-core-enclave`)
- Maps the exact published SDK API to Core's typed contracts.
- Enforces Core-first fail-closed gates such as BIP-110 preflight.
- Does not implement hardware providers, attestation cryptography, persistence,
  networking, or runtime orchestration.

### 2.4 Supporting Services (`conxian-gateway `)
- **Sovereign Service Integration**: `conxian-gateway repo: src/engine/mod.rs` (Bisq, RGB, Changelly)
- **Bitcoin Layer Status**: `conxian-gateway repo: src/engine/mod.rs` (Stacks, Liquid, Rootstock, etc.)
- **Compliance & Risk**: `conxian-gateway repo: src/engine/remediation.rs`
- **MCP Integration**: `conxian-gateway repo: src/api/mcp_handler.rs`

### 2.4 External Platform Logic (`conxius-platform`)
- **Workflow Orchestration**: Cross-service state machine management.
- **Institutional Egress**: PAPSS, BRICS, and ISO 20022 signal normalization.

### 2.5 Reference Application (`conxius-wallet`)
- **Secure Signing**: StrongBox TEE integration.
- **Account Abstraction**: ERC-4337 biometric passkey flows.

## 3. Production Boundary Rules

1. **Mainnet-Only `main`**: No testnet principals, stubs, or mocks are permitted in the `main` branch of production repositories.
2. **Fail-Closed Execution**: All components must default to a secure, locked state if validation or connectivity fails.
3. **Additive Ingress**: External signals (e.g., ISO 20022) are triggers only and must generate a Proposal-only state update (CON-162).
4. **ZSE Compliance**: Zero Secret Egress is mandatory. Credentials must live in TEE or managed secret stores, never in Git.

## 4. Maintenance

This map is updated during weekly launch reviews (CON-229). Any architectural shift must be reflected here first before implementation.
