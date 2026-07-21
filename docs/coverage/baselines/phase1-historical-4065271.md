# Rust coverage report

- Baseline label: `historical-phase1`
- Source commit: `4065271bf6d9b035aa17f1c454f6a1db0c54754c`
- Tool: `cargo-llvm-cov 0.8.6`
- Rust toolchain: `1.89.0`
- Method: `cargo llvm-cov --package lib-conxian-core --locked --all-targets --no-default-features`
- Canonical metric: **line coverage**; regions and functions are reported for diagnosis.
- Branch coverage: disabled/not gated because the LLVM branch mode is currently unstable.

| Scope | Lines | Regions | Functions | Status | Eventual line target |
| --- | ---: | ---: | ---: | --- | ---: |
| `overall` | 69.40% | 70.95% | 64.80% | measured | 85.00% |
| `universal-signing` | N/A | N/A | N/A | not_applicable | 90.00% |
| `protocol-verification` | N/A | N/A | N/A | not_applicable | 90.00% |
| `trust-policy` | 90.93% | 92.89% | 98.04% | measured | 95.00% |
| `bip110-policy` | N/A | N/A | N/A | not_applicable | 90.00% |

Target shortfalls are advisory in this report-first phase; measurement and parser failures are not.
