# Rust coverage report

- Baseline label: `current-implementation`
- Source commit: `73143c2b916a6c0f3cf9117ea36c0ae1e170d9d4`
- Tool: `cargo-llvm-cov 0.8.6`
- Rust toolchain: `1.89.0`
- Method: `cargo llvm-cov --package lib-conxian-core --locked --all-targets --no-default-features`
- Canonical metric: **line coverage**; regions and functions are reported for diagnosis.
- Branch coverage: disabled/not gated because the LLVM branch mode is currently unstable.

| Scope | Lines | Regions | Functions | Status | Eventual line target |
| --- | ---: | ---: | ---: | --- | ---: |
| `overall` | 73.31% | 73.21% | 72.64% | measured | 85.00% |
| `universal-signing` | 73.16% | 68.40% | 83.33% | measured | 90.00% |
| `protocol-verification` | 71.91% | 73.44% | 82.88% | measured | 90.00% |
| `trust-policy` | 91.83% | 93.63% | 98.44% | measured | 95.00% |
| `bip110-policy` | 82.57% | 80.22% | 70.59% | measured | 90.00% |

Target shortfalls are advisory in this report-first phase; measurement and parser failures are not.
