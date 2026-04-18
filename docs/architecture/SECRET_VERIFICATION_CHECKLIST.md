# Approved Secret Verification Checklist (CON-272)

This document defines the mandatory verification standard for all secrets and credentials used across Conxian repositories and environments.

## 1. Governance & Accountability
- [ ] **Named Owner**: Every active secret must have a designated maintainer or business unit owner.
- [ ] **Defined Purpose**: The logic or service requiring the secret must be explicitly documented.
- [ ] **Scoped Access**: Access is limited using the least-privilege principle (e.g., repository-specific or environment-specific secrets).

## 2. Storage & hygiene (ZSE Compliance)
- [ ] **No Git Persistence**: Secrets MUST NOT exist in any Git branch (public or private).
- [ ] **Managed Store Usage**: Credentials must live in GitHub Secrets, GCP Secret Manager, or the hardware-backed StrongBox TEE.
- [ ] **Sanitized Artifacts**: Verified that `.env`, `.pem`, `.key`, and generated reports are excluded via `.gitignore`.

## 3. Lifecycle & Rotation
- [ ] **Rotation Status**: Verified the last rotation date. P0 secrets must be rotated every 90 days.
- [ ] **Incident History**: Checked for any suspected exposure or leakage in logs or third-party audits.
- [ ] **Decommission Path**: Documented the procedure for revoking the credential if the service is retired.

## 4. Verification Workflow
1. **Automated Scan**: Run `trufflehog` or `gitleaks` on the repository.
2. **Manual Review**: Cross-reference found matches against structural code (field names, types).
3. **Attestation**: Record the verification outcome in the relevant Linear issue.

## 5. Maintenance
This checklist is the required standard for all Security Boundary Audits. Failure to comply blocks mainnet promotion.
