# lib-conxian-core

[![Rust CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml)
[![Version](https://img.shields.io/badge/version-0.2.10-blue.svg)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache%202.0-blue.svg)](LICENSE)

Shared protocol primitives for the Conxian ecosystem.

## ⚠️ Vault SDK

For **hardware-backed signing, attestation, and policy primitives**, use the production
[`conxius-enclave-sdk`](https://crates.io/crates/conxius-enclave-sdk) crate (v2.0.11) instead.

This repository provides shared protocol primitives used across the Conxian stack.

## Purpose

Provide reusable protocol-support primitives for Bitcoin-native and Conxian-aligned applications:

- Protocol primitives (types, state machines, invariants)
- Chain adapters (Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint)
- Control models (trust tiers per CON-791)

## Core Capabilities

- **Advanced Crypto:** BIP327 MuSig2 key aggregation and BitVM2 proof verification.
- **Trust Policy:** Explicit enforcement of approved bridge and messaging trust tiers (CON-791).
- **Control Models:** Canonical data structures for state proposals and ecosystem intake.
- **Chain Adapters:** Bitcoin, Stacks, Lightning, RGB, Babylon, Fedimint support.

## Status

**v0.2.10 Stable.** This repository is the foundational platform core. Runtime implementation for the Unified API and protocol routing belongs in the standalone [`conxian-gateway`](https://github.com/Conxian/conxian-gateway).

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
lib-conxian-core = "0.2.10"

# For Vault SDK features (hardware signing, attestation)
lib-conxian-core = { version = "0.2.10", features = ["enclave"] }
```

### Quick Start

```rust
use lib_conxian_core::{
    Wallet,
    control_model::{TrustTier, VerificationClass},
    musig2::Musig2Participant,
};

// Use protocol primitives
let participant = Musig2Participant::new();
let (pk, _) = participant.x_only_public_key();

// Trust tier validation
assert!(TrustTier::Strict.is_production_allowed());
```

## Documentation

- **PRD:** [docs/PRD.md](docs/PRD.md)
- **API Reference:** [docs/API.md](docs/API.md)
- **Boundaries:** [docs/ARCHITECTURE_BOUNDARIES.md](docs/ARCHITECTURE_BOUNDARIES.md)
- **CXIP Index:** [docs/CXIP_INDEX.md](docs/CXIP_INDEX.md)
- **Research:** [docs/UNIVERSAL_SUPPORT_RESEARCH.md](docs/UNIVERSAL_SUPPORT_RESEARCH.md)

## Development

```bash
cargo build
cargo test --workspace
```

## Contact

- **Conxian Labs:** https://www.conxian-labs.com
- **Support:** support@conxian-labs.com
- **Security:** security@conxian-labs.com

## Governance & Security

This repository follows standardized governance and security defaults.

- **Contributing:** [CONTRIBUTING.md](CONTRIBUTING.md)
- **Security Policy:** [SECURITY.md](SECURITY.md)
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)

## License

Dual-licensed under MIT and Apache 2.0. See [LICENSE](./LICENSE) for details.
