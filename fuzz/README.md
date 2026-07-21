# `lib-conxian-core` fuzzing

This directory contains the four bounded `cargo-fuzz` regression targets for
current `lib-conxian-core` APIs and selected dependency-level
parser/aggregation surfaces. The targets are intended to find parser panics,
unexpected aborts, and dependency regressions; malformed input and a normal
`false` result are expected for the validation targets.

## Target ownership and scope

| Target | Coverage | Ownership boundary |
| --- | --- | --- |
| `parse_intent` | `IntentManager::resolve_fdc3_intent` with bounded UTF-8 input | Core protocol API |
| `anchoring_receipt` | `serde_json::from_slice::<AnchoringReceipt>` with bounded bytes | Core data model deserialization |
| `musig2_aggregate` | `musig2::KeyAggContext` over bounded public-key input | Direct upstream `musig2` dependency-level key aggregation coverage |
| `proof_request_validate` | `ProofVerificationRequest` JSON deserialization followed by structural and policy validation | Core protocol model; not Groth16 or BitVM2 cryptographic verification |

The targets cap input size and key count before invoking the parser or
aggregator. `proof_request_validate` rejects inputs larger than 16 KiB before
JSON decoding. Dependency code may allocate internally after receiving this
bounded input. They do not reintroduce the removed `src/bitvm2.rs`,
`src/musig2.rs`, or historical Bitcoin orchestrator PSBT APIs. Production
MuSig2 and BitVM2 coverage, including cryptographic proof verification, belongs
to [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk), not
this crate.

## Local prerequisites

- Rust stable for the workspace checks.
- A Rust nightly toolchain for `cargo-fuzz`:

  ```bash
  rustup toolchain install nightly --profile minimal
  ```

- The pinned fuzz runner used by CI:

  ```bash
  cargo install cargo-fuzz --locked --version 0.13.2
  ```

## Check and smoke-test commands

Run from the repository root. The `--locked` check confirms that the workspace
lockfile contains the declared fuzz dependencies.

```bash
cargo check -p lib-conxian-core-fuzz --bins --locked
for target in \
  parse_intent \
  anchoring_receipt \
  musig2_aggregate \
  proof_request_validate
do
  cargo +nightly fuzz check "$target"
done
```

Run one target for a short bounded smoke session with a five-second per-input
timeout, a 2 GiB RSS ceiling, and generated input length bounded at 16 KiB:

```bash
cargo +nightly fuzz run --sanitizer none --dev --jobs 1 \
  anchoring_receipt fuzz/corpus/anchoring_receipt -- \
  -max_total_time=5 -timeout=5 -rss_limit_mb=2048 -max_len=16384
```

On a small development machine, the unoptimized, unsanitized mode keeps the
Rust dependency build below the available memory ceiling. Run the bounded
smoke suite for all four targets as follows:

```bash
for target in \
  parse_intent \
  anchoring_receipt \
  musig2_aggregate \
  proof_request_validate
do
  cargo +nightly fuzz run --sanitizer none --dev --jobs 1 \
    "$target" "fuzz/corpus/$target" -- \
    -max_total_time=5 -timeout=5 -rss_limit_mb=2048 -max_len=16384
done
```

For a longer local reproduction matching scheduled CI, replace
`-max_total_time=5` with `-max_total_time=30`.

## Corpus and artifact policy

- Reviewable deterministic seeds live under `fuzz/corpus/<target>/` and are
  tracked in Git.
- Keep corpus additions small and reviewable; remove hash-named inputs generated
  by local smoke runs unless they are intentionally minimized regression seeds.
- `fuzz/artifacts/` contains generated crashes, timeouts, and other failure
  output. The directory is ignored and must not be committed.
- The fuzz-local `fuzz/target/` build directory and generated crash filenames
  are ignored by `fuzz/.gitignore`.
- A discovered failure should be reproduced with the target's corpus path and
  the generated artifact path before adding a small, reviewable regression
  seed.

## Scheduled CI

`.github/workflows/fuzz-regression.yml` runs each target weekly and supports
manual dispatch. Each matrix job runs for 30 seconds with a 2 GiB RSS limit,
an explicit five-second per-input timeout, and a 16 KiB maximum input length.
It fails on fuzz-run errors and uploads target-specific corpus/artifact output
when a job fails.
