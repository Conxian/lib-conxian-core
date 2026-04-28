# Conxian Release Process & Discipline (CON-547 & CON-558)

This document defines the standardized release process for repositories in the Conxian ecosystem, ensuring consistency, reliability, and transparency for both internal teams and external contributors.

## 1. Versioning Strategy

Conxian projects follow [Semantic Versioning 2.0.0](https://semver.org/).

- **MAJOR**: Incompatible API changes.
- **MINOR**: Additive functionality in a backwards-compatible manner.
- **PATCH**: Backwards-compatible bug fixes.

## 2. Changelog Maintenance

Every repository must maintain a `CHANGELOG.md` following the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

- Every PR that changes functionality must include an update to the `[Unreleased]` section of `CHANGELOG.md`.
- When a release is tagged, the `[Unreleased]` content is moved to a new version section with the release date.

## 3. Release Lifecycle

### 3.1 Development (`dev` branch)
- Feature branches are merged into `dev` after passing CI and peer review.
- `dev` represents the integrated state for the next testnet deployment.

### 3.2 Validation (`staged` branch)
- When features in `dev` are ready for production verification, they are merged into `staged`.
- `staged` is used for final end-to-end testing and mainnet-shadowing.

### 3.3 Production (`main` branch)
- Only mainnet-ready, verified code is promoted from `staged` to `main`.
- **Mocks and stubs are strictly prohibited on `main`.**
- Promotion to `main` triggers the final production deployment and automated release tagging.

## 4. Tagging and Artifacts

- Releases are tagged in Git using the format `vX.Y.Z` (e.g., `v0.2.2`).
- Tags must be signed by a maintainer key.
- Release artifacts (binaries, SDK packages, Docker images) are generated automatically by CI upon tagging.

## 5. Rollback Playbook

In the event of a production regression:
1. **Revert**: The `main` branch is reverted to the last known good tag.
2. **Deploy**: CI triggers an automated redeploy of the previous version.
3. **Analyze**: A post-mortem is performed, and the fix is implemented on a new feature branch.
