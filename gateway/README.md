# Conxian Gateway

The Conxian Gateway is a production-grade infrastructure component that enables secure transaction coordination and policy-aware routing for native Bitcoin applications. It serves as the reference implementation and primary routing layer for the [Conxian Vault SDK (`lib-conxian-core`)](../README.md).

## Features

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

## Documentation

- [PRD (Product Requirements Document)](../docs/PRD.md) - System overview and requirements.
- [API Reference](../docs/API.md) - Detailed endpoint documentation and data models.
- [Enhancement Roadmap](../docs/ENHANCEMENTS.md) - Evolution plan and current progress.
- [Architecture & Infrastructure](../docs/architecture/GCP_INFRASTRUCTURE.md) - Deployment and topology details.
- [BOS Ownership Map](../docs/architecture/BOS_OWNERSHIP_MAP.md) - Canonical repo and runtime ownership.

## Getting Started

### Prerequisites

- Rust (Latest stable version)
- Cargo

### Run the Gateway

To run the gateway locally:

```bash
cargo run
```

The gateway listens on port 8080 by default. Use `RUST_LOG=info` for detailed logging.

## Testing

The Gateway includes a comprehensive suite of integration tests covering all services and system endpoints.

```bash
cargo test
```

## Architecture

The gateway is organized into a modular Rust architecture to maintain security boundaries and audit-readiness:

- `src/api`: Web handlers, routing, and request validation.
- `src/engine`: Core logic, state management, background monitoring, and protocol handlers.
- `src/lib.rs`: Entry point and module declarations.

For detailed infrastructure information, see [docs/architecture/GCP_INFRASTRUCTURE.md](../docs/architecture/GCP_INFRASTRUCTURE.md).
