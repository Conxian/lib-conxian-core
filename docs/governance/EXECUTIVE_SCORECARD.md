# Weekly Executive Operating Scorecard (CON-1271)

Compact weekly review roll-up for the Conxian Labs leadership.

## 1. Executive Summary

| Lane | Health | Blockers | Decisions Needed |
| :--- | :--- | :--- | :--- |
| **Launch** | 🟢 | None | Finalize audit engagement |
| **Repo/Gov** | 🟢 | None | Finalize CXIP-21 approval |
| **Release** | 🟢 | None | Confirm v0.3.3 stability |
| **Growth** | 🟢 | None | Select primary Babylon Finality Provider |

## 2. Key Metrics

- **Core Tests**: 268 Total Rust Workspace Tests Passing (141 Core Unit/Doc Tests + 127 Integration/Enclave/Conformance Tests; 79 Python Guard Tests)
- **Open Security Issues**: Named verifier placeholders are fail-closed under CON-1509; downstream verifier completion and external audit remain tracked follow-ups.
- **CI/CD Health**: 🟢 (Gitleaks, Dependency Review, Nitro Enclave CI, and scheduled/manual fuzz regression active)
- **Fuzz Regression Suite**: 4 bounded targets — `parse_intent`, `anchoring_receipt`,
  `musig2_aggregate`, and `proof_request_validate`. MuSig2 coverage is direct
  dependency-level key aggregation only; this repository has no PSBT, BIP-322,
  or BitVM2 fuzz target. Production BIP-322 signing/message-authenticity
  verification and BitVM2 proof verification belong to `conxius-enclave-sdk`.
- **Active CXIPs**: 3 (Drafting: 26, 27, 28)

## 3. High-Priority Actions

1. **Audit**: Execute external security audit for core cryptographic paths (CON-1333).
2. **Alignment**: Repair broken submodule pins in `conxian-business` (CON-1308).
3. **Research**: Progress BitVMX and BitVM3 research for v0.3.2 floor.
