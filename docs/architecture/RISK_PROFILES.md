# Canonical Chain Risk Profiles (CORE-007 / GitHub #177)

## Purpose and ownership

`lib-conxian-core` owns a versioned schema for **static protocol metadata**. It does not collect
market data, observe chain health, calculate live risk, or choose a runtime route. Nexus owns live
observation and verification evidence; Gateway owns orchestration, routing, persistence, and
service policy. Wallets, adapters, and other consumers may use the core contract, but must not
interpret a static profile as a live `VerificationStatus`.

The checked-in initial set is `data/risk_profiles/v1.json`:

- Schema version: `1`
- Profile-set version: `1.0.0`
- Effective date for every initial profile: `2026-07-21`
- Coverage: all 6 `ChainFamily` values and all 23 current `Chain` variants
- Decision: every profile is explicitly `not_assessed`
- Governance reference: `github:Conxian/lib-conxian-core#177`

The issue and repository documentation are governance/change references, not empirical risk
evidence. The initial artifact therefore has empty `evidence_refs` and does not promote example
scores from the roadmap, adapters, or other planning material.

## Schema-v1 contract

The Rust API is re-exported from `control_model` and is intentionally additive. The main types are
`RiskProfile`, `RiskProfileAssessment`, `RiskDimensions`, `RiskMetricValue`, `RiskScore`,
`RiskTarget`, `StaticPolicyAssumptions`, `VersionedRailMetadata`, and
`CanonicalRiskProfileSet`.

| Field | Meaning and validation |
| --- | --- |
| `schema_version` | Numeric schema version. Only `1` is accepted; unsupported versions fail closed during deserialization. |
| `profile_set_version` | Valid SemVer with major version `1`; `1.1.0` is compatible, while `0.x`, `2.x`, and malformed values are rejected. Every profile must match the set value. The embedded artifact is pinned to `CANONICAL_RISK_PROFILE_SET_VERSION` (`1.0.0`). |
| `target` | `family` or `chain` identity. A chain target must carry the canonical `ChainFamily` mapping. |
| `profile_revision` | Positive per-target revision. Increment it for a change to that target's approved metadata. |
| `effective_date` | Calendar date in `YYYY-MM-DD`; malformed dates are rejected. |
| `governance_ref` | Required change, approval, or review reference. It must not be confused with evidence. |
| `evidence_refs` | Typed `{ "kind", "reference" }` references supporting an assessed or partially assessed decision. Kinds are `specification`, `audit`, `research`, and `observation`; governance/change references are intentionally excluded and belong in `governance_ref`. Exact duplicate entries and blank references are rejected. They may be empty for `not_assessed`. |
| `rationale` | Required explanation for the current decision, including why a profile remains not assessed. |
| `assessment` | Explicit dimension states plus an explicit overall state/band. Missing dimensions are rejected. |
| `static_policy` | Optional typed trust/verification/finality assumptions; never a live observation. |
| `rail_metadata` | Optional schema-v1 wrapper around the legacy `RailMetadata` shape with family validation. |

### Score units and polarity

`RiskScore` is a unitless integer in the inclusive range `0..=100`. A score of `0` is a valid
assessed minimum and is never used to mean unknown. Unknown and not-assessed values are separate
wire states.

All six dimensions use the same **strength** polarity: `0` is the weakest assessed strength and
`100` is the strongest assessed strength.

| Canonical field | Legacy field | Meaning of a higher score |
| --- | --- | --- |
| `data_availability_score` | `da_score` | Stronger assessed availability guarantees |
| `settlement_score` | `settlement_score` | Stronger assessed settlement assurance |
| `bridge_score` | `bridge_score` | Stronger assessed bridge controls |
| `exit_mechanism_score` | `exit_mechanism_score` | Stronger assessed exit guarantees |
| `operator_dependency_score` | `operators_score` | Greater operator independence/resilience; **not** greater raw operator dependency |
| `decentralization_score` | `decentralization_score` | Stronger assessed decentralization |

The canonical API uses `operator_dependency_score` to make the legacy dimension explicit. Its
polarity is deliberately normalized so that higher is better. Raw operator counts, liveness,
liquidity, outage rates, and other observations remain live Nexus/Gateway data.

The overall band is represented by an opaque `RiskBand` label. Core does not invent or interpret a
universal band vocabulary or threshold mapping; an approved governance process must define any
label before it appears in an assessed profile.

### Explicit assessment states

Each metric is encoded as one of:

- `{"state":"assessed","score":0}` through `{"state":"assessed","score":100}`
- `{"state":"unknown","reason":"..."}` when the dimension is relevant but current evidence is inconclusive
- `{"state":"not_assessed"}` when no approved assessment has been made

The aggregate `status` is validated against all six dimensions and `overall`:

| Status | Required dimensions | Overall state | Evidence |
| --- | --- | --- | --- |
| `assessed` | All six assessed | `assessed` with an opaque band | At least one evidence reference |
| `partially_assessed` | At least one assessed and at least one unknown/not assessed | `unknown` with a reason | At least one evidence reference |
| `unknown` | All six unknown | `unknown` with a reason | Optional |
| `not_assessed` | All six not assessed | `not_assessed` | Optional; empty in the initial set |

Mixed states that do not match one of these rules are rejected. Required fields are not defaulted
when omitted.

## Decode boundary and compatibility policy

The public JSON decode boundary is fail-closed: `serde_json::from_*::<RiskProfile>()`,
`::<CanonicalRiskProfileSet>()`, and the directly deserializable component types enforce their
documented invariants before returning a value. Schema-v1 wire structs reject unknown or typo
fields. Additive fields therefore require an explicit schema compatibility decision and, when the
wire contract changes incompatibly, a schema-version bump. The documented `RiskDimensions`
aliases (`da_score` and `operators_score`) remain accepted for source compatibility but serialize
using the canonical field names.

`profile_for_target()` validates a supplied chain target and compares the complete target identity;
an invalid chain/family pair returns `None` rather than falling through to a same-chain profile.

## Static policy assumptions

`StaticPolicyAssumptions` contains only the existing `TrustTier`, `VerificationClass`, and
`FinalityClass` types. It calls `validate_trust_tier_policy`:

- `Strict` requires `LightClient` verification.
- `Managed` may use the existing approved non-strict verification classes.
- `Expedient` is production-allowed under the existing policy and may use the existing verification classes.
- `ObserverOnly` is rejected for production policy.
- No universal trust-tier/finality combination rule is invented here.

These assumptions describe a profile's declared protocol posture. They do not assert that a live
provider is currently verified, healthy, available, or eligible for a route. Live status belongs to
Nexus/Gateway and uses the existing verifier contracts and `VerificationStatus` where appropriate.

## Targets and compatibility

Family targets cover the six taxonomy values. Chain targets cover the 23 current variants. A chain
target with a mismatched family is invalid even if the profile otherwise parses.

`RiskAssessment` and `RailMetadata` in `control_model::trust` remain legacy, unversioned
compatibility types. They retain their existing fields, JSON shape, and Rust source shape without
deprecation attributes. New canonical consumers should use schema-v1 types. If rail metadata is
embedded in a canonical profile, it must use `VersionedRailMetadata { schema_version: 1, metadata:
RailMetadata }`, and its `rail_family` must match the profile target. The wrapper is a compatibility
boundary, not a migration of the legacy wire shape.

## Accessor and downstream field ownership

`canonical_risk_profile_set()` parses and validates the checked-in artifact once. It is a compile-
time embedded data source, not network or filesystem I/O.

| Field/contract | Core owns | Nexus consumes/owns | Gateway consumes/owns | Wallet/adapters consume |
| --- | --- | --- | --- | --- |
| Schema, set version, revision | Wire shape and validation | Cache/track compatible revisions | Expose compatible API versions | Pin compatible crate/data versions |
| Target and chain-family mapping | Canonical mapping and mismatch rejection | Select observation scope | Select service scope, never infer a route from target alone | Validate signer/adapter target identity |
| Six scores and overall status | Units, bounds, state consistency | Compare with live evidence; do not silently overwrite | Combine with live policy inputs | Display or pass through only when labeled static |
| Evidence references and rationale | Required fields and assessed-state rules | Produce/verify empirical evidence | Persist/audit evidence links | Preserve provenance in downstream records |
| Static policy assumptions | Existing trust-tier invariant | Check against verifier capabilities | Apply separate runtime policy | Reject incompatible adapter declarations |
| Rail metadata wrapper | Version and family reconciliation | Observe rail behavior separately | Route/persist only under Gateway policy | Use legacy fields only through the wrapper |

Core does not calculate a route, rank rails, or turn a profile score into a transaction decision.

## Profile change review process

Every approved profile change must be reviewable as one coordinated change. The data artifact,
affected `profile_revision`, `profile_set_version` when the set changes, governance/change
reference, evidence references, focused validation tests, documentation, and release notes must
change together. A Rust constant or adapter edit alone is not a canonical profile change.

Reviewers must confirm:

1. The target is exact and its chain-family mapping is valid.
2. Every score has the documented unit and polarity; zero is not being used as unknown.
3. Assessed/partial states have evidence and provenance; not-assessed records have a rationale and
   governance reference.
4. The canonical artifact still has exact six-family/23-chain coverage with no duplicates.
5. Downstream ownership remains static-core versus live-Nexus/Gateway, with no runtime I/O in core.
6. JSON compatibility, schema/version handling, and release notes are updated together.
