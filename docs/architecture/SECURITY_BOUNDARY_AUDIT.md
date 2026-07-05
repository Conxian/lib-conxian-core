# Security Boundary Audit: lib-conxian-core (CON-188 & CON-191)

## 1. Audit Summary
**Date:** 2026-04-15
**Status:** PASSED
**Scope:** Secret detection and public/private boundary verification.

## 2. Findings

### 2.1. Secret Detection
- A scan for `password`, `secret`, `key`, and `private` was performed across the repository.
- All matches in `src/` and `the extracted gateway` were found to be structural (e.g., `SigningKey` type, `public_key` field name) rather than actual credentials.
- No `.env` files or `.pem` files are tracked in version control.

### 2.2. Public/Private Boundary
- Business-critical strategy documents have been migrated to the secure Linear Virtual Office (CON-306).
- The repository only contains open-source compatible logic and models required for the standalone Conxian Gateway.
- Operational mailboxes (`support.rs`) use public-safe placeholder domain (`mail.privateemail.com`) without credentials.

## 3. Remediation
- Verified `.gitignore` hardening to prevent future leakage of `*.key`, `*.keystore`, and `.env*` files.
- Version bumped to 0.2.2 to reflect the alignment sweep completion.
- **2026-05-05 Update:** Implemented mandatory administrative authentication for high-privilege endpoints (Proposal Approval/Execution, MCP) via `X-Gateway-Admin-Key (enforced by standalone Gateway)` requirement (CON-420).

## 4. Conclusion
The `lib-conxian-core` and `conxian-gateway` components are compliant with Zero Secret Egress (ZSE) standards.

### 4.1. Hardened Architectural Boundary (2026-06-27)
- **Removal of Environment Side-Effects**: Logic in `src/wallet.rs` that directly read from environment variables (`std::env::var`) has been removed. This eliminates insecure defaults and ensures the core library remains platform-agnostic and free of configuration-dependent "magic" behavior.
- **Enforced Contamination Guard**: The `scripts/verify_contamination_guard.py` script now explicitly forbids the use of `std::env` within core production code (`src/`).
- **Secret Hygiene**: Verified that all wallet and signing initializations are now explicit, passing credentials only as function arguments from the runtime layer.

High-privilege state transition endpoints remain protected by mandatory administrative authentication as enforced by the standalone Gateway.
