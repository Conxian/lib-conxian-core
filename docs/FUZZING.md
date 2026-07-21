# Fuzzing

This document is the authoritative inventory and operating guide for the
`lib-conxian-core` fuzz suite. The suite is intentionally limited to parsing,
structural/policy/evidence-binding validation, and the currently supported
dependency-level MuSig2 aggregation path.

## Target inventory

| Target | Target source | Exercised paths and symbols | Coverage boundary |
| --- | --- | --- | --- |
| `parse_intent` | `fuzz/fuzz_targets/parse_intent.rs` | `src/protocol/intent.rs`: `Fdc3Instrument` and `IntentManager::resolve_fdc3_intent` | UTF-8 intent input and FDC3 instrument resolution |
| `musig2_aggregate` | `fuzz/fuzz_targets/musig2_aggregate.rs` | `musig2::KeyAggContext`, `musig2::secp::Point`, and `secp256k1::PublicKey` | Compressed public-key parsing and aggregation through the external `musig2` crate; production Vault SDK flows remain in `conxius-enclave-sdk` |
| `anchoring_receipt` | `fuzz/fuzz_targets/anchoring_receipt.rs` | `src/anchoring.rs`: `AnchoringReceipt` and its `serde` representation | JSON deserialization of anchoring receipts, including timestamps, enum values, publications, and optional metadata |
| `proof_request_validate` | `fuzz/fuzz_targets/proof_request_validate.rs` | `src/verifier.rs`: `ProofVerificationRequest` and its inherent `validate()` method | JSON deserialization followed by structural validation; when an optional proof envelope is present, policy and evidence-binding validation also run; it does **not** perform Groth16, BitVM2, or message-signature cryptographic verification |

The removed `psbt_parse`, Core-owned `key_aggregate`, and BitVM2 `proof_verify`
targets are intentionally not restored. Their APIs no longer belong to this
crate's current fuzz surface.

BIP-322 is also intentionally not a dedicated target in the current
four-target suite. Production BIP-322 signing and message-authenticity
verification belong to `conxius-enclave-sdk`. The current core `Bip322Bridge`
implementation is structural-only and must not be used as an authenticity
decision; it does not provide cryptographic verification coverage. No BIP-322
authenticity or cryptographic coverage should be inferred from this fuzz suite.

## Local usage

Install a nightly toolchain and the pinned cargo-fuzz release used by CI:

```sh
rustup toolchain install nightly --profile minimal
cargo +nightly install cargo-fuzz --locked --version 0.13.2
```

Compile the fuzz package and one target without running it:

```sh
cargo check -p lib-conxian-core-fuzz --bins --locked
cargo +nightly fuzz check <target>
```

Run a bounded local smoke test. The same bounds are used by CI:

```sh
cargo +nightly fuzz run <target> fuzz/corpus/<target> -- \
  -max_total_time=30 \
  -timeout=5 \
  -rss_limit_mb=2048 \
  -max_len=16384 \
  -artifact_prefix=fuzz/artifacts/<target>/
```

Create the target-specific artifact directory before running the command:

```sh
mkdir -p fuzz/artifacts/<target>
```

The target implementations also apply their own narrower caps where useful:
`parse_intent` accepts at most 4 KiB, `musig2_aggregate` at most 32
compressed keys, and `anchoring_receipt` plus `proof_request_validate` at
most 16 KiB. The runner-level maximum remains 16 KiB for all targets.

Replace `<target>` with one of `parse_intent`, `musig2_aggregate`,
`anchoring_receipt`, or `proof_request_validate`.
The proof-request target performs JSON deserialization followed by structural,
policy, and evidence-binding validation when applicable; it does not perform
Groth16, BitVM2, or message-signature cryptographic verification.

## CI regression policy

`.github/workflows/fuzz-regression.yml` runs every target in a matrix on a
weekly schedule and through `workflow_dispatch`. Each matrix job compiles with
Rust nightly and runs cargo-fuzz for a maximum of 30 seconds, a five-second
per-input timeout, a 16 KiB generated-input limit, and a 2048 MiB resident-
memory limit. Fuzz failures are not allowed to pass via `continue-on-error`; a
crash, timeout, OOM, or other non-zero fuzz result fails its matrix job. Crash
artifacts and target corpus files are uploaded after every run, including
failed runs, with a 14-day retention period.

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
