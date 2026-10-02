# Fee Model Binding & Deprecation Plan (ADR-004)

## 1. Canonical Source of Truth

`src/fee.rs` is the **canonical** ADR-004 settlement fee model:

- `SettlementRail` (8 settlement rails) + `VolumeDecayTier` (4 tiers).
- Rail flat floors, logarithmic volume decay, clamped + quantised load factor.
- `calculate_dynamic_fee` (with `ObserverOnly` rejection + 50/30/20 distribution).

The TypeScript `@conxian/market-sdk` `calculateDynamicFee` is a **mirror** of this
implementation and must not diverge.

## 2. Cross-Repo Conformance Contract

`fixtures/fee_conformance.json` is the **behavioral contract**. It MUST stay
byte-identical between:

- `lib-conxian-core` → `fixtures/fee_conformance.json`
- `@conxian/market-sdk` → `tests/fixtures/fee_conformance.json`

Both repos execute a conformance test against the same vectors:

- **Rust** — `src/fee.rs` `fee_conformance_vectors`.
- **TypeScript** — `tests/fee_conformance.test.ts`.

Any change to the fee model must update the fixture in **both** repos and keep
both tests green.

## 3. Binding (current → future)

- **Current** — *behavioral binding* via the shared conformance fixture. No
  runtime cross-language dependency; drift is caught by the conformance tests.
- **Future** — *type-level binding* once the market SDK is published and consumed:
  - **JSON Schema codegen** (`schemars` on `src/fee.rs` → `json-schema-to-typescript`
    zod/types) — recommended, keeps the Rust types as the single source.
  - **wasm-bindgen** (compile `src/fee.rs` to WASM) — heavier, adds a wasm target.

## 4. Deprecation Plan

| Surface | Action | When |
| :--- | :--- | :--- |
| `@conxian/market-sdk` `src/fee_calculator.ts` | Keep as the consumable TS surface; annotate as a mirror of `src/fee.rs`; keep conformance-tested | now |
| same | Replace `calculateDynamicFee` with a thin wrapper over the generated binding | after type-level binding lands |
| `conxian-nexus` `src/api/billing` (CON-24) | Keep — it is subscription *signature-limit* tiers, a distinct concern; align the "enterprise cap" semantics with `FeeOptions.enterprise_subscription_cap` | now |
| fixture drift | Add a CI cross-check (or gitlink) asserting the fixture is byte-identical across repos | follow-up |

## 5. Acceptance (G4)

- [x] Canonical fee-model types in core (`src/fee.rs`).
- [x] Behavioral Rust↔TS binding via the shared conformance fixture.
- [x] Deprecation plan for the duplicated TS / nexus billing surfaces (this doc).
