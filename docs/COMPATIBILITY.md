# Rust and Feature Compatibility

## Supported package floor

`lib-conxian-core` declares `rust-version = "1.91"`. Rust `1.91.0` is the
explicit toolchain used by CI. This is a package-wide floor: Cargo exposes one
`rust-version` for the crate, so the supported floor applies to the default
and optional feature sets alike.

| Surface | Feature selection | Supported Rust | Locked dependency evidence |
| --- | --- | --- | --- |
| Package | Any published feature set | `1.91+` | The package metadata establishes the single supported floor. |
| Default graph | `default = []` | `1.91+` | The current locked graph includes `enum-ordinalize 4.4.1`, which requires Rust `1.89`. The package floor remains `1.91` so every feature set has one documented support contract. |
| Optional enclave graph | `enclave` | `1.91+` | `conxius-enclave-sdk 2.0.11` resolves `alloy 2.1.1` (Rust `1.91`) and `ruint 1.19.0` (Rust `1.90`). |

The default graph may have a lower transitive minimum than the package floor,
but Rust versions below `1.91` are not supported for this crate release. CI
runs locked `check`, `test`, and all-target `clippy -D warnings` coverage for
both the default and all-feature graphs.

## Optional enclave SDK coordination

The `enclave` feature remains a thin, optional integration with the production
[`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate at
version `2.0.11`. Hardware-backed signing, attestation, and policy behavior
remain owned by that SDK; this crate only exposes the optional dependency path.

When upgrading `conxius-enclave-sdk`:

1. Review the SDK's declared MSRV and its resolved transitive graph, including
   Alloy and `ruint` versions.
2. Keep `Cargo.toml`, CI's explicit toolchain, this matrix, and the release
   notes synchronized in the same change when the supported floor changes.
3. Run the locked default and all-feature `check`, `test`, and all-target
   `clippy -D warnings` commands before publishing either side of the
   integration.

Do not downgrade Alloy in `lib-conxian-core` solely to preserve the stale Rust
`1.85` declaration; any dependency change requires separate compatibility
validation and coordination with the SDK release.

## Opt-in SDK v2.0.11 compatibility evidence

The release-compatibility harness is a separate, non-published workspace package
at [`tests/sdk-compat`](../tests/sdk-compat). It is opt-in through the local
`run` feature and does not add an SDK, simulator, or mock dependency to Core's
default production graph.

### Evidence baseline

This checkpoint was based on Core `origin/main` commit
`97ffaa76d847d34e1caef3d946804bd6ebdb445f` on **July 21, 2026**:

| Surface | Exact baseline |
| --- | --- |
| Core package | `lib-conxian-core` `0.3.0` |
| Core Rust floor | `1.91` (`1.91.0` in CI and the harness) |
| SDK formal release | `conxius-enclave-sdk` `2.0.11` |
| SDK release tag | `v2.0.11` at commit `d3e9a6a26da1bd4c15e612ce7051a0bfdf640a83` |
| SDK release Rust floor | `1.85` as declared by the published release manifest |
| Lockfile provenance | Root `Cargo.lock`; SDK registry checksum `e35d22138325b93283bab7afeeca5a121a0d2019bd7d9d81b69af7ec46db5f2d` |

The SDK release's declared `1.85` floor is distinct from the Core package
floor: every harness invocation runs with Rust `1.91.0` because it compiles
the current Core package and its optional `enclave` graph. SDK `main` is not a
release baseline and is not included; at this checkpoint it advertises package
metadata `2.0.12` and Rust `1.94.1`.

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
cargo +1.91.0 fetch --locked
python3 scripts/run_sdk_compat.py --offline
```

The regular workspace `--all-features` CI job deliberately enables the
harness's `run` feature and exercises its DTO-only SDK feature checks. That is
a broad workspace smoke test, not a substitute for this dedicated eight-command
matrix.

The individual locked offline commands use the following shape (the script
expands the full Core/SDK matrix):

```text
cargo +1.91.0 test --offline --manifest-path tests/sdk-compat/Cargo.toml --locked \
  --no-default-features --features run,core-enclave,all-supported
```

The dedicated `.github/workflows/sdk-compat.yml` workflow runs for pull
requests targeting `main`, pushes to `main`, and manual dispatches. It first
acquires the locked dependency graph with `cargo +1.91.0 fetch --locked`, then
runs the eight matrix commands with Cargo's `--offline` flag. This proves the
matrix is network-independent at runtime after dependency acquisition; the
locked fetch step itself is the dependency-acquisition phase and may use the
network. Focused formatting and linting remain repository-local commands:

```text
cargo fmt --all -- --check
cargo +1.91.0 clippy --manifest-path tests/sdk-compat/Cargo.toml --tests \
  --locked --no-default-features --features run,core-enclave,all-supported -- -D warnings
```

### Dependency direction and ownership

```text
lib-conxian-core-sdk-compat (non-published, opt-in test evidence)
├── local lib-conxian-core 0.3.0
│   └── conxius-enclave-sdk 2.0.11 only when Core's `enclave` feature is enabled
└── conxius-enclave-sdk =2.0.11 (direct evidence dependency)
```

The harness depends on both local Core and the exact SDK release; the SDK does
not depend on Core. This preserves the production ownership boundary and avoids
an SDK-to-Core dependency cycle. The harness has no network, database,
credential, hardware, real-key, or secret-material path. The SDK's mock and
development features are never forwarded through Core's production features.

### What the evidence proves—and what it does not

The harness provides compile-time and deterministic offline evidence for:

- an explicit Core `TrustTier` ↔ SDK v2.0.11 rail `TrustTier` representation
  mapping (`Strict`/`Managed`/`Expedient`/`ObserverOnly` ↔ `T1`/`T2`/`T3`/`T4`);
- serde round trips for both sides' trust types, SDK signing/attestation DTOs,
  Core signing fixtures, and Core BIP-110 preflight fixtures;
- Core canonical BIP-110 limits paired with the SDK's independent
  `MempoolPolicy` configuration in a test-local evidence record; disabled or
  non-canonical Core policies and invalid SDK policy values fail closed;
- the existing deterministic Core signer and BIP-110 fixture boundaries.

The trust mapping is a representation adapter only. It does not make SDK `T4`
production-eligible when Core maps it to `ObserverOnly`, and it does not
authorize a signing or settlement operation. The v2.0.11 SDK exposes no
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
