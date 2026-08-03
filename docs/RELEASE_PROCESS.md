# lib-conxian-core Release Process (CON-218, CON-1508)

This document defines the fail-closed release process for the protocol
primitives crate. The source tree may intentionally contain a version that has
not been published yet; normal CI must still be able to validate that state.

## Authority and parity contract

`Cargo.toml` `[package].version` is the authoritative candidate version. It
must be a valid SemVer 2.0.0 value in the form
`X.Y.Z[-prerelease][+build]`. Stable versions, prerelease versions such as
`0.3.1-rc.1`, and build metadata such as `0.3.1+build.1` are accepted. Cargo
accepts these forms when packaging a crate; Cargo ignores build metadata while
resolving dependency requirements, so this process treats it as informational
and compares the complete string exactly everywhere it is surfaced.

The root `lib-conxian-core` entry in `Cargo.lock` must mirror the complete
version. The release guard also requires the structured current-version markers
in `README.md` to match:

- the Version badge;
- the `Stable` or `Pre-release` status line; and
- both `lib-conxian-core` dependency examples.

`CHANGELOG.md` must contain an exact release heading for the candidate,
`## [vX.Y.Z[-prerelease][+build]]` (an optional ` - YYYY-MM-DD` suffix is
allowed). References in the `Unreleased` section and historical release
sections do not satisfy this requirement and do not cause false positives for
another candidate.

The version remains immutable after publication. If the exact complete version
string is present on crates.io, do not edit the source to republish it or rerun
`cargo publish` for that version. The guard requires the registry response to
match the complete string, including prerelease and build metadata. Fix
release metadata through the recovery flow, or prepare a new patch version
when the published artifact itself is wrong.

## Rust and optional enclave SDK coordination

- Keep the package `rust-version`, the explicit CI toolchain, and
  [docs/COMPATIBILITY.md](COMPATIBILITY.md) synchronized.
- The supported floor for `lib-conxian-core` is Rust `1.91+` for both default
  and optional `enclave` builds.
- The optional `enclave` dependency is coordinated with
  `conxius-enclave-sdk 2.0.11`; review its declared MSRV and resolved Alloy /
  `ruint` graph before changing the SDK version.
- Before publishing a release or SDK upgrade, run locked default and
  all-feature `check`, `test`, and all-target `clippy -D warnings` coverage.

## Release preparation

Prepare the source tree before choosing a release operation:

1. Update `[package].version` in the root `Cargo.toml`.
2. Update the README badge, stable/pre-release status, and dependency examples
   to the same version.
3. Add the release heading and date below `## [Unreleased]` in `CHANGELOG.md`.
4. Run `cargo check --workspace` or another Cargo command so the root package
   entry in `Cargo.lock` is synchronized.
5. Run the local source-only guard and required checks below.

Do not manually publish, tag, or create a GitHub Release while validating a
change to the release flow.

## Lifecycle phases

The standard-library guard, `scripts/verify_release_version.py`, has explicit
phases:

| Phase | Required state | Used by |
| --- | --- | --- |
| `source-only` | Cargo manifest/lock, README markers, and changelog are locally consistent. No public artifact is required. | Pull requests and `main` CI; manual dry-runs. |
| `pre-publish` | Valid `vX.Y.Z[-prerelease][+build]` tag matches Cargo, the GitHub tag exists and points at the checked-out commit, and the exact crate version and GitHub Release are both absent. | Tag pushes and manual `publish`. |
| `post-publish` | The exact candidate exists on crates.io and the tag/source identity still matches. A matching GitHub Release may be absent. | After successful `cargo publish`; manual `release-only` recovery. |
| `post-release` | The exact candidate exists on crates.io and a matching, non-draft GitHub Release exists. | Final workflow verification. |

The public registry and GitHub APIs are checked without printing credentials.
The GitHub API client uses `GITHUB_API_URL` when supplied by the runner and
falls back to `https://api.github.com`.
After `cargo publish`, crates.io propagation is polled with a bounded retry
window and an actionable failure message. A timeout is never treated as proof
that publication failed. The workspace release has an additional dependency
ordering rule: `lib-conxian-core` is published and confirmed first, then the
workflow waits for its registry/index entry before dry-running and publishing
`lib-conxian-core-enclave`.

## Pre-tag checklist

Run these commands from the release commit before creating a tag:

```bash
cargo check --workspace
cargo check --workspace --all-features --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --all-features --locked
python scripts/verify_release_version.py --phase source-only
python scripts/verify_release_hygiene.py
python -m unittest discover -s scripts/tests -p 'test_*.py'
```

Confirm that the changelog entry describes the intended release boundary, then
create and push the tag from the exact commit that passed the checks:

```bash
VERSION=0.3.1
TAG="v${VERSION}"
git fetch origin main --tags
git switch main
git pull --ff-only origin main
git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"
```

At this revision, `Cargo.toml`, `Cargo.lock`, the current README markers, and the
latest release heading all identify `0.3.1`, so the normal release tag is exactly
`v0.3.1`. Future releases must derive the tag from the then-current authoritative
Cargo version instead of copying this example.

The tag-push workflow checks the tag against Cargo and the GitHub tag API before
it can publish. Do not create a tag solely to make the public registry version
match the source; the release decision and changelog boundary must be
intentional.

## Dry-run and manual operation

Use the `workflow_dispatch` `mode=dry-run` input for validation. It runs source
parity and `cargo publish --dry-run --locked -p lib-conxian-core`; it does not
publish, create a tag, query publication state, or create a GitHub Release.
The add-on dry-run is intentionally deferred to the real publish path because
its `lib-conxian-core = "0.3.1"` registry dependency cannot resolve until Core
has been published and the crates.io/index entry has propagated. The
`release_tag` input is optional in dry-run mode.

A manual real-publish run uses `mode=publish` with an existing, matching
`release_tag`. It is still protected by local parity, tag/source identity,
token, crates.io state, bounded propagation checks, and the same fail-closed
publication logic as a tag push. `mode=release-only` is the recovery path after
the exact Core and add-on versions are already published; it never runs
`cargo publish`. Recovery verifies both registry candidates before creating or
accepting the GitHub Release.
Tag-triggered runs are the only automatic path that creates GitHub Releases.

## Fail-closed execution order

The publishing workflow follows this order and stops on any failed step:

1. Check out the tag. For manual operations, require an explicit `release_tag`,
   fetch that existing tag, and check it out detached.
2. Run local source parity and SemVer-aware tag/source identity checks.
3. In `pre-publish`, reject an existing crates.io candidate or GitHub Release.
4. Require `CARGO_REGISTRY_TOKEN`, then run `cargo publish --locked -p
   lib-conxian-core` exactly once. If Cargo exits non-zero, the workflow checks
   the exact Core candidate before failing; only a confirmed published
   candidate may continue to the add-on.
5. In `post-publish`, poll until the exact Core crate version is visible.
6. Run `cargo publish --dry-run --locked -p lib-conxian-core-enclave` with
   bounded retries for the specific Core index-propagation error. Any other
   add-on packaging failure is fatal.
7. Publish the add-on with `cargo publish --locked -p
   lib-conxian-core-enclave`, verify its exact registry version, and treat only
   an exact already-published candidate as a safe retry state.
8. Create the GitHub Release only after both package registry verifications. The creation
   step is idempotent when the matching release already exists.
9. Run `post-release` parity verification.

Manual `dry-run` runs only source parity and the Core package dry-run.
Both publication paths use Cargo's lockfile exactly as checked in, so dependency
resolution cannot silently change between verification and upload. Manual
workflow inputs are named `mode` and `release_tag`: `mode` is required and may
be `dry-run`, `publish`, or `release-only`; `release_tag` is optional for
`dry-run` and must be an existing matching
`vX.Y.Z[-prerelease][+build]` tag for the other two modes. Manual `publish`
requires both `mode=publish` and an existing matching `release_tag`. Manual
`release-only` requires the same tag input, verifies the already-published
candidate, and never runs `cargo publish`.

## Safe retry matrix

| Failure point | Safe action | Unsafe action |
| --- | --- | --- |
| Before upload is accepted (for example, dry-run, missing token, or a clearly failed pre-upload request) | Fix the cause and rerun `publish` after `pre-publish` confirms the exact version is absent. | Bypassing the guard or manually creating a release. |
| `cargo publish` fails and crates.io does not contain the exact candidate | Inspect the error, wait for any transient registry response, then rerun `publish` only after the preflight still reports absence. | Assuming every timeout means no upload occurred. |
| Publication result is ambiguous or the workflow times out | Query crates.io for the exact version. If it exists, stop publication attempts and use `release-only`; if it is still absent, rerun preflight rather than guessing. | Repeating `cargo publish` immediately. |
| Crates.io contains the candidate but GitHub Release creation failed | Run `release-only` with the exact existing tag. It verifies registry/tag parity, creates the release if missing, and performs post-release verification. | Rerunning `publish`; crates.io versions are immutable. |
| GitHub Release already exists | Run `release-only` if final verification is needed; release creation is idempotent. | Deleting or recreating the published crate. |

### Missing token

If the first publication is needed and `CARGO_REGISTRY_TOKEN` is missing, the
workflow fails before publication and no GitHub Release is created. Configure
the repository secret and rerun the same exact tag workflow.

### Delayed crates.io propagation

Only an API-confirmed HTTP 404 means that the exact candidate is absent. A
timeout, rate limit, server error, malformed response, or identity mismatch is
unknown state and fails closed. If a publish succeeded but propagation is slow,
query crates.io again and use `release-only` once the exact version is visible;
do not immediately repeat `cargo publish`.

### Incorrect version already published

Crates.io versions are immutable. Do not try to overwrite or delete an
incorrectly published version. Correct `Cargo.toml`, `Cargo.lock`, README, and
changelog parity, then release a new version with the exact matching tag. Treat
the incorrect publication as a permanent historical artifact and record the
recovery in the changelog or release notes.

## Exact release-only recovery

Use this procedure when publication succeeded but the workflow did not finish
the GitHub Release step:

1. Confirm the exact candidate is visible on crates.io and identify the tag,
   for example the current release tag `v0.3.1` or a future prerelease tag
   such as `v0.3.1-rc.1`.
2. Start the **Publish to crates.io** workflow manually with:
   - `mode`: `release-only`
   - `release_tag`: the existing matching tag, for example `v0.3.1` or `v0.3.1-rc.1`
3. If using the CLI, the equivalent invocation is:

   ```bash
   gh workflow run "Publish to crates.io" \
     --ref main \
     -f mode=release-only \
     -f release_tag=v0.3.1
   ```

4. Wait for `post-publish` registry verification, idempotent release creation,
   and `post-release` verification to pass. No crates.io token is needed for
   this recovery path, and no publication is attempted.

If the tag does not exist, points at different source, or the registry does not
contain the exact candidate, stop and resolve that state before retrying. Do
not force-move a published tag or change the immutable package version.

## Versioning rules

- **Major:** Breaking changes to shared data models or public protocol APIs.
- **Minor:** New protocol primitives, chain support, or additive public APIs.
- **Patch:** Security fixes, bug fixes, and non-breaking hygiene improvements.

Every release must preserve the Cargo/lock/README/changelog/tag/crates.io
identity invariant. Downstream consumers should use the published crate or a
pinned release tag.

## Post-release follow-up

After the workflow passes, review the generated GitHub Release, notify
downstream consumers (`conxian-gateway`, `conxius-platform`), and retain the
workflow run as the release audit trail.
