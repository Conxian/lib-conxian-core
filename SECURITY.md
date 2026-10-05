# Security Policy

## Overview

`lib-conxian-core` is the primary strategic protocol core for the Conxian ecosystem, providing core cryptographic primitives, cross-chain adapters, intent selection logic, covenant scripts, and enclave security boundaries. Security is a foundational design requirement for all protocol components.

---

## Supported Versions

Only the current active release line is supported with security updates and vulnerability patches.

| Release Line | Supported | Notes |
| :--- | :--- | :--- |
| **v0.3.x** | :white_check_mark: **Yes** | Active stable line (`v0.3.3`) |
| **v0.2.x** | :x: No | EOL (End of Life) |
| **< v0.2.0** | :x: No | EOL |

---

## Reporting a Vulnerability

If you discover a potential security vulnerability within `lib-conxian-core` or associated ecosystem components, please report it privately to our security team.

### Primary Contact Channel

- **Email**: `security@conxian.org`
- **PGP Key / Encryption**: Available upon request via `security@conxian.org`

**Do NOT open public GitHub issues or discussions for security vulnerabilities.**

### Submission Guidance

When reporting a vulnerability, please include:
1. **Description**: Clear description of the vulnerability and potential impact.
2. **Reproduction Steps**: Step-by-step instructions or Proof of Concept (PoC) code demonstrating the issue.
3. **Affected Components**: Specific modules, crates, or files involved.
4. **Environment**: Rust toolchain version, target platform, and configuration.

---

## Response SLAs

We are committed to responding to security reports promptly and transparently:

- **Acknowledgment**: Within **48 hours** of initial receipt.
- **Triage & Initial Assessment**: Within **5 business days**.
- **Status Updates**: Every **7 days** until resolution or release of patch.
- **Public Disclosure**: Coordinated disclosure after fix validation and patch deployment (typically 30-90 days depending on severity).

---

## Core Security Invariants

All code in `lib-conxian-core` must adhere strictly to the following non-negotiable architectural security invariants:

1. **Zero Secret Egress (ZSE)**: Private keys, seed phrases, and unencrypted witness secrets must never cross enclave or security boundaries in plaintext.
2. **Fail-Closed Verifiers**: All validation, signature verification, intent evaluation, and cryptographic assertions must default to strict rejection (fail-closed) on invalid, missing, or malformed inputs.
3. **Transport Neutrality**: Core protocol logic must remain completely decoupled from transport layer implementations and network I/O.
4. **Deterministic Protocol Execution**: Verification logic must yield identical results regardless of runtime environment or platform architecture.

---

## Legal & Sovereign Autonomous Business (SAB) Notice

🛡️ **Sovereign Autonomous Business (SAB)**. LEGAL REVIEW REQUIRED: Any timelines or remediation commitments must be reviewed by legal counsel before inclusion in contractual language.
