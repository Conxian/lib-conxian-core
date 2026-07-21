# Core coverage policy (CON-1504 / CORE-008)

This document establishes the first measured coverage target for
`lib-conxian-core`. The initial CI stage is intentionally **report-only**:
coverage artifacts and threshold results are published on every pull request
and push, but a low number does not fail the workflow yet. Enforcement starts
only after the baseline and exclusions have been reviewed.

## Tooling and reproducible commands

The selected collector is `cargo-llvm-cov` **0.8.7**, using the LLVM source
coverage tools supplied by the active Rust toolchain. The version is pinned in
`config/core_coverage.json` and in CI. No optional features are enabled for
this gate; the optional enclave/MSRV concern remains outside the default
feature baseline.

```bash
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.8.7 --locked
cargo llvm-cov --version

mkdir -p target/coverage
ignore='(^|/)(tests|fuzz|target|generated|vendor|examples)/'
cargo llvm-cov --workspace --locked \
  --ignore-filename-regex "$ignore" \
  --json --output-path target/coverage/coverage.json
cargo llvm-cov report --locked \
  --ignore-filename-regex "$ignore" \
  --lcov --output-path target/coverage/coverage.lcov
cargo llvm-cov report --locked \
  --ignore-filename-regex "$ignore" \
  --html --output-dir target/coverage/html

python3 scripts/core_coverage.py \
  --input target/coverage/coverage.json \
  --policy config/core_coverage.json \
  --mode report-only \
  --output-json target/coverage/core-coverage.json \
  --output-markdown target/coverage/core-coverage.md
```

The JSON parser is deliberately defensive about the observed LLVM shape
(`data[].files[].summary`) and refuses ambiguous duplicate file entries rather
than silently producing a misleading denominator. The evaluator supports
`report-only` and `enforce` modes. In enforce mode, every failure includes the
actual metric, required threshold, affected target, and a next action.

## Measured dimensions

| Dimension | Meaning | Initial treatment |
| --- | --- | --- |
| Line | Executable source lines reached | Overall and named targets use this metric where specified |
| Region | LLVM source regions reached | Reported for diagnosis; not currently a threshold |
| Function | Functions reached | Trust-policy target uses this metric |
| Branch | LLVM branch decisions reached | Reported when available; current LLVM export has zero branch records without unstable `--branch` |

The report also preserves LLVM's MC/DC dimension when present, but it is not a
target. `--branch` is not enabled in the first stage because the
`cargo-llvm-cov` interface marks it unstable; the policy does not invent
branch precision from line coverage. Trust-policy branch intent is instead
reviewed against the named decision taxonomy until a stable branch report is
available.

## Denominator and exclusions

The initial denominator is the normalized repository path set
`src/**/*.rs`, after applying the exclusions in
`config/core_coverage.json`:

- `src/tests.rs` and standalone `tests/**` are test sources, not production
  denominator paths. Their tests still execute production code.
- `fuzz/**` is fuzzing, not coverage evidence.
- `examples/**` is executable documentation/example code, not production
  behavior.
- `generated/**`, `vendor/**`, and `target/**` are generated or dependency
  artifacts and are excluded.
- No difficult signing, verifier, trust-policy, adapter, or BIP-110 source is
  excluded merely because it is difficult to cover.

The denominator is intentionally path-based. Inline `#[cfg(test)]` modules
embedded in a production Rust file share that file's source path in LLVM's
JSON export; they are not silently relabeled as production assurance. This is
recorded as a limitation for the exclusions review, while standalone test
targets remain outside the denominator.

Production files that LLVM reports without any measurable executable item are
kept in the machine-readable inventory as `unmeasured_files`; they contribute
zero LLVM line/region/function items rather than being assigned synthetic
coverage.

Generated JSON, LCOV, HTML, and metadata are written only below `target/coverage`
and CI artifact storage. They are ignored by Git and must not contain
environment/configuration dumps, credentials, or private keys.

## Named targets

| Target | Scope | Metric | Initial target |
| --- | --- | --- | ---: |
| Overall | All denominator files | Lines | **>= 85%** |
| Signing | `src/signing.rs` | Lines | **>= 95%** |
| Verification | `src/verifier.rs` | Lines | **>= 95%** |
| Trust policy | `src/control_model/trust.rs` | Functions | **100%** |
| BIP-110 | `src/control_model/bip110.rs` and `src/control_model/bip110_preflight.rs` | Lines | **>= 95%** |

The trust-policy target is a function-coverage floor paired with review of all
decision/branch intent. It is not a claim that function coverage proves every
trust-tier combination. The BIP-110 target is paired with the boundary matrix
below, because a line percentage alone does not demonstrate boundary behavior.

## BIP-110 boundary matrix

The BIP-110 tests must keep the following boundary cases explicit for every
applicable measurement field:

| Measurement | Required cases |
| --- | --- |
| Pushdata | Exact 256-byte limit passes; 257 bytes fails |
| `OP_RETURN` | Exact 83-byte limit passes; 84 bytes fails |
| ScriptPubKey | Exact 34-byte limit passes; 35 bytes fails |
| Witness element | Exact 256-byte limit passes; 257 bytes fails |
| Preflight structure | Missing measurements, phase/source mismatch, disabled compliance, and unknown context fail closed |
| Occurrence handling | Multiple violations preserve category and occurrence order |
| Taproot control block | The separate boundary is inclusive at its limit and rejects the next byte |

Taproot, BIP-341/BIP-342, Miniscript, DLC, and other specialized contexts must
not be represented as generic byte-size coverage. Where Core does not own the
parser/interpreter contract, the preflight context is an explicit fail-closed
gap and the owning adapter/SDK must provide the deeper evidence.

## Historical and current baselines

`docs/coverage/baselines.json` is the small checked-in baseline artifact. It
records the exact source SHA, toolchain metadata, command, denominator policy,
overall dimensions, and named-target status. Raw JSON/LCOV/HTML remains a
reproducible CI artifact rather than a checked-in generated report.

The historical baseline uses
`4065271bf6d9b035aa17f1c454f6a1db0c54754c`, the SHA required by CON-1504. It
predates the finalized `src/signing.rs`, `src/verifier.rs`, and BIP-110
preflight modules, so its missing named scopes are labeled **historical
pre-critical-module** rather than treated as current enforcement evidence.

The current enforcement candidate is the latest `origin/main` used for this
rollout: `604ec2be569e2862acaaa400d5d2b90824b13fd3`. The distinction is
important: the historical report is reproducible context, while current main
is the candidate for future threshold enforcement.

To reproduce the historical artifact without changing that commit, create a
temporary worktree at the exact SHA, copy this script and the policy into the
worktree, run the same coverage commands, and pass
`--commit-sha 4065271bf6d9b035aa17f1c454f6a1db0c54754c` to the evaluator. The
temporary files and raw reports stay outside version control.

## Gap inventory

- **Universal chain signing (UCS):** exercise supported chain, algorithm,
  operation, payload, address, capability, and response-validation paths in
  `src/signing.rs`; retain explicit negative tests for unsupported combinations
  and malformed public metadata.
- **Protocol verification:** cover request validation, capability discovery,
  evidence binding, finality monotonicity, result provenance, policy metadata,
  backend errors, and adversarial backend postconditions in `src/verifier.rs`.
- **Trust policy:** cover every `Strict`, `Managed`, `Expedient`, and
  `ObserverOnly` decision intent, including incompatible verification classes
  and fail-closed defaults. Function coverage is a floor, not a substitute for
  the decision matrix.
- **BIP-110:** maintain exact-limit/limit-plus-one vectors, aggregate
  violation ordering, preflight structural failures, provenance/phase checks,
  and disabled-compliance rejection.
- **BIP-341/BIP-342/Miniscript:** parser/interpreter behavior is outside current
  Core ownership. Unsupported contexts remain explicit fail-closed gaps and
  must not be claimed as generic size coverage.
- **Adapters and error paths:** inventory each chain adapter's validation,
  trust mapping, proof/finality, unsupported operation, malformed input, and
  typed error branches. Adapter behavior that requires network IO or persistence
  belongs in Gateway tests, not this crate's coverage denominator.

Coverage does not prove cryptographic correctness, consensus compatibility,
property completeness, fuzzing quality, static-analysis quality, audit quality,
or production-path equivalence. Those assurances remain separate controls.
