# Rust and Feature Compatibility

## Supported package floor

`lib-conxian-core` declares `rust-version = "1.97.1"`. Rust `1.97.1` is the
explicit toolchain used by CI. This is a package-wide floor: Cargo exposes one
`rust-version` for the crate, so the supported floor applies to the default
and optional feature sets alike.

| Surface | Feature selection | Supported Rust | Locked dependency evidence |
| --- | --- | --- | --- |
| Package | Any published feature set | `1.97.1+` | The package metadata establishes the single supported floor. |
| Default graph | `default = []` | `1.97.1+` | The package floor is intentionally shared across every feature set. |
| Optional SDK graph | `enclave`, category features, or `full-sdk` | `1.97.1+` | Root SDK Git tag `v2.0.14` declares `rust-version = "1.97.1"`. Its stale package metadata still reports version `2.0.12`. |

The default graph may have a lower transitive minimum than the package floor,
but Rust versions below `1.97.1` are not supported for this crate release. CI
runs locked `check`, `test`, and all-target `clippy -D warnings` coverage for
both the default and all-feature graphs.

## Optional enclave SDK coordination

The root SDK features select
[`conxius-enclave-sdk`](https://github.com/Conxian/conxius-enclave-sdk) Git tag
`v2.0.14`, locked to commit
`d3adefa40b3db8ce72e4959227ff9afffcca3bc5`. The tag's manifest has a stale
package `version = "2.0.12"`, but its selected Git tag is still `v2.0.14` and
its declared Rust requirement is `1.97.1`. Hardware-backed signing,
attestation, and policy behavior remain owned by that SDK; this crate exposes
only the optional dependency and public re-export paths.

When upgrading `conxius-enclave-sdk`:

1. Review the SDK's declared MSRV and its resolved transitive graph, including
   Alloy and `ruint` versions.
2. Keep `Cargo.toml`, CI's explicit toolchain, this matrix, and the release
   notes synchronized in the same change when the supported floor changes.
3. Run the locked default and all-feature `check`, `test`, and all-target
   `clippy -D warnings` commands before publishing either side of the
   integration.

Do not confuse the tag name with the stale package version field, and do not
downgrade the Git tag, dependency graph, or re-export surface to match that
field. Any dependency change requires separate compatibility validation and
coordination with the SDK release.

## Opt-in SDK v2.0.11 compatibility evidence

The release-compatibility harness is a separate, non-published workspace package
at [`tests/sdk-compat`](../tests/sdk-compat). It is opt-in through the local
`run` feature and does not add an SDK, simulator, or mock dependency to Core's
default production graph.

### Evidence baseline

This rebased checkpoint is based on Core `origin/main` commit
`5325860499800ae440e03962605de9dd833e53e1` on **July 21, 2026**:

| Surface | Exact baseline |
| --- | --- |
| Core package | `lib-conxian-core` `0.3.1` |
| Core Rust floor | `1.97.1` (exactly `1.97.1` in CI and the harness) |
| SDK formal release | `conxius-enclave-sdk` `2.0.11` |
| SDK release tag | `v2.0.11` at commit `d3e9a6a26da1bd4c15e612ce7051a0bfdf640a83` |
| SDK release Rust floor | `1.85` as declared by the published release manifest |
| Lockfile provenance | Root `Cargo.lock`; SDK registry checksum `e35d22138325b93283bab7afeeca5a121a0d2019bd7d9d81b69af7ec46db5f2d` |

The SDK release's declared `1.85` floor is distinct from the Core package
floor: every harness invocation runs with Rust `1.97.1` because it compiles the
current Core package. The harness's direct SDK `2.0.11` release dependency is a
separate published compatibility baseline; it does not replace the root
package's Git-tagged `v2.0.14` optional dependency.

### SDK feature matrix

The published/tagged v2.0.11 manifest was checked directly. Its exact features
are:

| SDK feature name | v2.0.11 status | Harness treatment |
| --- | --- | --- |
| `default` | Supported (`[]`) | Tested as the default SDK graph |
| `mock-cloud-enclave` | Supported (`[]`) | Tested only as an explicit opt-in evidence graph |
| `dev-attestation-bypass` | Supported (`[]`) | Tested only as an explicit non-production evidence graph |
| `development-simulators` | **Not present** | Documented as unsupported; never requested |
| `bip110_compliant` | **Not present** | Documented as unsupported; never requested |

`all-supported` is a harness convenience feature that enables both supported
non-default SDK features. It is not an SDK feature name. The matrix covers Core
`default` and `enclave` against SDK `default`, `mock-cloud-enclave`,
`dev-attestation-bypass`, and their combined `all-supported` selection.

Run the complete matrix from the repository root with:

```text
cargo +1.97.1 fetch --locked
python3 scripts/run_sdk_compat.py --toolchain 1.97.1 --offline
```

The regular workspace `--all-features` CI job deliberately enables the
harness's `run` feature and exercises its DTO-only SDK feature checks. That is
a broad workspace smoke test, not a substitute for this dedicated eight-command
matrix.

The individual locked offline commands use the following shape (the script
expands the full Core/SDK matrix):

```text
cargo +1.97.1 test --offline --manifest-path tests/sdk-compat/Cargo.toml --locked \
  --no-default-features --features run,core-enclave,all-supported
```

The dedicated `.github/workflows/sdk-compat.yml` workflow runs for pull
requests targeting `main`, pushes to `main`, and manual dispatches. It first
acquires the locked dependency graph with `cargo +1.97.1 fetch --locked`, then
runs the eight matrix commands with Cargo's `--offline` flag. This proves the
matrix is network-independent at runtime after dependency acquisition; the
locked fetch step itself is the dependency-acquisition phase and may use the
network. Focused formatting and linting remain repository-local commands:

```text
cargo fmt --all -- --check
cargo +1.97.1 clippy --manifest-path tests/sdk-compat/Cargo.toml --tests \
  --locked --no-default-features --features run,core-enclave,all-supported -- -D warnings
```

### Dependency direction and ownership

```text
lib-conxian-core-sdk-compat (non-published, opt-in test evidence)
├── local lib-conxian-core 0.3.1
│   └── conxius-enclave-sdk Git tag v2.0.14 when a Core SDK feature is enabled
└── conxius-enclave-sdk =2.0.11 (direct evidence dependency)
```

The harness depends on both local Core and the exact SDK release; the SDK does
not depend on Core. This preserves the production ownership boundary and avoids
an SDK-to-Core dependency cycle. The harness has no network, database,
credential, hardware, real-key, or secret-material path. The SDK's mock and
development features are never forwarded through Core's production features.

### What the evidence proves—and what it does not

The harness provides compile-time and deterministic offline evidence for:

- explicit adapter-owned Core signing-tier requirements for SDK v2.0.11 rail
  `TrustTier` (`Strict`/`Managed`/`Expedient` require `T1`/`T2`/`T3` or
  stronger); SDK `T4` is observation-only and is not a sign-capable
  `ObserverOnly` mapping;
- serde round trips for both sides' trust types, SDK signing/attestation DTOs,
  Core signing fixtures, and Core BIP-110 preflight fixtures;
- Core canonical BIP-110 limits paired with the SDK's independent
  `MempoolPolicy` configuration in a test-local evidence record; disabled or
  non-canonical Core policies and invalid SDK policy values fail closed;
- the existing deterministic Core signer and BIP-110 fixture boundaries.

The trust mapping is an adapter-owned policy check, not an SDK or Core enum
mutation. It does not make SDK `T4` production-eligible or authorize a signing
or settlement operation. The v2.0.11 SDK exposes no
BIP-110 module, validator, or `bip110_compliant` feature. Accordingly, the
local BIP-110 evidence record is deliberately not an SDK enforcement claim;
transaction parsing, classification, policy enforcement, signing, and
attestation remain owned by their respective downstream layers.

This artifact does **not** claim production signing, settlement, hardware
attestation, WASM readiness, unconditional SDK production support, cryptographic
correctness of either implementation, or downstream runtime adoption.

### Unresolved production gates

The release baseline remains subject to the SDK's unresolved production work:

- [SDK #194 — align SDK policy types with Core control-model contracts](https://github.com/Conxian/conxius-enclave-sdk/issues/194)
- [SDK #195 — enforce hardware-backed signing and mandatory attestation](https://github.com/Conxian/conxius-enclave-sdk/issues/195)
- [SDK #198 — make CCTP, account abstraction, and asset metadata fail closed](https://github.com/Conxian/conxius-enclave-sdk/issues/198)
- [SDK #200 — harden the WASM secret boundary and add runtime/platform evidence](https://github.com/Conxian/conxius-enclave-sdk/issues/200)
- [SDK #202 — complete independent security review and release acceptance evidence](https://github.com/Conxian/conxius-enclave-sdk/issues/202)

[SDK #196](https://github.com/Conxian/conxius-enclave-sdk/issues/196) was
closed on July 21, 2026 and is not treated as an unresolved gate here; its
closure does not expand this harness's claims. Core tracking remains
[CON-1496](https://linear.app/conxian-labs/issue/CON-1496/research-expansion-and-implementation),
the Core umbrella [#173](https://github.com/Conxian/lib-conxian-core/issues/173),
and the deterministic downstream fixture work
[CON-1505](https://linear.app/conxian-labs/issue/CON-1505/core-009-build-deterministic-core-to-downstream-integration-tests).

## Companion adapter crate

`addons/lib-conxian-core-enclave` targets the exact published
`conxius-enclave-sdk =2.0.11` release. It is a separate workspace member so the
Core default feature graph remains SDK-independent and no SDK-to-Core
dependency is introduced. The companion adapter depends on both Core and the
published SDK; the SDK package itself remains standalone.

The root package and companion surfaces intentionally use different sources:
Core's optional SDK dependency selects Git tag `v2.0.14`, while the adapter and
evidence harness remain pinned to the published `2.0.11` crate. The stale
`2.0.12` package version in the `v2.0.14` manifest does not change the selected
tag.

| Surface | Feature selection | SDK target | Effective Rust | Default bypass features |
| --- | --- | --- | --- | --- |
| `lib-conxian-core-enclave` | Workspace member; no feature flags | Published `=2.0.11` | `1.97.1+` workspace floor | None; simulator/mock/dev paths are not enabled |
| Core SDK features | Optional direct SDK dependency | Git tag `v2.0.14` at `d3adefa40b3db8ce72e4959227ff9afffcca3bc5` | `1.97.1+` | None by default |

The companion adapter is intentionally narrower than the SDK. It supports
explicit algorithm conversion, a deny-by-default chain/algorithm allowlist,
deterministic derivation-path rendering, digest-only SHA-256 request
construction, conservative trust-tier gates, request-bound attestation evidence
retention, adapter-owned rail/network policy, typed Core envelope replay
binding, typed public response mapping, and Core-first BIP-110 preflight for
Bitcoin signing. Provider lifecycle, cryptographic signature/attestation
verification, replay storage/cache TTL, networking, persistence, telemetry, and
environment-specific policy remain outside Core and this adapter. Core's
canonical BIP-110 validator remains authoritative. See
[`addons/lib-conxian-core-enclave/README.md`](../addons/lib-conxian-core-enclave/README.md)
and [`SIGNING_ARCHITECTURE.md`](SIGNING_ARCHITECTURE.md).
