# lib-conxian-core Release Process (CON-218)

This document defines the mandatory steps for releasing a new version of the Vault SDK and protocol primitives.

## 1. Pre-requisites
- 100% pass rate on all workspace tests (`cargo test --workspace`).
- No compilation warnings (`cargo check`).
- README.md and PRD.md versions are synchronized with Cargo.toml.
- CHANGELOG.md is updated with all notable changes since the last release.
- Audit reports (docs/architecture/) are updated if structural changes occurred.

## 2. Versioning Rules
- **Major**: Breaking changes to the Vault SDK (src/sdk_primitive.rs) or shared data models.
- **Minor**: New protocol primitives, new Bitcoin layer support, or additive SDK features.
- **Patch**: Security fixes, bug fixes, or non-breaking hygiene improvements.

## 3. Execution Sequence
1. Create a release branch (e.g., `release/v0.2.6`).
2. Update version in root `Cargo.toml`.
3. Update `CHANGELOG.md` with the release date and summary.
4. Run `cargo build` to update `Cargo.lock`.
5. Submit for final review (P0 Mainnet Blocker gate).
6. Tag the commit once merged: `git tag -a v0.2.6 -m "Release v0.2.6"`.
7. Push tag: `git push origin v0.2.6`.

## 4. Post-Release
- Publish GitHub Release artifact with tag description.
- Notify downstream consumers (conxian-gateway, conxius-platform).
