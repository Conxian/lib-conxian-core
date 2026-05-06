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

## Conxian Gateway

The Conxian Gateway is a production-grade infrastructure component that enables secure transaction coordination and policy-aware routing for native Bitcoin applications. It provides a high-performance, secure bridge between sovereign services and various Bitcoin layers.

### Features

- **Unified API**: All services and layers are accessible via the `/api/v1/*` prefix.
- **Sovereign Services**: Full integration for Bisq (P2P), RGB (Client-side), BitVM (Optimistic), and Changelly (Centralized).
- **Extensive Bitcoin Layer Support**:
  - **L2s & Sidechains**: Stacks, Lightning, Liquid, Rootstock, BOB, Merlin, Botanix, B² Network, Citrea, Bitlayer, Alpen, Zulu, Bison, Hemi.
  - **Infrastructure & Protocols**: Babylon (Staking), Nubit (DA), Lorenzo (Staking), Taproot Assets.
- **Dynamic Monitoring**: Real-time tracking of block heights, TVL, channel capacity, and service latency.
- **Automated TVL Aggregation**: Centralized monitoring of Total Value Locked across the entire ecosystem.
- **Risk Transparency**: Detailed trust model and risk metadata aligned with research from [bitcoinlayers.org](https://bitcoinlayers.org/).
- **Advanced Compliance**: Integrated address verification (`/compliance/check`) and network integrity monitoring.
- **Agentic Surface (MCP)**: Read-only audit layer for AI agents to verify telemetry, proofs, and financials.
- **Observability**: Prometheus-compatible metrics endpoint with per-service latency and risk gauges.
- **High Performance**: Built with Rust and Actix-web for maximum reliability, memory safety, and throughput.

### Documentation

- [PRD (Product Requirements Document)](docs/PRD.md) - System overview and requirements.
- [API Reference](docs/API.md) - Detailed endpoint documentation and data models.
- [Enhancement Roadmap](docs/ENHANCEMENTS.md) - Evolution plan and current progress.
- [Architecture & Infrastructure](docs/architecture/GCP_INFRASTRUCTURE.md) - Deployment and topology details.
- [BOS Ownership Map](docs/architecture/BOS_OWNERSHIP_MAP.md) - Canonical repo and runtime ownership.
- [Portfolio Map](docs/architecture/PORTFOLIO_MAP.md) - Repository classification and review standards.
- [Flagship Repositories](docs/architecture/FLAGSHIP_REPOS.md) - Pinned repo selection and narrative order.

### Getting Started

#### Prerequisites

- Rust (Latest stable version)
- Cargo

#### Run the Gateway

To run the gateway locally:

```bash
cd gateway
cargo run
```

The gateway listens on port 8080 by default. Use `RUST_LOG=info` for detailed logging.

### Testing

The Gateway includes a comprehensive suite of 59 integration tests covering all services and system endpoints.

```bash
cd gateway
cargo test
```

## Architecture

The system is organized into a unified modular Rust architecture (internal/gateway) to maintain security boundaries and audit-readiness:

- `gateway/src/api`: Web handlers, routing, and request validation.
- `gateway/src/engine`: Core logic, state management, background monitoring, and protocol handlers.
- `services/`: Client-side TypeScript library for interacting with the Gateway.

For detailed infrastructure information, see [docs/architecture/GCP_INFRASTRUCTURE.md](docs/architecture/GCP_INFRASTRUCTURE.md).

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
