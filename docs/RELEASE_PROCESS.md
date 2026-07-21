# lib-conxian-core Release Process (CON-218, CON-1508)

This document defines the safe release process for the protocol primitives
crate. The root `[package].version` in `Cargo.toml` is authoritative. The
release guard requires that version to match the root package entry in
`Cargo.lock`, the current-version markers in `README.md`, and the latest
non-`Unreleased` heading in `CHANGELOG.md`.

Historical version references in migration notes and older changelog entries
are allowed. Only explicitly current markers are checked.

## 1. Prepare a release

1. Update `[package].version` in the root `Cargo.toml`.
2. Update the README badge, stable status, and dependency examples to the same
   version.
3. Add the release heading and date below `## [Unreleased]` in `CHANGELOG.md`.
4. Run `cargo check` or another Cargo command so the root package entry in
   `Cargo.lock` is synchronized.
5. Run the local guard and required checks:

   ```bash
   python scripts/verify_release_hygiene.py
   python -m unittest discover -s scripts/tests -p 'test_*.py'
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --locked
   ```

Do not manually publish, tag, or create a GitHub Release while validating a
change to the release flow.

## 2. Normal tag release

After the parity change is merged, create and push exactly
`v{Cargo.toml version}`. For example, when `Cargo.toml` says `0.2.12`:

```bash
git tag -a v0.2.12 -m "Release v0.2.12"
git push origin v0.2.12
```

The tag workflow stops before publication when local parity fails or when the
tag is not exactly `v{Cargo.toml version}`. It then checks the exact crate and
version on crates.io. A confirmed existing version is reused safely; a
confirmed HTTP 404 permits publication; all other API, HTTP, timeout, or
network errors fail closed.

When publication is needed, `CARGO_REGISTRY_TOKEN` must be present. The
workflow never prints the token and fails before `cargo publish` when the
secret is missing. After a successful publish, or after a safe retry where the
exact version is already present, the workflow polls crates.io a bounded number
of times before considering GitHub Release creation.

The GitHub Release step also validates the exact tag. If a release already
exists for that tag, the workflow skips creation rather than attempting to
create a duplicate.

## 3. Dry-run and manual operation

Use the `workflow_dispatch` `dry_run` input for validation. It defaults to
`true` and runs `cargo publish --dry-run`; it does not query publication state,
publish a crate, create a tag, or create a GitHub Release.

A manual real-publish run is still protected by local parity, token, crates.io
state, bounded propagation checks, and the same fail-closed publication logic.
Tag-triggered runs are the only runs that create GitHub Releases.

## 4. Safe retry and recovery

### Missing token

If the first publication is needed and `CARGO_REGISTRY_TOKEN` is missing, the
workflow fails before publication and no GitHub Release is created. Configure
the repository secret and rerun the same exact tag workflow.

### Delayed crates.io propagation

The workflow treats only an API-confirmed HTTP 404 as “not published.” A
timeout, rate limit, server error, malformed response, or identity mismatch is
unknown state and fails closed. If a publish succeeded but propagation is slow,
rerun the same tag after crates.io confirms the exact version; the preflight
will skip republishing safely.

### Publish succeeded, GitHub Release failed

Rerun the same exact tag workflow. The existing crate/version is validated and
republishing is skipped. If no GitHub Release exists, the workflow can create
the exact-tag release after the crates.io confirmation gate. If the release
already exists, it is recognized and no duplicate is created.

### Already-published version

An exact crates.io response is accepted only when both the crate name and
version in the response match the root package identity. The workflow skips
`cargo publish` only after that validation. It never treats an arbitrary API or
network failure as proof that a version is absent.

### Existing GitHub Release

The exact tag is queried before release creation. An existing release with that
tag is a safe idempotent terminal state for this workflow; it is not blindly
created again. Repair release text or assets separately if needed, without
reusing a different tag for the same package version.

### Incorrect version already published

Crates.io versions are immutable. Do not try to overwrite or delete an
incorrectly published version. Correct `Cargo.toml`, `Cargo.lock`, README, and
changelog parity, then release a new version with the exact matching tag. Treat
the incorrect publication as a permanent historical artifact and record the
recovery in the changelog or release notes.

## 5. Versioning rules

- **Major:** Breaking changes to shared data models or public protocol APIs.
- **Minor:** New protocol primitives, chain support, or additive public APIs.
- **Patch:** Security fixes, bug fixes, and non-breaking hygiene improvements.

Every release must preserve the Cargo/lock/README/changelog/tag/crates.io
identity invariant. Downstream consumers should use the published crate or a
pinned release tag.
