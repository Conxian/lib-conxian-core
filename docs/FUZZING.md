# Fuzzing

This document is the authoritative inventory and operating guide for the
`lib-conxian-core` fuzz suite. The suite is intentionally limited to parsing,
structural validation, and the currently supported MuSig2 aggregation path.

## Target inventory

| Target | Target source | Exercised paths and symbols | Coverage boundary |
| --- | --- | --- | --- |
| `parse_intent` | `fuzz/fuzz_targets/parse_intent.rs` | `src/protocol/intent.rs`: `Fdc3Instrument` and `IntentManager::resolve_fdc3_intent` | UTF-8 intent input and FDC3 instrument resolution |
| `musig2_aggregate` | `fuzz/fuzz_targets/musig2_aggregate.rs` | `musig2::KeyAggContext`, `musig2::secp::Point`, and `secp256k1::PublicKey` | Compressed public-key parsing and aggregation through the external `musig2` crate; production Vault SDK flows remain in `conxius-enclave-sdk` |
| `anchoring_receipt` | `fuzz/fuzz_targets/anchoring_receipt.rs` | `src/anchoring.rs`: `AnchoringReceipt` and its `serde` representation | JSON deserialization of anchoring receipts, including timestamps, enum values, publications, and optional metadata |
| `proof_request_validate` | `fuzz/fuzz_targets/proof_request_validate.rs` | `src/verifier.rs`: `ProofVerificationRequest` and its inherent `validate()` method | JSON deserialization followed by structural validation for successfully decoded requests; when an optional proof envelope is present, its fail-closed contract and policy validation also runs; this is **not** cryptographic BitVM2 proof verification |

The removed `psbt_parse`, Core-owned `key_aggregate`, and BitVM2 `proof_verify`
targets are intentionally not restored. Their APIs no longer belong to this
crate's current fuzz surface.

BIP-322 is also intentionally not a dedicated target in the current
four-target suite. The current core `Bip322Bridge` implementation performs
structural checks only; it does not establish message-signature authenticity or
provide cryptographic verification coverage. No such authenticity or
cryptographic coverage should be inferred from this fuzz suite.

## Local usage

Install a nightly toolchain and the pinned cargo-fuzz release used by CI:

```sh
rustup toolchain install nightly
cargo +nightly install cargo-fuzz --locked --version 0.13.2
```

Compile one target without running it:

```sh
cargo +nightly fuzz check <target>
```

Run a bounded local smoke test. The same bounds are used by CI:

```sh
cargo +nightly fuzz run <target> -- \
  -max_total_time=30 \
  -rss_limit_mb=2048 \
  -timeout=5 \
  -max_len=16384
```

Replace `<target>` with one of `parse_intent`, `musig2_aggregate`,
`anchoring_receipt`, or `proof_request_validate`. The proof-request target
performs structural and policy validation only; it does not verify Groth16 or
BitVM2 cryptographic proofs.

## CI regression policy

`.github/workflows/fuzz-regression.yml` runs every target in a matrix on a
weekly schedule and through `workflow_dispatch`. Each matrix job compiles with
Rust nightly and runs cargo-fuzz for a maximum of 30 seconds with a 2048 MiB
resident-memory limit, a five-second per-input timeout, and a 16 KiB maximum
input length. Fuzz failures are not allowed to pass via `continue-on-error`; a
crash or other non-zero fuzz result fails its matrix job. Crash artifacts and
target corpus files are uploaded after every run, including failed runs, with a
14-day retention period.

## Corpus review and retention

Checked-in seed inputs may live under `fuzz/corpus/<target>/` when they are
small, deterministic, reproducible, and useful for regression coverage.
`fuzz/corpus/` is intentionally not ignored: only reviewed seed corpora may be
committed, and generated local corpus growth must be cleaned up or reviewed
before committing. Do not add a blanket ignore for this directory because CI
uploads its contents for review.
Generated crash output is written under `fuzz/artifacts/<target>/`, and
generated cargo-fuzz build output is written under `fuzz/target/`; both paths
are ignored by `fuzz/.gitignore`. Review minimized crash inputs before adding
them to a corpus, document the regression they preserve, and remove redundant
or non-reproducible seeds.

## Zero-secret corpus constraints

Fuzz inputs and checked-in corpora must contain no private keys, credentials,
access tokens, environment files, customer data, network secrets, or other
confidential material. Use synthetic protocol values only. Treat uploaded
crash and corpus artifacts as public repository CI artifacts and review them
under the same zero-secret standard before retaining or sharing them.
