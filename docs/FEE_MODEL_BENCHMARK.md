# Fee Model Benchmark — ADR-004 vs Industry

Cross-referenced 2026-10-02 against three established fee systems to validate
and adapt the ADR-004 dynamic fee model.

> **ADR-005 (2026-10-10):** rates and floors recalibrated from live competitor
> and per-rail cost research — volume decay 50/25/15/10 bps; on-chain floors
> raised (Statechain 125, RGB 312, sBTC 375); EVM moved to L2 (floor 75).

## Sources
- **Lightning Network** routing fees — `base_fee` (flat, msat) + `fee_rate`
  (proportional, ppm).
- **Card processing** — interchange-plus (cost + markup) vs flat-rate vs
  tiered pricing.
- **Ethereum EIP-1559** — congestion `base fee` + `priority tip`.

## Mapping ADR-004 ↔ Industry

| ADR-004 component | Industry analog | Verdict |
| :--- | :--- | :--- |
| `flat_floor` (rail default) | Lightning `base_fee` (fixed overhead) | ✅ aligned — should be *cost + margin* |
| percentage `bps` (volume-decayed) | Lightning `fee_rate` (ppm) / card interchange % | ✅ aligned |
| `VolumeDecayTier` (50 → 10 bps) | volume-based tiered discount | ✅ aligned + hysteresis (G8) |
| `system_load_factor` (1.0–3.0) | EIP-1559 `base fee` / surge pricing | ✅ aligned — needs an oracle |
| `enterprise_subscription_cap` | subscription/committed-use pricing | ✅ aligned (G5 nexus link) |

## Key learnings

1. **Settlement fee ≠ routing fee.** Lightning routing runs 1–5,000 ppm; our
   settlement fee runs 10–50 bps (= 1,000–5,000 ppm) because it prices a
   *settlement rail* (rail cost + margin), not just channel-capital opportunity
   cost. This distinction is deliberate and must not be miscategorized as
   "overpriced vs Lightning".
2. **Flat floor = base fee = fixed cost + margin** (interchange-plus). The
   current per-rail floors are **placeholders** pending measured per-rail cost
   (G8 rail calibration).
3. **bps ↔ ppm**: `1 bps = 100 ppm`. Lightning's 200 ppm ≈ 2 bps; our Tier1
   50 bps = 5,000 ppm (a settlement, not a route).
4. **Cost-plus transparency** (interchange-plus displaced opaque tiered
   pricing). The result must keep exposing `percentage_fee_sat` +
   `flat_floor_sat` + `effective_fee_sat` + `distribution`.
5. **Volume tiers reduce (never raise) rates** — our decay is monotonic
   downward (50 → 25 → 15 → 10 bps), matching industry.

## Adaptations applied

- **G8 hysteresis** — Schmitt trigger, 5% band, one tier per call (debounces
  tier boundaries the way production billing systems do).
- **Rail floors** — documented as *cost + margin* placeholders pending
  telemetry (see `rail_default_flat_floor`).
- **`enterprise_subscription_cap`** — subscription/committed-use pricing,
  linked to nexus `SubscriptionTier::Enterprise` (G5).
