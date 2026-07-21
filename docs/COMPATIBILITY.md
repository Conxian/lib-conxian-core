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
