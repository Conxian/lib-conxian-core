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
