# Security Policy

`lib-conxian-core` provides shared protocol, verification, and cryptographic state-machine primitives for the Conxian ecosystem. Security and fail-closed correctness are core requirements for all code and governance policies in this repository.

## Supported Versions

Only the current stable release line (`0.3.x`) receives security patches and active support.

| Version Line | Supported | Status | Notes |
| ------------ | --------- | ------ | ----- |
| `0.3.x`      | ✅         | Active | Current release line (v0.3.2 stable) |
| `< 0.3.0`    | ❌         | End of Life | Deprecated; upgrade to `0.3.x` |

Downstream consumers are strongly advised to keep their dependency on `lib-conxian-core` updated to the latest patch release within the `0.3.x` series.

## Reporting a Vulnerability

**Do NOT report security vulnerabilities through public GitHub issues, PR comments, or public discussions.**

If you discover a potential vulnerability or security flaw, please submit a report privately using one of the following channels:

1. **GitHub Private Vulnerability Reporting:** Submit a advisory via the [Repository Security Advisory Tab](https://github.com/Conxian/lib-conxian-core/security/advisories/new).
2. **Encrypted / Direct Email:** Email `security@conxian-labs.com`.

### What to Include in Your Report

To help us triage and investigate reports efficiently, please include:

- **Component & Version:** Affected module, function, or version (e.g., `0.3.2`, `control_model`, `verifier`, or `bip110`).
- **Description:** A detailed explanation of the issue and potential security impact.
- **Reproduction Steps:** Minimal proof-of-concept (PoC) code or step-by-step instructions to reproduce the issue.
- **Suggested Remediation:** Any proposed fix or mitigation, if available.

### Response Timelines & SLA

- **Acknowledgement:** Within **48 hours** of initial receipt.
- **Triage & Initial Assessment:** Within **5 business days**.
- **Status Updates:** Regular updates every **7 days** until a patch or mitigation is published.
- **Coordinated Public Disclosure:** Security advisories and patched releases are published after fix verification, following a mutually agreed disclosure timeline.

## Security Architecture & Design Invariants

`lib-conxian-core` enforces strict architectural boundaries to eliminate attack vectors in foundational protocol primitives:

1. **Deterministic & Transport-Neutral (CON-700):** Core code in `src/` contains zero side effects, no file I/O, no network I/O (`std::net`), and no process spawning (`std::process`). Contamination guards automatically enforce this boundary in CI.
2. **Zero Secret Egress (ZSE):** Core protocol primitives do not log, persist, or expose unencrypted private key material. All signing operations requiring hardware keys are delegated to the dedicated [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) or the fail-closed [`lib-conxian-core-enclave`](addons/lib-conxian-core-enclave) adapter.
3. **Fail-Closed Verification (CON-1509):** Protocol verifiers and proof assertions reject invalid, malformed, or missing inputs by returning typed failure results rather than falling back or authorizing non-authoritative states.
4. **BIP-110 Preflight Bounds:** Fixed-width preflight measurement contracts strictly validate pushdata sizes (max 256 bytes), script pubkeys (max 34 bytes), witness elements (max 256 bytes), and op_return payloads (max 83 bytes) before transaction propagation.

## Hygiene & CI/CD Security Controls

- **Immutable Dependency Pins:** All GitHub Actions workflows in `.github/workflows/` are pinned to verified, immutable commit SHAs.
- **Automated Dependency Auditing:** `cargo-audit` scans dependency trees for known vulnerabilities on every commit and PR.
- **Forbidden Tracked Artifacts:** Automated hygiene checks (`scripts/verify_tracked_artifacts.py`) prevent tracked `.env`, secret keys, credentials, or transient build outputs.
- **Fuzz Testing:** Bounded fuzzing targets in `fuzz/` continuously test intent parsing, key aggregation, anchoring receipts, and proof validation.

## Security Advisories & Disclosure Policy

Security advisories are published through [GitHub Security Advisories](https://github.com/Conxian/lib-conxian-core/security/advisories) and reported to the Rust SecUnsound / RustSec advisory database when appropriate.
