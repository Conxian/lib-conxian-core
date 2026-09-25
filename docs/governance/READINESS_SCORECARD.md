# Cross-Lane Readiness Scorecard (CON-1273)

This scorecard tracks the readiness of the Conxian ecosystem across active development lanes.

## 1. Readiness Categories

| Category | Description | Status | Evidence |
| :--- | :--- | :--- | :--- |
| **Launch Gates** | Critical blockers for public release. | 🟡 In Progress | PRD, CXIP index |
| **Repo Trust** | CI/CD, security hygiene, and governance standards. | 🟢 Healthy | `.github` workflows |
| **Release Maturity** | Versioning, changelogs, and deployment stability. | 🟢 v0.3.3 | CHANGELOG.md |
| **Product Proof** | Core protocol verification (Tests, Audits). | 🟢 281 Workspace Tests | src/tests.rs |
| **Commercial Safety** | Claim evidence, SLA tiering, and regulatory alignment. | 🟢 Aligned | SUPPORT.md, docs/ECONOMY.md |

## 2. Evidence Requirements

- **Closure Evidence**: All high-priority Linear issues (`Urgent`, `High`) must be resolved.
- **Review Cycle**: Weekly review by core maintainers.
- **Verification**: Zero failed CI checks in the main branch.

## 3. Active Lane Status

- **Protocol Core**: 🟢 Ready (v0.3.3) - Open Source / No SLA
- **Gateway Runtime**: 🟡 Hardening - Enterprise B2B SLA Tier
- **Enclave SDK**: 🟡 Boundary Audit (v2.0.17) - Hardware Attestation
- **Wallet UI**: 🟡 Integration Testing - Sovereign / Non-Custodial
