# Canonical Static Risk Profiles (CORE-007 / CON-1500)

`lib-conxian-core` owns the versioned wire schema and invariants for static
chain-family risk metadata. It does **not** own live market scoring, proof
acquisition, freshness observation, or routing policy.

| Layer | Owns | Does not own |
| --- | --- | --- |
| **Core** (`lib-conxian-core`) | Static versioned metadata, public evidence references, governance references, and fail-closed schema/invariant validation | Network I/O, live scoring, proof acquisition, freshness polling, or route selection |
| **Nexus** (`conxian-nexus`) | Live chain observations, proof/finality acquisition, and freshness state | Changing Core's canonical schema or silently converting unknown metadata into an assessment |
| **Gateway** (`conxian-gateway`) | Runtime orchestration, routing, and policy decisions | Treating a static profile as a live market score or as an implicit route authorization |

## Schema and revision policy

The additive `control_model::risk_profile` module exposes:

- `RiskProfileSchemaVersion`: the wire-contract version. The current supported
  value is `1.0`.
- `profile_revision`: the subject's data revision. It changes when approved
  metadata changes and is separate from the schema version.
- `effective_from`: the UTC date at which the profile revision becomes
  effective.
- `supersedes`: an optional prior revision and its effective date. The prior
  revision must be lower than the current revision and its effective date must
  precede the new one.
- `governance`: public decision and policy references. These are identifiers,
  not approval credentials.

An unsupported schema version fails validation and fails closed during
`CanonicalRiskProfile` deserialization. Existing `RiskAssessment`,
`RailMetadata`, and their serialized fields are unchanged; the new model is
additive rather than a replacement or reinterpretation of those public types.

## Status, score units, and evidence

`AssessmentStatus` is explicit:

- `assessed`: all six dimensions are present, in deterministic order, within
  the declared bounds; static posture and public evidence are required.
- `not_assessed`: governance has intentionally not approved an assessment;
  scores and static posture are absent.
- `unknown`: the subject or available metadata cannot currently be classified;
  scores and static posture are absent.

The six dimensions (`data_availability`, `settlement`, `bridge`,
`exit_mechanism`, `operators`, and `decentralization`) preserve the vocabulary
of the legacy `RiskAssessment` model. `RiskScoreUnit::NormalizedPoints` uses the
exact inclusive `0..=100` range, enforced by profile validation, and is only a
transport scale. It is not a probability, market score, health metric, or
routing recommendation.

An assessed profile must include public, reviewable `RiskEvidence` references
and a `GovernanceReference`. Evidence references must not contain secrets,
credentials, private endpoints, or raw sensitive data.

## Subject identity and resolution

`RiskProfileSubject` always carries the canonical `ChainFamily`; a missing
`chain` identifies a family baseline and a present `chain` identifies a
chain-specific override. Known chain/family mismatches fail validation using
the existing `Chain::family()` mapping.

`RiskProfileRegistry::resolve` uses a chain-specific entry when one exists and
otherwise falls back to the family baseline. An explicit chain-level
`unknown` or `not_assessed` entry wins over the family baseline; consumers must
not silently inherit a broader or stale assessment when a chain-specific
governance decision says that the chain is not assessed.

`RiskProfileRegistry::canonical()` covers every currently enumerated
`ChainFamily` and `Chain` value. Bitcoin, Stacks, Babylon, and Liquid are
explicit chain entries. The initial registry is deliberately all
`not_assessed`, with the governance gap recorded against public issue #177;
it contains no approved chain risk scores.

## Static posture and CORE-006 policy

An assessed profile may carry `RiskProfilePosture` with the existing
`TrustTier`, `VerificationClass`, and `FinalityClass` enums. Validation calls
the existing `validate_trust_tier_policy` helper rather than defining a second
policy matrix. In particular, `TrustTier::Strict` still requires
`VerificationClass::LightClient`.

Static posture describes declared protocol metadata only. It does not prove
that a current observation is fresh or final, and it does not authorize a
route.

## Downstream consumption and fail-closed behavior

Consumers should validate the schema and registry before use. Unknown,
not-assessed, unsupported, or stale metadata must fail closed downstream:

1. Nexus supplies current proof, finality, and freshness observations.
2. Gateway combines validated static metadata with those observations and its
   own governed policy.
3. Gateway, not Core, decides whether a route or operation is allowed.

The scoring examples formerly listed in the Phase 1 roadmap and other
planning/scoring documents are historical planning material only. They are not
canonical input and were intentionally not copied into the registry without a
separate approved governance decision and public evidence.
