# lib-conxian-core / Vault SDK

[![Rust CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml)
[![Version](https://img.shields.io/badge/version-0.2.5-blue.svg)](docs/governance/CHANGELOG.md)

Shared protocol primitives and foundational library for the broader Conxian ecosystem.

## Purpose

Provide the **Vault SDK** and reusable protocol-support primitives for Bitcoin-native and Conxian-aligned applications.

## Core Capabilities

- **Vault SDK:** Hardware-backed signing and policy-aware transaction coordination.
- **Advanced Crypto:** BIP327 MuSig2 key aggregation and BitVM2 proof verification.
- **Trust Policy:** Explicit enforcement of approved bridge and messaging trust tiers.
- **Control Models:** Canonical data structures for state proposals and ecosystem intake.

## Status

**v0.2.5 Stable.** This repository is a shared protocol and platform foundation. Runtime implementation for the Unified API and protocol routing belongs in [`conxian-gateway`](https://github.com/Conxian/conxian-gateway).

## Scope

This repository owns shared primitives and reusable foundations. It should not become a dumping ground for unrelated business logic, public narrative copy, or unbounded platform concerns.

## Governance relation

This repository is maintained by Conxian-Labs. It supports the broader Conxian ecosystem, but it is not itself the DAO-facing governance authority.

## Usage

Add `lib-conxian-core` to your `Cargo.toml`:

```toml
[dependencies]
lib-conxian-core = { git = "https://github.com/Conxian/lib-conxian-core.git", tag = "v0.2.5" }
```

### Quick Start (Vault SDK)

```rust
use lib_conxian_core::{VaultSDK, SigningPolicy, Wallet};

// Initialize the SDK
let sdk = VaultSDK::new(wallet, SigningPolicy::default());

// Sign a transaction after policy verification
let result = sdk.sign_with_policy("tx_id_123", 100_000, "destination_address");
```

## Documentation

- **PRD:** [docs/PRD.md](docs/PRD.md)
- **API Reference:** [docs/API.md](docs/API.md)
- **Boundaries:** [docs/ARCHITECTURE_BOUNDARIES.md](docs/ARCHITECTURE_BOUNDARIES.md)

## Development

```bash
cargo build
cargo test
```

## Governance

This repository is maintained by Conxian Labs. It provides the public-facing primitives for the Conxian protocol stack.

- **Contributing:** [docs/governance/CONTRIBUTING.md](docs/governance/CONTRIBUTING.md)
- **Code of Conduct:** [docs/governance/CODE_OF_CONDUCT.md](docs/governance/CODE_OF_CONDUCT.md)
- **Support:** [docs/governance/SUPPORT.md](docs/governance/SUPPORT.md)
- **Security policy:** [docs/governance/SECURITY.md](docs/governance/SECURITY.md)
- **Changelog:** [docs/governance/CHANGELOG.md](docs/governance/CHANGELOG.md)

## Security

Vulnerability disclosure: [SECURITY.md](docs/governance/SECURITY.md) or `security@conxian-labs.com`.

## License

See [LICENSE](./LICENSE).
