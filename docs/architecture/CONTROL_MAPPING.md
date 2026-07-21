# Conxian Control & Assurance Mapping (CON-1180)

This document maps security and operational controls across the public repository estate and release surfaces.

## 1. Repository Roles & Baseline Controls

| Role | Target Repositories | Branch Protection | Secret Handling | Dependency Review |
| :--- | :--- | :--- | :--- | :--- |
| **Protocol Core** | `lib-conxian-core`, `conxian-nexus` | Required PRs, 2+ Approvals, Status Checks | Zero Secret Egress (ZSE) | Required on all PRs |
| **Security SDK** | `lib-conclave-sdk` | Required PRs, Security Lead Approval | Enclave-Bound (No tracking) | Required on all PRs |
| **Infrastructure** | `conxian-gateway` | Required PRs, Status Checks | Environment variables (No .env) | Required on all PRs |
| **Product/UI** | `conxius-wallet`, `conxian_ui` | Required PRs | Environment variables | Required on all PRs |
| **Website** | `conxian-labs-site` | Required PRs | Environment variables | Required on all PRs |

## 2. Release & Deployment Posture

| Surface | Release Type | Versioning | Deployment Logic |
| :--- | :--- | :--- | :--- |
| **Reusable SDKs** | Tagged Release | SEMVER (vX.Y.Z) | Pinned Git tags / Crates.io |
| **App-Layer** | Deployment Tracked | Main branch SHA | Continuous Deployment (CD) |
| **Protocol** | Tagged Release | SEMVER (vX.Y.Z) | Controlled Promotion Lanes |

## 3. Enforcement Status (Audit Date: 2026-06-14)

- **Branch Protections:** Enabled across all primary public repositories.
- **Push Protection:** Active on the Conxian organization.
- **Secret Scanning:** Active on all public repositories.
- **Dependency Review:** Workflow integrated into CI baseline for core repositories.
- **Gitleaks:** Integrated into `lib-conxian-core` CI.

## 4. Known Gaps & Remediation

- **Manual Enforcement:** Release tagging for app-layer shims remains manual.
- **Visibility Audit:** Periodic review of private/public boundaries is ongoing (CON-1183).

## 5. Canonical risk-profile controls (CORE-007)

The schema-v1 canonical risk-profile contract is a protocol control, not a live risk engine.
`lib-conxian-core` owns the checked-in `data/risk_profiles/v1.json` artifact, schema/version
validation, score bounds and polarity, explicit unknown/not-assessed states, exact target
coverage, chain-to-family reconciliation, trust-tier policy invariants, and the versioned rail
metadata compatibility wrapper. See
[`docs/architecture/RISK_PROFILES.md`](RISK_PROFILES.md).

| Control | Core | Nexus | Gateway | Wallet/adapters |
| --- | --- | --- | --- | --- |
| Schema/set/profile revision | Define and validate | Track compatible observations | Expose and persist compatible versions | Pin and preserve versions |
| Static six-dimension metadata | Store only approved artifact values | Compare with live evidence | Apply separate runtime policy | Display/pass through as static |
| Evidence and provenance | Require evidence for assessed/partial states | Acquire and verify empirical evidence | Persist/audit references | Preserve provenance |
| Trust/verification/finality assumptions | Reuse `validate_trust_tier_policy`; no universal finality rule | Check against verifier capabilities | Decide runtime eligibility | Reject incompatible declarations |
| Routing and live status | Explicitly out of scope | Observe/verify | Own orchestration and route selection | Request/sign under caller policy |

The initial set intentionally marks every family and chain profile `not_assessed`; the issue and
repository docs are governance references, not evidence. Any profile change must update the data
artifact, affected revision, set version when applicable, evidence/change reference, tests,
documentation, and release notes in one reviewable change.
