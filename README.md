# Conxian Core Libraries

This repository contains the core logic for the Conxian network, centered around the **Conxian Gateway**.

## Purpose

Centralize shared models, APIs, and core logic used by Conxian Gateway and downstream consumers (platform services, wallet integrations, and tooling).

## Status

Active development. Expect iteration as new layers, metadata, and compliance primitives are integrated.

## Audience

- Gateway engineers extending engine, API, and persistence logic.
- Platform developers building shared clients and service integrations.
- Contributors working on layer metadata, risk transparency, and observability.

## Relationship to the Conxian stack

- Serves as the shared core for Conxian Gateway and related services.
- Consumed by orchestration and product layers like `conxius-platform` and Conxius Wallet.

## Conxian Gateway

The Conxian Gateway is a unified, audit-ready Rust binary that serves as the single entry point for all sovereign services and Bitcoin/Stacks state logic. It provides a high-performance, secure bridge between sovereign services and various Bitcoin layers.

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
- **Observability**: Prometheus-compatible metrics endpoint with per-service latency and risk gauges.
- **High Performance**: Built with Rust and Actix-web for maximum reliability, memory safety, and throughput.

### Documentation

- [PRD (Product Requirements Document)](docs/PRD.md) - System overview and requirements.
- [API Reference](docs/API.md) - Detailed endpoint documentation and data models.
- [Enhancement Roadmap](docs/ENHANCEMENTS.md) - Evolution plan and current progress.
- [Architecture & Infrastructure](docs/architecture/GCP_INFRASTRUCTURE.md) - Deployment and topology details.

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

The system is organized into a modular Rust architecture:

- `gateway/src/api`: Web handlers, routing, and request validation.
- `gateway/src/engine`: Core logic, state management, background monitoring, and protocol handlers.
- `services/`: Client-side TypeScript library for interacting with the Gateway.

For detailed infrastructure information, see [docs/architecture/GCP_INFRASTRUCTURE.md](docs/architecture/GCP_INFRASTRUCTURE.md).
