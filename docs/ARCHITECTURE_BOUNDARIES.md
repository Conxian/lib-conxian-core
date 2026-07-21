# Conxian Repository & SDK Boundaries (CON-555 / CON-700)

This document clarifies ownership and responsibility boundaries between core libraries and service layers in the Conxian ecosystem.

The deterministic, offline contract-test layer that exercises these boundaries is documented in
[`docs/INTEGRATION_TESTING.md`](INTEGRATION_TESTING.md). It uses only test-local doubles and does
not add runtime, network, persistence, or cross-repository dependencies.

## 1. Core Primitives (`lib-conxian-core`)

**Role**: Root dependency for protocol-bearing components.

**Responsibilities (managed in independent repository)**:
- Canonical data models (e.g., `StateProposal`, `PartnerLead`).
- Low-level cryptographic primitives (MuSig2, BitVM2 proof verification).
- Shared financial and yield metrics logic.
- Job Card Schema (CJCS) specifications.
- Shared wallet models and generic signing interfaces.
- Shared control-model primitives for:
  - wallet authority classes,
  - protected-action lifecycle states,
  - trigger and pending-action state modeling,
  - timelock/quorum invariant descriptors and validators,
  - signed envelope descriptors and replay/idempotency helpers,
  - session trust/security claim types,
  - adapter-facing traits for intent authorization and session issuance.
- The versioned, platform-neutral BIP-110 transaction preflight request/result contract. Core owns
  byte-measurement semantics, context support declarations, checked wire-to-shape conversion, and
  deterministic fail-closed findings; transaction-aware adapters own parsing and classification.
- The platform-neutral `UniversalChainSigner` contract and its canonical request,
  response, capability, address, verification, and secret-safe error models;
  see [docs/SIGNING_ARCHITECTURE.md](SIGNING_ARCHITECTURE.md).
- Platform-neutral protocol verification contracts and canonical proof,
  block-reference, finality, capability, provenance, and typed-error models.
- Versioned static chain-family risk-profile metadata, public evidence and
  governance references, and fail-closed invariants; see
  [docs/architecture/RISK_PROFILES.md](architecture/RISK_PROFILES.md). Core does not
  own live risk scoring or routing policy.

**Constraints**:
- Must NOT contain hardware-specific or enclave-specific implementation logic.
- Must NOT contain provider-specific standalone gateway runtime logic, transport clients, or persistence adapters.
- May define interfaces/traits for integration points, but runtime implementations stay outside core.
- Must remain platform-agnostic and audit-ready.
- Must avoid "dumping ground" growth: if behavior depends on environment, tenancy, provider APIs, or workflow orchestration, it belongs to standalone Conxian Gateway/Platform.

## 2. Secure Enclave SDK (`lib-conclave-sdk`)

**Role**: Implementation layer for secure execution environments and trusted hardware.

**Responsibilities (managed in independent repository)**:
- TEE (Trusted Execution Environment) integration logic (e.g., StrongBox).
- Remote attestation verification and proof generation.
- Secure key management and enclave-bound signing.
- Biometric auth and Passkey (FIDO2) implementation.

**Constraints**:
- Depends on `lib-conxian-core` for data models and protocol rules.
- Contains the "How" of secure execution, while Core contains the "What".

## 3. Unified standalone Conxian Gateway (`conxian-gateway`)

**Role**: Single entry point for sovereign services and protocol routing.

**Responsibilities (managed in independent repository)**:
- Unified REST API and MCP server.
- Protocol monitoring and TVL aggregation.
- Live compliance/risk observation and policy inputs; static canonical risk
  metadata remains a Core contract and is not a live market score.
- Runtime/provider implementations that satisfy core integration traits.
- Routing requests to external Bitcoin layers and sidechains.

## 4. Interaction Map

1. **standalone Conxian Gateway** uses **lib-conxian-core** for state and control-model types.
2. **standalone Conxian Gateway** implements runtime adapters and provider workflows against core traits.
3. **Wallet** uses **lib-conclave-sdk** for enclave-anchored signing.
4. **lib-conclave-sdk** uses **lib-conxian-core** to ensure signed intents align with protocol rules.

## 5. Core-vs-standalone Conxian Gateway guardrail (CON-700)

Use this decision rule when adding new capability:

- **Core (`lib-conxian-core`)**: canonical types, state machines, invariant validation, and interface contracts.
- **standalone Conxian Gateway (`conxian-gateway`)**: runtime orchestration, persistence, provider integrations, retries, observability, and external side effects.

If a change needs network IO, database access, deployment/environment configuration, or provider-specific branching, it should not land in core.

The `ProtocolVerifier<B>` ownership boundary and examples are documented in
[`docs/architecture/PROTOCOL_VERIFIER.md`](architecture/PROTOCOL_VERIFIER.md).
Runtime implementations provide the lower-level `ProtocolVerifierBackend`
hooks, while consumers use only the façade so capability, request, result, and
postcondition validation cannot be bypassed. Core's evidence-binding hash
provides structural consistency, not cryptographic authenticity or production
readiness.

### Canonical static risk-profile ownership

Core owns the additive, explicitly versioned static risk-profile schema and its
invariants through the artifact-backed `control_model::risk` module. Nexus owns
live proof, finality, and freshness observations.
Gateway owns runtime routing and policy decisions. A static risk profile is
neither a live market score nor a routing decision. Unknown, not-assessed, or
stale metadata must fail closed in downstream consumers. See
[`docs/architecture/RISK_PROFILES.md`](architecture/RISK_PROFILES.md).

### BIP-110 preflight ownership

`Bip110PreflightRequest`, `Bip110PreflightResult`, and their findings are interface contracts, not
runtime orchestration. Core owns the API version, pre-construction versus post-serialization phase
labels, explicit measurement provenance, authoritative byte units, the generic fully-classified
`bitcoin_transaction` context, the separate fixed-width Taproot control-block size field, and
stable errors for missing data, phase/source mismatches, unknown contexts, or unsupported contexts.
Core also composes ordinary measurements with enabled `Bip110Compliance` instead of maintaining a
second set of ordinary limits.

Core does **not** parse or build transactions, serialize scripts, identify Taproot annexes or
control-block position/shape, validate control-block commitments or cryptography, compile
Miniscript, execute Tapscript, validate DLC semantics, sign, estimate fees, broadcast, persist
UTXOs, or infer network deployment state. Those responsibilities remain with transaction-aware
adapters and the owning SDK, Wallet, Gateway, or Nexus layers. Pre-construction is caller-classified
planning metadata rather than full serialized transaction validation. Empty vectors are valid only
for an explicitly present supported generic context with zero constrained occurrences; missing data
or an unknown/unsupported context fails closed.

The contract is a downstream handoff for SDK #179, Gateway #245, and Wallet #381. This repository
does not claim that those consumers currently enforce the contract; their builders and routing
flows must reject non-compliant or unsupported results rather than treating them as warnings.

### Canonical risk-profile ownership (CORE-007)

`RiskProfile`, `RiskProfileAssessment`, `RiskScore`, `RiskTarget`, and
`CanonicalRiskProfileSet` are static protocol metadata contracts. Core owns their schema-v1 wire
shape, score units and bounds, explicit assessed/unknown/not-assessed states, provenance rules,
exact six-family/23-chain coverage, chain-family mismatch validation, trust-tier policy validation,
and the checked-in `data/risk_profiles/v1.json` artifact. The artifact is compile-time embedded;
core performs no network I/O, provider lookup, persistence, observation, or route selection.

Nexus owns live chain observation, verification evidence, and comparison of current conditions with
the static profile. Gateway owns runtime risk assessment, policy composition, persistence, and
routing. Wallets and adapters may validate and preserve static target/policy metadata, but must
not treat it as a live `VerificationStatus` or silently promote a not-assessed profile to a score.

The six dimensions are unitless `0..=100` strength scores. Zero is a valid assessed minimum;
unknown and not-assessed are explicit wire states. Assessed and partially assessed profiles require
typed evidence references (`specification`, `audit`, `research`, or `observation`); governance/change
references are separate and cannot substitute for empirical evidence. Not-assessed profiles require
a governance/change reference, effective date, and rationale. Public schema-v1 JSON decoding is
fail-closed for semantic invariants and unknown/typo fields; additive fields require a schema
decision/version bump. The legacy `RiskAssessment` and `RailMetadata` shapes remain unchanged for
source and JSON compatibility; canonical rail metadata uses a versioned wrapper with target-family
validation. See [`architecture/RISK_PROFILES.md`](architecture/RISK_PROFILES.md).

Profile changes are reviewable data/API changes: artifact, profile revision, set version when
applicable, evidence/change reference, tests, documentation, and release notes must change
together.

### Deterministic core-to-downstream fixture boundary (CON-1505)

Core owns the synthetic golden fixtures in `tests/fixtures/` and the
repository-local harness in `tests/core_to_downstream_integration.rs`. The
fixtures deserialize into the existing public signing, verifier, BIP-110, and
adapter contract types where those types exist, then exercise deterministic
round trips, version constants, capability gates, request/result postconditions,
and fail-closed error shapes.

The harness uses clearly named test-only doubles for successful signing and
verification, unsupported capabilities, malformed proofs, stale evidence, and
policy rejection. These doubles never acquire evidence, access a node, use
credentials, hold secret material, or stand in for hardware. Adapter coverage
is limited to representative `TxParams`, address, family, trust, and fee
metadata; it intentionally does not assert that an adapter's placeholder proof
method is authoritative cryptographic verification.

This is the initial Core-owned serialized-contract checkpoint. Its known
compatibility assumptions are limited to Core `0.3.0`, Rust `1.85`, the Core
default feature set (`default = []`), signing API version `1`, BIP-110 preflight
API version `1`, and protocol-verifier evidence binding version `1`. The
optional `conxius-enclave-sdk` `2.0.11` reference is recorded as a dependency
assumption only; it is not compiled by this layer, and no optional SDK or
`--all-features` path is enabled. This checkpoint intentionally does not claim
direct compile compatibility or revision pins for `conxius-enclave-sdk`,
`conxian-gateway`, or `conxian-nexus`. Pinned downstream fan-out is deferred
until the UCS and ProtocolVerifier APIs stabilize, so no Gateway or Nexus pin is
asserted here. Downstream repositories remain responsible for runtime
orchestration, parsing/classification, live evidence, cryptography, persistence,
and external side effects.

Local verification starts with `cargo fmt --all -- --check` and
`cargo test --locked --test core_to_downstream_integration`; adjacent Core tests
and the full workspace command remain follow-up checks. Live downstream CI
fan-out is deferred and must remain opt-in and pinned until the APIs stabilize.


## 6. SDK Ownership & Version Policy (CON-1178)

### 6.1. Canonical Ownership
- **Shared Core (`lib-conxian-core`):** Protocol-bearing primitives, canonical data models, and platform-agnostic crypto. Owned by the Protocol Team.
- **Secure Enclave (`lib-conclave-sdk`):** TEE-specific implementations and hardware-bound signing. Owned by the Security Team.
- **Gateway-Local:** Provider-specific orchestration and temporary integration shims. Owned by the Infrastructure Team.

### 6.2. Version Policy
- **Protocol Core:** SEMVER-compliant releases. Breaking changes to canonical models require a 2-week deprecation notice in `CHANGELOG.md`.
- **Enclave SDK:** Beta/RC dependencies are allowed but must be pinned to exact revisions.
- **Stacks JS / Clarinet:** Standardized on v7.3+ family across all production surfaces.

### 6.3. Consumption Guidance
- **Production Apps:** Must consume `lib-conxian-core` via crates.io or pinned Git tags.
- **Local Integrations:** Use repo-local code for experimental rails only. Once a rail reaches T2 (Managed) maturity, its models must be upstreamed to Core.

### 6.4. Release Posture
- **Reusable SDKs:** (Core, Enclave) must have tagged GitHub releases and maintained changelogs.
- **App-Layer:** (Gateway, Wallet, UI) are deployment-tracked. Main branch state determines production status.
