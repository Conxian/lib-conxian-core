# Repository Hygiene Baseline & Controls (CON-516)

This document defines the mandatory hygiene standards, audit cadence, and exception register for repositories within the Conxian ecosystem.

## 1. Automated Guardrails

### 1.1 Forbidden File Checks
The CI pipeline (`.github/workflows/hygiene.yml`) automatically scans every push and pull request for forbidden tracked files, including:
- Environment files (`.env`)
- Private keys (`*.pem`, `*.key`)
- Dependency directories (`node_modules`)
- Build artifacts (`target/`, `dist/`, `build/`)
- Test reports (`test-results/`, `playwright-report/`)

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
