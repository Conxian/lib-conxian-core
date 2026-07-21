# Audit Report: SDK Extraction Viability (CON-627)

> **Historical snapshot:** This report records the pre-extraction architecture
> considered by CON-627. It is not the current ownership model. Production
> signing, attestation, and policy flows now belong to `conxius-enclave-sdk`;
> `lib-conxian-core` owns protocol primitives and invariants, and
> `lib-conxian-core-enclave` is only a compatibility adapter.

## 1. Historical Architecture Map
At the time of this report, the core signing and policy logic was described as
distributed across:
- `src/wallet.rs`: Basic secp256k1/k256 signing and key management.
- `src/sdk_primitive.rs`: High-level Vault SDK with policy enforcement (max amount, allowlist).
- `src/musig2.rs`: BIP327-compliant key aggregation.

## 2. Extraction Viability
### 2.1 Hardware-Signing Isolation (historical)
The report described the `Wallet` struct as a backend-agnostic signing
interface prepared for StrongBox TEE integration.
- **Viability:** HIGH.

### 2.2 Policy Enforcement (historical)
The report described the `SigningPolicy` and `VaultSDK` in
`src/sdk_primitive.rs` as decoupled from UI concerns.
- **Viability:** HIGH.

### 2.3 Chain Adapters
Chain-specific logic is isolated in `src/bitcoin/`, `src/stacks/`, and `src/lightning/`.
- **Viability:** MEDIUM. Some coupling exists in `core primitives` which should be moved to the SDK core if it's to be reusable.

## 3. Risks & Recommendations
- **Historical risk:** UI/UX logic was out of scope, while the report attributed
  orchestration concerns to Core primitives.
- **Superseded recommendation:** The production SDK boundary is now formalized
  by `conxius-enclave-sdk`; runtime orchestration belongs to the Gateway and
  downstream applications rather than Core.

## 4. Conclusion
Extraction of a sellable SDK primitive is viable. The codebase already follows a modular pattern that separates protocol primitives from service routing.
