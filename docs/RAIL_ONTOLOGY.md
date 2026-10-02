# Rail Ontology (G6)

This document fixes the terms that were previously conflated under "rails". It
is the single source of truth for the settlement topology across the Rust chain
and the TypeScript product surface.

## Three distinct concepts

| Term | Type | Location | Definition |
| :--- | :--- | :--- | :--- |
| **Settlement rail** | `SettlementRail` (enum, 8) | `lib-conxian-core/src/fee.rs` | The destination chain a settlement **settles on**. |
| **Bridge rail** | `bridges` module (6 transports) | `conxius-enclave-sdk/src/protocol/bridges` | The cross-chain transport that **moves value**. |
| **Chain id** | `Chain` (enum, 80+) | `lib-conxian-core/src/control_model/trust.rs` | The canonical chain **identifier**. |

## Settlement rails (8 destinations)

Statechain, sBTC, RGB, Babylon, Fedimint, Lightning, AlexStacks, EVM/ERC-8183.

## Bridge transports (6)

Bisq, Boltz, Changelly, NTT, Wormhole, x402.

## `TrustTier` disambiguation

Two `TrustTier` enums previously shared a name:

| Enum | Location | Meaning | Variants |
| :--- | :--- | :--- | :--- |
| `TrustTier` | core `control_model` + market SDK | **Attestation** trust level | `Strict` / `Managed` / `Expedient` / `ObserverOnly` |
| `RailTrustTier` | enclave-sdk `bridges` | **Settlement rail** security tier | `T1` / `T2` / `T3` / `T4` |

They map 1:1 (`T1↔Strict`, `T2↔Managed`, `T3↔Expedient`, `T4↔ObserverOnly`)
via `conxius-enclave-sdk/src/protocol/control_model_adapter.rs`.

## How a settlement flows

```
intent ──> bridge rail (transport) ──> settlement rail (destination) ──> fee (ADR-004)
```

## Rename history (G6)

- enclave-sdk `protocol/rails` → `protocol/bridges` and `rails::TrustTier`
  (T1–T4) → `bridges::RailTrustTier` — v2.1.0 (PR #423).
- core `sdk-rails` feature → `sdk-bridges`.
