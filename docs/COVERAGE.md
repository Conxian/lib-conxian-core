# Core Rust coverage

This repository uses a **report-first** coverage rollout for the root
`lib-conxian-core` package. Coverage is evidence about the exercised Rust
source, not a substitute for fuzzing, static analysis, audits, or protocol
review.

## Canonical method

The reproducible entry point is:

```text
python scripts/coverage_report.py --mode report-only --baseline-label ci --output-dir coverage/llvm-cov
```

The script refuses an unexpected toolchain or coverage-tool version and records
the checked-out full Git commit in every summary. The pinned method is:

- Rust toolchain `1.89.0` for coverage only. The package MSRV remains Rust
  `1.85` in `Cargo.toml`.
- `cargo-llvm-cov` exactly `0.8.6`.
- Package `lib-conxian-core` only; the workspace's fuzz package is not part of
  the core coverage measurement.
- `--locked`, `--all-targets`, and no optional `enclave` feature by default.
- One instrumented test run followed by report-only JSON, LCOV, HTML, and text
  exports, so report formats do not rerun tests.

The underlying test command is equivalent to:

```text
cargo llvm-cov --no-report --package lib-conxian-core --locked --all-targets --no-default-features
cargo llvm-cov report --package lib-conxian-core --locked --json --output-path coverage/llvm-cov/coverage.json
cargo llvm-cov report --package lib-conxian-core --locked --lcov --output-path coverage/llvm-cov/lcov.info
cargo llvm-cov report --package lib-conxian-core --locked --html --output-dir coverage/llvm-cov
cargo llvm-cov report --package lib-conxian-core --locked --text --output-path coverage/llvm-cov/coverage.txt
```

Generated reports belong under the ignored `coverage/` directory. CI uploads
the JSON, LCOV, HTML, text, and generated summary artifacts; build outputs and
absolute workstation paths are not committed.

## Metrics and staged gates

**Line coverage is the canonical gate metric.** Region and function coverage
are reported because they identify control-flow and API exercise gaps. LLVM
branch coverage is intentionally disabled and not gated initially because its
current mode is unstable for this repository.

The reviewed policy is [`docs/coverage/policy.json`](coverage/policy.json):

| Scope | Eventual line target |
| --- | ---: |
| Overall package | 85% |
| Universal signing (`src/signing.rs`) | 90% |
| Protocol verification (`src/verifier.rs`) | 90% |
| Trust policy (`src/control_model/trust.rs` plus `mod.rs`) | 95% |
| BIP-110 (`bip110.rs` plus `bip110_preflight.rs`) | 90% |

The current measured floors are kept separately in the generated
[`current implementation JSON baseline`](coverage/baselines/current-implementation.json),
not mixed with the eventual targets. The target aggregate is computed from
covered/count totals in the raw LLVM JSON; it is never a manual average of
file percentages. The generated Markdown baseline gives the same named
summary for reviewers.

Rollout stages:

1. **Report-only (current):** tool installation, test execution, report
   generation, JSON parsing, and summary generation must succeed. Eventual
   target shortfalls are advisory and do not fail CI.
2. **Reviewed baseline / no-regression:** compare the current line totals with
   the checked-in current implementation floors before accepting regressions.
3. **Ratchet:** deliberately raise floors as coverage work lands and review
   any intentional source or target-scope changes.
4. **Final gate:** enforce the 85% overall target and the named critical-module
   floors.

The script has separate `--mode no-regression` and `--mode enforce` paths for
the later stages. CI currently invokes only `--mode report-only`.

## Two baselines

The Phase 1 roadmap ref `4065271bf6d9b035aa17f1c454f6a1db0c54754c` predates the
current signing, verifier, and BIP-110 module layout. It is therefore not
honest to label current-module measurements as historical Phase 1 coverage.
This rollout keeps two explicit baselines:

- [`Phase 1 historical baseline`](coverage/baselines/phase1-historical-4065271.json)
  — generated from exactly the historical ref in a clean temporary checkout.
  Critical files absent at that commit are recorded as `not_applicable`, not
  as zero coverage.
- [`Current implementation baseline`](coverage/baselines/current-implementation.json)
  — generated with the same pinned method from the current implementation
  source. The branch tooling/doc/workflow changes do not change Rust source
  coverage; the artifact records the exact source commit and dirty paths.

The historical label is guarded by the script and fails unless the checked-out
commit is exactly the required full ref.

The generated artifacts currently record these exact aggregate percentages
(line coverage is the gate metric):

| Baseline | Scope | Lines | Regions | Functions | Status |
| --- | --- | ---: | ---: | ---: | --- |
| Phase 1 `4065271…` | Overall | 69.40% | 70.95% | 64.80% | measured |
| Phase 1 `4065271…` | Universal signing | N/A | N/A | N/A | not applicable: file absent |
| Phase 1 `4065271…` | Protocol verification | N/A | N/A | N/A | not applicable: file absent |
| Phase 1 `4065271…` | Trust policy | 90.93% | 92.89% | 98.04% | measured |
| Phase 1 `4065271…` | BIP-110 | N/A | N/A | N/A | not applicable: files absent |
| Current `73143c2…` | Overall | 73.31% | 73.21% | 72.64% | measured |
| Current `73143c2…` | Universal signing | 73.16% | 68.40% | 83.33% | measured |
| Current `73143c2…` | Protocol verification | 71.91% | 73.44% | 82.88% | measured |
| Current `73143c2…` | Trust policy | 91.83% | 93.63% | 98.44% | measured |
| Current `73143c2…` | BIP-110 | 82.57% | 80.22% | 70.59% | measured |

These values are copied from the generated JSON/Markdown artifacts above;
the script derives each aggregate from covered/count totals and does not
average file percentages. The current implementation source commit is the
full `73143c2b916a6c0f3cf9117ea36c0ae1e170d9d4`; the line floors and their
covered/count totals are also recorded in `docs/coverage/policy.json`.

## Scope and interpretation

- **Generated code:** no blanket exclusions are configured. Any future
  exclusion must name an exact generated path and be reviewed in the policy and
  documentation.
- **Unreachable code:** not automatically excluded. Unreachable or defensive
  paths remain visible until their necessity and testability are reviewed.
- **Examples and doctests:** `--all-targets` covers the package's normal library,
  integration-test, example, binary, and benchmark targets that Cargo exposes.
  Doctests are not enabled because cargo-llvm-cov marks doctest coverage as an
  unstable limitation; a future change must measure and document them
  separately rather than implying they are covered.
- **Optional features:** the baseline intentionally disables default features
  with `--no-default-features`. The enclave SDK is a separate production crate
  and must not silently change the core baseline. Feature-specific coverage
  should be a separately named measurement.
- **Fuzz targets:** the `fuzz` workspace member remains separate. Fuzzing is
  valuable regression evidence but is not counted as unit/integration coverage.
- **Adapters and error paths:** low adapter/error-path numbers are retained in
  the report. They are gap signals, not reasons to exclude adapters or weaken
  the package denominator.

## Current gap inventory

The baseline is a measurement of the current implementation boundary; it does
not claim that unimplemented protocol surfaces are covered.

- **UCS / universal signing:** `src/signing.rs` is the Core contract surface;
  concrete custody and hardware-backed signing belong to
  `conxius-enclave-sdk`. Exercise malformed requests, capability rejection,
  response validation, and negative verification without introducing private
  key material into Core tests.
- **BIP-341 / BIP-342 / Miniscript:** transaction parsing, Taproot control
  block semantics, Tapscript execution, and Miniscript compilation remain an
  implementation gap owned by the appropriate adapter/SDK boundary. They are
  not blanket coverage exclusions. The BIP-110 preflight contract measures
  supplied size metadata and must not be mistaken for full script validation.
- **BIP-110:** `bip110.rs` validates classified size vectors, while
  `bip110_preflight.rs` validates request structure, phase/source pairing,
  supported contexts, control-block limits, and deterministic findings. Keep
  the two files named together in the eventual target; reporting only the
  already-100%-covered validator would be misleading.
- **ProtocolVerifier:** `src/verifier.rs` owns the Core façade and invariant
  checks. Backend I/O, chain evidence acquisition, persistence, and retries
  remain outside this crate. Time-boundary, evidence-binding, provenance,
  capability, and nested-finality error paths are the highest-value gaps.
- **Adapters and error paths:** adapter modules, Bitcoin/BIP-322, anchoring,
  contract bridge, and RGB paths have lower or zero measured line coverage in
  the current implementation. Their paths should be covered with deterministic
  contract tests or explicitly implemented before any final gate is raised.

## Reproduction and verification

Install the exact coverage tool under the independent Rust coverage toolchain:

```text
rustup toolchain install 1.89.0
rustup component add llvm-tools-preview --toolchain 1.89.0
cargo +1.89.0 install cargo-llvm-cov --locked --version 0.8.6
python3 scripts/test_coverage_report.py
python3 scripts/coverage_report.py --mode report-only --baseline-label ci --output-dir coverage/llvm-cov
```

Normal repository verification remains separate and keeps the package MSRV
contract visible:

```text
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The historical artifact was generated in a temporary clean worktree at the
exact Phase 1 ref with the same script, policy, tool, and coverage toolchain.
Do not regenerate it from current `main` and relabel the result: that would
conflate source layouts and falsify the historical comparison.

## Related work and caveats

Coverage targets are intentionally not used to close unrelated implementation
issues. CORE-001 / issue #174 is closed; CORE-002 / issue #178 remains open and
its implementation is absent; CORE-004 / issue #179 remains open although the
BIP-110 code exists; fuzz issue #147 remains open; strict CI issue #155 is
closed; and parent issue #173 remains open. These statuses explain why the
baseline includes explicit gaps and why report-first CI is not yet a hard
85% gate.
