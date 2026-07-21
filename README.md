# lib-conxian-core

[![Rust CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml)
[![Version](https://img.shields.io/badge/version-0.2.12-blue.svg)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue.svg)](LICENSE)

Shared protocol primitives for the Conxian ecosystem.

## ⚠️ Vault SDK Migration

For **hardware-backed signing, attestation, and policy primitives**, use the production
[`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate (v2.0.11) instead.

> **v0.2.11 Breaking Change**: Deprecated modules (VaultSDK, Musig2, BitVM2, Wallet) have been removed.
> See [docs/MIGRATION.md](docs/MIGRATION.md) for migration instructions.

This repository provides shared protocol primitives used across the Conxian stack.

## Purpose

Provide reusable protocol-support primitives for Bitcoin-native and Conxian-aligned applications:

- Protocol primitives (types, state machines, invariants)
- Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint)
- Control models (trust tiers per CON-791)

## Core Capabilities

- **Control Models:** Trust tier taxonomy (CON-791), lifecycle states, invariant validation
- **Chain Adapters:** Universal adapter trait for Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint
- **Anchoring:** State root persistence models
- **Deployment:** Deployment manifests and verification types
- **Risk profiles:** Versioned static chain-family metadata and fail-closed invariants

## Status

**v0.2.12 Stable.** This repository is the foundational platform core. Runtime implementation for the Unified API and protocol routing belongs in the standalone [`conxian-gateway`](https://github.com/Conxian/conxian-gateway).

## Scope

This repository owns shared primitives and reusable foundations. It adheres to strict architectural boundaries (CON-700) and does not contain environment-specific side effects or network IO.

## Relationship to Other Crates

| Crate | Purpose |
|-------|---------|
| [`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) | **Production Vault SDK** - Hardware-backed signing, attestation, FROST DKG, BitVM2 |
| `lib-conxian-core` | Shared protocol primitives - control models, anchoring, chain types |
| [`conxian-gateway`](https://github.com/Conxian/conxian-gateway) | Runtime orchestration and middleware |

## Usage

Add `lib-conxian-core` to your `Cargo.toml`:

```toml
[dependencies]
lib-conxian-core = "0.2.12"

# For Vault SDK features (hardware signing, attestation)
lib-conxian-core = { version = "0.2.12", features = ["enclave"] }
```

### Quick Start

```rust
use lib_conxian_core::{
    control_model::{TrustTier, VerificationClass},
    ContractBridge,
};
use k256::ecdsa::SigningKey;

// Trust tier validation
assert!(TrustTier::Strict.is_production_allowed());

// Create signed contract call
let signing_key = SigningKey::from_slice(&private_key_bytes)?;
let signed_call = ContractBridge::create_signed_call(
    &signing_key,
    "ST1...contract-name.function-name",
    "function-name",
    vec![],
)?;
```

## Documentation

- **Migration Guide:** [docs/MIGRATION.md](docs/MIGRATION.md)
- **PRD:** [docs/PRD.md](docs/PRD.md)
- **API Reference:** [docs/API.md](docs/API.md)
- **Boundaries:** [docs/ARCHITECTURE_BOUNDARIES.md](docs/ARCHITECTURE_BOUNDARIES.md)
- **Signing Guides:** [docs/signing/README.md](docs/signing/README.md)
- **CXIP Index:** [docs/governance/CXIP_INDEX.md](docs/governance/CXIP_INDEX.md)
- **Alignment:** [docs/ALIGNMENT.md](docs/ALIGNMENT.md)

## Development

```bash
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Contact

- **Conxian Labs:** https://www.conxian-labs.com
- **Support:** support@conxian-labs.com
- **Security:** security@conxian-labs.com

## Governance & Security

This repository follows standardized governance and security defaults.

- **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)
- **Security Policy:** [SECURITY.md](SECURITY.md)
- **Support:** [SUPPORT.md](SUPPORT.md)
- **Code of Conduct:** [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)

## License

Dual-licensed under MIT and Apache 2.0. See [LICENSE](./LICENSE) for details.
