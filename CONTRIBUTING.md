# Contributing to Conxian

Thank you for your interest in contributing to Conxian! We welcome contributions from the community to help make the Conxian network and its Gateway more secure, efficient, and user-friendly.

## Code of Conduct

All contributors are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Node.js](https://nodejs.org/) (for TypeScript client changes)

### Local Setup

1. Fork and clone the repository.
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Run tests:
   ```bash
   cargo test
   ```

## Development Standards

- **Rust**:
  - Follow standard Rust naming conventions.
  - Run `cargo fmt` before submitting a PR.
  - Run `cargo clippy` to check for common mistakes and improvements.
- **TypeScript**:
  - Ensure all types are properly defined.
  - Use `npm run lint` if applicable.

## Development Workflow

### Security First

- **No Secrets**: Never commit API keys, private keys, or credentials.
- **Ignore Rules**: Adhere to the `.gitignore` rules. Do not bypass them or use `git add --force` for sensitive files.
- **Verification**: Ensure all changes are verified and do not introduce unintended public/private boundary issues.

## Pull Request Process

1. Create a new branch for your feature or bug fix.
2. Commit your changes with descriptive commit messages.
3. Ensure all tests pass.
4. Update relevant documentation.
5. Submit a pull request to the `main` branch.

All PRs require review from at least one core maintainer before merging.


## Governance Support Routing

- For support and issue-routing guidance, use [SUPPORT.md](SUPPORT.md).
- For vulnerability handling, follow [SECURITY.md](SECURITY.md) and avoid public disclosure.

## Sensitive File Changes

Changes to governance-sensitive files require CODEOWNERS review:

- `CODEOWNERS`
- `SECURITY.md`
- `SUPPORT.md`
- `.github/ISSUE_TEMPLATE/**`
- `.github/PULL_REQUEST_TEMPLATE*`
- `.github/workflows/**`
- `.github/release.yml`

## Security

If you discover a security vulnerability, please refer to our [Security Policy](SECURITY.md) for reporting instructions.

## Release Discipline & Versioning

- **Semantic Versioning**: We use [SemVer](https://semver.org/).
  - `MAJOR` version for incompatible API changes.
  - `MINOR` version for functionality in a backwards compatible manner.
  - `PATCH` version for backwards compatible bug fixes.
- **Changelog**: All changes must be recorded in `CHANGELOG.md`.
- **Tags**: Releases must be tagged in Git (e.g., `v0.2.0`).
- **Licensing**: This project is dual-licensed under MIT and Apache 2.0. By contributing, you agree that your contributions will be licensed under these terms.
- **Mainnet Safety**: Code promoted to the `main` branch must be mainnet-ready. Non-production behavior (stubs, mocks) is restricted to `dev` or `staged` branches.
