# Repository Hygiene Baseline & Controls (CON-516)

This document defines the mandatory hygiene standards, audit cadence, and exception register for repositories within the Conxian ecosystem.

## 1. Automated Guardrails

### 1.1 Forbidden File Checks
The CI pipeline (`.github/workflows/hygiene.yml`) and local verification script (`scripts/verify_tracked_artifacts.py`) automatically scan every commit, push, and pull request for forbidden tracked files. The checked patterns are fully synchronized across the codebase, covering:
- **Environment & config files**: `.env`, `.env.*`
- **Private keys & certificates**: `*.pem`, `*.key`, `*.pub`
- **Dependency directories**: `node_modules`
- **Build artifacts**: `target/`, `dist/`, `build/`
- **Test reports & coverage**: `test-results/`, `playwright-report/`

### 1.2 Secret Scanning
GitHub Secret Scanning must be enabled for all public and private repositories to detect accidental credential exposure.

### 1.3 Dependency Review
Dependency graph and Dependabot alerts must be enabled to monitor for vulnerable packages (RUSTSEC, CVEs).

## 2. Audit Cadence

Repository hygiene audits are performed according to the following schedule:

| Audit Type | Frequency | Responsibility |
| :--- | :--- | :--- |
| **Automated CI** | Per Commit | GitHub Actions |
| **Manual hygiene review** | Monthly | Security Lead / Maintainer |
| **Full Security Audit** | Quarterly | Internal/External Audit Team |

## 3. Exception Register

Repositories that temporarily diverge from these standards must be recorded here.

| Repository | Exception | Rationale | Owner | Expiry |
| :--- | :--- | :--- | :--- | :--- |
| None | - | - | - | - |

## 4. Remediation Procedure

If a forbidden file is discovered in version control:
1. **Immediate Removal**: Use `git rm --cached` to remove the file from the current index.
2. **History Cleaning**: If sensitive data (secrets) was committed, use `git filter-repo` or BFG Repo-Cleaner to purge it from the entire history.
3. **Credential Rotation**: Any exposed secrets must be rotated immediately.
4. **ZSE Verification**: Perform a follow-up Zero Secret Egress (ZSE) check.

## 5. Supplier-State SLO (CON-542)

To maintain the integrity of the Conxian ecosystem, the following Service Level Objectives (SLOs) are enforced for repository hygiene and security:

| Event Type | Triage Time | Closure/Remediation | Responsibility |
| :--- | :--- | :--- | :--- |
| **Security Regression (P0)** | < 4 Hours | < 24 Hours | Security Lead |
| **Secret Exposure (P0)** | < 1 Hour | < 4 Hours (Rotation) | Account Owner |
| **Hygiene CI Failure (P1)** | < 12 Hours | < 3 Business Days | PR Author |
| **Governance Drift (P2)** | < 48 Hours | < 5 Business Days | Maintainer |
| **Audit Exception Expiry** | 5 Days Before | On/Before Expiry | Exception Owner |

### 5.1 Enforcement
Failure to meet these SLOs triggers an automated escalation to the Office of the Founder (CON-286) and blocks further mainnet state transitions in the standalone Gateway.
