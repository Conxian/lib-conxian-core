# Security & Implementation Advisory (2026-07-06)

> **Superseded for the named verifier paths by CON-1509 / GitHub #188.**
> This historical advisory must not be read as evidence that the in-core
> BIP-322, universal state-proof, RGB, FROST, or shallow DLC entry points are
> production verifiers. Those boundaries now fail closed until a complete
> audited backend is wired in.

## 1. Cryptographic Hardening Audit
- **MuSig2 (`src/musig2.rs`):** Verified real scalar-sum signature aggregation. COMPLETED.
- **FROST (`src/protocol/frost.rs`):** Historical placeholder claim superseded; the core boundary now returns typed unsupported and production FROST remains SDK-owned.
- **Enclave (`src/enclave/mod.rs`):** Verified X.509 DER parsing using the `der` crate. COMPLETED.
- **Fedimint (`src/fedimint/mod.rs`):** Hardened with real ECC blinding (note = H(secret) * G + r * G). COMPLETED.

## 2. Protocol Gaps
- **Silent Payments:** Scanning logic in `src/bitcoin/mod.rs` hardened. Performs real summation of input public keys and shared secret derivation via ECC point multiplication (sum(P_in) * d_scan) to align with BIP-352 scanning requirements. COMPLETED.
- **DLC:** The real oracle attestation equation (s*G = R + e*P) remains verified, while shallow execution is typed unsupported and a bound outcome/collateral/expiry helper is available.
- **RGB:** Adapter transition/seal checks fail closed; Shadow mode is explicitly non-authoritative and cannot authorize production execution.
- **Fuzz Testing:** Four-target cargo-fuzz regression coverage is defined in [`docs/FUZZING.md`](FUZZING.md) and runs weekly or manually for intent parsing, MuSig2 aggregation, anchoring receipt deserialization, and proof-request deserialization plus structural validation; when an optional proof envelope is present, its fail-closed contract and policy validation also runs. The proof-request target does not claim cryptographic BitVM2 proof verification. COMPLETED.

## 3. Recommended Actions
1. **Security Audit (CON-1333):** Engage external audit for MuSig2 and BitVM2 implementations, and complete the downstream verifier work for the typed-unsupported boundaries introduced by CON-1509. Named in-core placeholder success paths are not considered resolved by this advisory.
2. **BitVM3 (CON-1337):** Advance garbled circuit research into implementation for recursive proof aggregation.
3. **Universal Adapters:** Extend state root verification for Solana and Move to use real TEE-attested light client proofs.
