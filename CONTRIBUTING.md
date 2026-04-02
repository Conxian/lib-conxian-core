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

## Pull Request Process

1. Create a new branch for your feature or bug fix.
2. Commit your changes with descriptive commit messages.
3. Ensure all tests pass.
4. Update relevant documentation.
5. Submit a pull request to the `main` branch.

All PRs require review from at least one core maintainer before merging.

## Security

If you discover a security vulnerability, please refer to our [Security Policy](SECURITY.md) for reporting instructions.
