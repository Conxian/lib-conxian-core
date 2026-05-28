# lib-conxian-core

![CI](https://github.com/Conxian/lib-conxian-core/actions/workflows/main.yml/badge.svg)

Conxian builds native application infrastructure for Bitcoin. This repository provides the core primitives for secure signing, policy enforcement, and transaction coordination on the existing Bitcoin stack.

## Purpose

Centralize shared models, APIs, and core logic used by Conxian Gateway and downstream consumers (platform services, wallet integrations, and tooling).

## Status

Active development. Expect iteration as new layers, metadata, and compliance primitives are integrated.

## Ownership

Ownership and review requirements are defined in [`CODEOWNERS`](./CODEOWNERS).

## Audience

- Gateway engineers extending engine, API, and persistence logic.
- Platform developers building shared clients and service integrations.
- Contributors working on layer metadata, risk transparency, and observability.

## Relationship to the Conxian stack

- Serves as the shared core for Conxian Gateway and related services.
- Consumed by orchestration and product layers like [`conxius-platform`](https://github.com/Conxian/conxius-platform) and [Conxius Wallet](https://github.com/Conxian/conxius-wallet).

## Architecture

The system is organized into a unified modular Rust architecture (internal/gateway) to maintain security boundaries and audit-readiness:

- `services/`: Client-side TypeScript library for interacting with the Gateway.

For detailed infrastructure information, see [docs/architecture/GCP_INFRASTRUCTURE.md](docs/architecture/GCP_INFRASTRUCTURE.md).

## Governance & Strategic Tracking

- [Risk Register](docs/governance/RISK_REGISTER.md) - Phase 5/6 risk monitoring and mitigation backlog (CON-675).
- [KPI Scorecard](docs/governance/KPI_SCORECARD.md) - Root-to-Leaf performance metrics and governance cadence (CON-674).

## Security & Mainnet Readiness (CON-145)

This repository is classified as a **P0 Mainnet Blocker**. The following security and readiness standards are enforced:

- **Fail-Closed Logic**: All cryptographic and protocol operations must fail closed.
- **No Mocks in Production**: Implementation stubs or simulated behaviors are strictly prohibited on the `main` branch.
- **Dependency Integrity**: Only audited or standard industry-vetted dependencies are permitted for core protocol logic.
- **MuSig2 Compliance**: Key aggregation follows the BIP327 standard for deterministic Taproot compatibility.
- **BitVM2 Verification**: Groth16 proof verification is performed using standard ark-works curves and verifiers.

## Release Hygiene (CON-218)

- **Versioning**: Adheres to Semantic Versioning (SemVer).
- **Changelog**: All changes are documented in [`CHANGELOG.md`](./CHANGELOG.md).
- **Licensing**: Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE).
- **Audit Trails**: Security assessments and audit reports are preserved in `docs/architecture/`.
