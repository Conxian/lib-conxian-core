# lib-conxian-core / Vault SDK

[![Rust CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml)
[![Version](https://img.shields.io/badge/version-0.2.9-blue.svg)](docs/governance/CHANGELOG.md)
[![Governance](https://img.shields.io/badge/governance-standard-green.svg)](docs/governance/)

Shared protocol primitives and foundational library for the broader Conxian ecosystem.

## Sovereign First

Conxian is built on the principle of individual and enterprise sovereignty. The Vault SDK ensures that keys never leave the hardware boundary and all operations are verified against immutable protocol rules.

## Purpose

Provide the **Vault SDK** and reusable protocol-support primitives for Bitcoin-native and Conxian-aligned applications. This repository is a **flagship credibility surface** for Conxian's core protocol logic.

## Core Capabilities

- **Vault SDK:** Hardware-backed signing and policy-aware transaction coordination.
- **Advanced Crypto:** BIP327 MuSig2 key aggregation and BitVM2 proof verification.
- **Trust Policy:** Explicit enforcement of approved bridge and messaging trust tiers (CON-791).
- **Control Models:** Canonical data structures for state proposals and ecosystem intake.

## Status

**v0.2.10 Stable.** This repository is the foundational platform core. Runtime implementation for the Unified API and protocol routing belongs in the standalone [`conxian-gateway`](https://github.com/Conxian/conxian-gateway).

## Scope

This repository owns shared primitives and reusable foundations. It adheres to strict architectural boundaries (CON-700) and does not contain environment-specific side effects or network IO.

## Governance relation

This repository is maintained by **Conxian Labs**. It provides the public-facing primitives and commercial SDK for the Conxian stack.

## Usage

Add `lib-conxian-core` to your `Cargo.toml`:

```toml
[dependencies]
lib-conxian-core = { git = "https://github.com/Conxian/lib-conxian-core.git", tag = "v0.2.10" }
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
- **CXIP Index:** [docs/governance/CXIP_INDEX.md](docs/governance/CXIP_INDEX.md)
- **Research:** [docs/UNIVERSAL_SUPPORT_RESEARCH.md](docs/UNIVERSAL_SUPPORT_RESEARCH.md)

## Development

```bash
cargo build
cargo test --workspace
```

## Governance & Security

This repository follows standardized governance and security defaults.

- **Contributing:** [docs/governance/CONTRIBUTING.md](docs/governance/CONTRIBUTING.md)
- **Security Policy:** [docs/governance/SECURITY.md](docs/governance/SECURITY.md)
- **Support:** [docs/governance/SUPPORT.md](docs/governance/SUPPORT.md)
- **Code of Conduct:** [docs/governance/CODE_OF_CONDUCT.md](docs/governance/CODE_OF_CONDUCT.md)
- **Changelog:** [docs/governance/CHANGELOG.md](docs/governance/CHANGELOG.md)

## License

Dual-licensed under MIT and Apache 2.0. See [LICENSE](./LICENSE) for details.
