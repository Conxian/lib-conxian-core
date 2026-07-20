# Security & Implementation Advisory (2026-07-06)

> **Scope:** This advisory records repository-boundary and fuzz-regression findings. It is not an external cryptographic audit and does not determine overall mainnet readiness.

## 1. Cryptographic Hardening Audit
- **MuSig2:** Production MuSig2 session and signing functionality is owned by `conxius-enclave-sdk`; the core fuzz suite covers direct upstream key aggregation only. ALIGNED.
- **FROST (`src/protocol/frost.rs`):** Verified Round 2 distribution with HMAC-based MACs. COMPLETED.
- **Enclave (`src/enclave/mod.rs`):** Verified X.509 DER parsing using the `der` crate. COMPLETED.
- **Fedimint (`src/fedimint/mod.rs`):** Hardened with real ECC blinding (note = H(secret) * G + r * G). COMPLETED.

## 2. Protocol Gaps
- **Silent Payments:** Scanning logic in `src/bitcoin/mod.rs` hardened. Performs real summation of input public keys and shared secret derivation via ECC point multiplication (sum(P_in) * d_scan) to align with BIP-352 scanning requirements. COMPLETED.
- **DLC:** Verification in `src/protocol/dlc.rs` hardened. Implemented real oracle attestation verification (s*G = R + e*P). COMPLETED.
- **RGB:** Implementation expanded in `src/rgb/mod.rs`. Introduced `RGBStockAdapter` for client-side validation, supporting future `rgb-std` Stock persistence. COMPLETED.
- **Fuzz Testing:** The current `fuzz/` suite has four bounded targets covering intent parsing, upstream MuSig2 aggregation, anchoring receipt deserialization, and proof-request deserialization plus fail-closed contract/policy validation. Weekly/manual CI is configured; the proof-request target does not claim cryptographic BitVM2 proof verification. COMPLETED.

## 3. Recommended Actions
1. **Security Audit (CON-1333):** Engage an external audit for the production MuSig2 and BitVM2 implementations in `conxius-enclave-sdk`. The changes reviewed here address repository-boundary and fuzz-regression findings only; they do not constitute an external cryptographic audit or an overall mainnet-readiness determination.
2. **BitVM3 (CON-1337):** Advance garbled circuit research into implementation for recursive proof aggregation.
3. **Universal Adapters:** Extend state root verification for Solana and Move to use real TEE-attested light client proofs.
