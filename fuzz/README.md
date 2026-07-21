# `lib-conxian-core` fuzzing

This directory contains the four bounded `cargo-fuzz` regression targets for
the current `lib-conxian-core` parsing, deserialization,
structural/policy/evidence-binding validation, and dependency-level aggregation
surfaces. The suite is designed to find panics, unexpected aborts, and
dependency regressions; malformed input and a normal validation error are
expected outcomes for the validation targets.

## Target inventory and ownership

| Target | Coverage | Ownership boundary |
| --- | --- | --- |
| `parse_intent` | `IntentManager::resolve_fdc3_intent` with bounded UTF-8 input | Core protocol API |
| `musig2_aggregate` | `musig2::KeyAggContext` over bounded compressed public-key input | Dependency-level aggregation through the external `musig2` crate |
| `anchoring_receipt` | `serde_json::from_slice::<AnchoringReceipt>` with bounded bytes | Core anchoring-receipt deserialization |
| `proof_request_validate` | `ProofVerificationRequest` JSON deserialization followed by structural validation and, when an optional proof envelope is present, policy and evidence-binding validation | Core protocol model; no Groth16, BitVM2, or message-signature cryptographic verification |

The four targets are intentionally the complete current inventory. Obsolete
PSBT, BIP-322, Core-owned MuSig2, and cryptographic `proof_verify` targets are
not restored. Production MuSig2 sessions, hardware-backed signing,
attestation, and BitVM2 operations belong to the
[`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk), not this
crate. Production BIP-322 signing and message-authenticity verification also
belong to `conxius-enclave-sdk`; this crate's `Bip322Bridge` is structural-only
and must not be used as an authenticity decision. There is no PSBT fuzz target
in this repository, and BIP-322 and BitVM2 are not current fuzz targets here.
In particular, `proof_request_validate` does **not** perform cryptographic
BitVM2 proof verification; it only exercises request deserialization plus
structural, contract, policy, and evidence-binding validation.

## Bounds and prerequisites

The scheduled regression job uses the same bounds for every target:

- `-max_total_time=30` seconds per target
- `-timeout=5` seconds per input
- `-rss_limit_mb=2048`
- `-max_len=16384` bytes per generated input

Targets may apply a narrower input cap before calling the exercised API:
`parse_intent` uses 4 KiB, `musig2_aggregate` uses 32 compressed keys, and
`anchoring_receipt` plus `proof_request_validate` use 16 KiB. The per-input
runner limit remains 16 KiB so CI and local invocations have a consistent
maximum.

Install the required toolchains and the pinned cargo-fuzz release:

```bash
rustup toolchain install nightly --profile minimal
cargo +nightly install cargo-fuzz --locked --version 0.13.2
```

## Compile and smoke-test commands

Run from the repository root. The workspace check confirms that the fuzz
package and its locked dependencies are available:

```bash
cargo check -p lib-conxian-core-fuzz --bins --locked
```

Compile each current target without running it:

```bash
for target in \
  parse_intent \
  musig2_aggregate \
  anchoring_receipt \
  proof_request_validate
do
  cargo +nightly fuzz check "$target"
done
```

Run one target with its reviewed corpus and target-specific artifact prefix:

```bash
target=proof_request_validate
corpus_dir="fuzz/corpus/$target"
artifact_dir="fuzz/artifacts/$target"
mkdir -p "$artifact_dir"
cargo +nightly fuzz run "$target" "$corpus_dir" -- \
  -max_total_time=30 \
  -timeout=5 \
  -rss_limit_mb=2048 \
  -max_len=16384 \
  -artifact_prefix="$artifact_dir/"
```

For a constrained development machine, it is acceptable to use a shorter
smoke duration and `--sanitizer none --dev --jobs 1`; record that deviation in
verification results because it is not equivalent to the scheduled CI run.

## Corpus and artifact policy

- Reviewed deterministic seeds live under `fuzz/corpus/<target>/` and are
  tracked in Git.
- Keep corpus additions small, synthetic, and reproducible. Derive structured
  seeds from current types and tests rather than inventing fields.
- `fuzz/artifacts/<target>/` contains generated crashes, timeouts, and other
  failure output. It is ignored and must not be committed.
- `fuzz/target/` contains generated cargo-fuzz build output and is ignored.
- Reproduce and minimize a discovered failure with the target's corpus and
  artifact paths before retaining a regression seed.

## Zero Secret Egress

Fuzz inputs and checked-in corpora must contain synthetic protocol values only.
Never add private keys, credentials, access tokens, environment files,
customer data, network data, or other confidential material. Treat uploaded
crash and corpus artifacts as public repository CI artifacts and review them
under the same zero-secret standard before retaining or sharing them.

## Scheduled CI

`.github/workflows/fuzz-regression.yml` runs the four targets weekly and via
`workflow_dispatch`. Each matrix job compiles with Rust nightly, runs the
bounded command above, and uploads the target-specific corpus and artifact
directories for 14 days. A crash, timeout, OOM, or other non-zero fuzz result
fails the matrix job, and artifacts are uploaded after every run, including
failed runs (`if: always()`).
