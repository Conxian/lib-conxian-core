# lib-conxian-core (Vault SDK)

![CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)

Conxian builds native application infrastructure for Bitcoin. `lib-conxian-core` is the foundational Vault SDK, providing the core primitives for secure signing, policy enforcement, and transaction coordination on the existing Bitcoin stack.

## Purpose

The Vault SDK centralizes shared models, APIs, and core logic used by the Conxian Gateway and downstream consumers (platform services, wallet integrations, and institutional tooling). It serves as the canonical home for shared capability interfaces and safety primitives.

## Strategic Role

As defined in the [Portfolio Map](docs/architecture/PORTFOLIO_MAP.md), this repository is the **primary commercial primitive** for the Conxian ecosystem. It is classified as **P0 - Hardened**, enforcing BIP-aligned, fail-closed logic with zero-mock production policies.

## Status

Active development. Version **v0.2.5** is synchronized with the latest strategic SDKs (Bitcoin v0.32, BDK v0.30, RGB v0.12.0).

## Ownership

Ownership and review requirements are defined in [`CODEOWNERS`](./.github/CODEOWNERS) and the [BOS Ownership Map](docs/architecture/BOS_OWNERSHIP_MAP.md).

## Audience

- **Protocol Engineers** extending shared signing and policy primitives.
- **Platform Developers** building shared clients and service integrations.
- **Security Auditors** verifying the integrity of cross-layer safety logic.

## Relationship to the Conxian stack

- **Vault SDK**: The primary primitive for secure interaction with Bitcoin and its layers.
- **[Conxian Gateway](./gateway/README.md)**: A downstream consumer providing unified API routing and protocol-specific orchestration.
- **[Conxius Wallet](https://github.com/Conxian/conxius-wallet)**: A reference asset management client leveraging the Vault SDK.

## Getting Started

### Prerequisites

- Rust (Latest stable version)
- Cargo

### Library Usage

To use the Vault SDK in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
lib-conxian-core = { git = "https://github.com/Conxian/lib-conxian-core.git", version = "0.2.5" }
```

### Testing

Run the core protocol tests:

```bash
cargo test
```

## Security & Mainnet Readiness (CON-145)

This repository is a **P0 Mainnet Blocker**. The following security standards are strictly enforced:

- **Fail-Closed Logic**: All cryptographic and protocol operations must fail closed.
- **No Mocks in Production**: Implementation stubs or simulated behaviors are strictly prohibited on the `main` branch.
- **Dependency Integrity**: Only audited or standard industry-vetted dependencies are permitted for core protocol logic.
- **MuSig2 Compliance**: Key aggregation follows the BIP327 standard for deterministic Taproot compatibility.
- **BitVM2 Verification**: Groth16 proof verification is performed using standard ark-works curves and verifiers.

## Governance & Strategic Tracking

- [Risk Register](docs/governance/RISK_REGISTER.md) - Phase 5/6 risk monitoring and mitigation (CON-675).
- [KPI Scorecard](docs/governance/KPI_SCORECARD.md) - Performance metrics and governance cadence (CON-674).
- [Release Process](docs/RELEASE_PROCESS.md) - Standardized release discipline and versioning (CON-547).

## License

Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE).
