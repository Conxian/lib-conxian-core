# Security & Implementation Advisory (2026-07-06)

> **Historical context:** This report is a July 6, 2026 snapshot of reported
> work. Its completion wording is not a claim that current
> `lib-conxian-core` APIs provide production verification. For the named
> verifier paths, it is superseded by the CON-1509 fail-closed boundaries in
> the v0.3.1 release; see [`VERIFIER_INVENTORY.md`](VERIFIER_INVENTORY.md).

## 1. Cryptographic Hardening Audit
- **MuSig2 (`src/musig2.rs`):** Verified real scalar-sum signature aggregation. COMPLETED.
- **FROST (`src/protocol/frost.rs`):** Historical report context described Round 2 distribution work. Current Core share generation, distribution, and aggregation remain a typed `Unsupported` boundary until an audited implementation is supplied; production FROST belongs in `conxius-enclave-sdk`.
- **Enclave (`src/enclave/mod.rs`):** Core can parse a DER container, but that is not certificate-chain or hardware-attestation verification. Production hardware attestation belongs in `conxius-enclave-sdk`.
- **Fedimint (`src/fedimint/mod.rs`):** Core provides deterministic secp256k1 point reconstruction/blinding primitives only, not provider-backed mint, note, or status verification. Authenticated mint status is unavailable without a provider.

## 2. Protocol Gaps
- **Silent Payments:** Scanning logic in `src/bitcoin/mod.rs` hardened. Performs real summation of input public keys and shared secret derivation via ECC point multiplication (sum(P_in) * d_scan) to align with BIP-352 scanning requirements. COMPLETED.
- **DLC:** Verification in `src/protocol/dlc.rs` hardened. Implemented real oracle attestation verification (s*G = R + e*P). COMPLETED.
- **BIP-322:** Core performs address/base64/witness shape handling and returns typed `Unsupported` before cryptographic script or signature verification. Production verification requires an audited provider.
- **RGB:** Implementation expanded in `src/rgb/mod.rs`. Introduced `RGBStockAdapter` for client-side validation, supporting future `rgb-std` Stock persistence. COMPLETED.
- **Fuzz Testing:** Four-target cargo-fuzz regression coverage is defined in [`docs/FUZZING.md`](FUZZING.md) and runs weekly or manually for intent parsing, MuSig2 aggregation, anchoring receipt deserialization, and proof-request deserialization plus structural validation; when an optional proof envelope is present, its fail-closed contract and policy validation also runs. The proof-request target does not claim cryptographic BitVM2 proof verification. COMPLETED.

## 3. Recommended Actions
1. **Security Audit (CON-1333):** Engage external audit for production cryptographic implementations. The historical "ALL STUBS RESOLVED" wording does not apply to Core's FROST, hardware-attestation, BIP-322, or Fedimint provider boundaries.
2. **BitVM3 (CON-1337):** Advance garbled circuit research into implementation for recursive proof aggregation.
3. **Universal Adapters:** Extend state root verification for Solana and Move to use real TEE-attested light client proofs.
