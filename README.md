# Conxian Core Libraries

This repository contains the core logic for the Conxian network, centered around the **Conxian Gateway**.

## Conxian Gateway

The Conxian Gateway is a unified, audit-ready Rust binary that serves as the single entry point for all sovereign services and Bitcoin/Stacks state logic.

### Features

- **Unified API**: All services (Bisq, RGB, BitVM, Changelly) are accessible via `/api/v1/\*`.
- **Monitoring & Compliance**: Integrated health, status, compliance, and metrics endpoints.
- **High Performance**: Built with Rust and Actix-web for maximum reliability and throughput.

### Getting Started

To run the gateway locally:

```bash
cd gateway
cargo run
```

The gateway listens on port 8080 by default.

### Testing

Run the test suite:

```bash
cd gateway
cargo test
```

## Architecture

For detailed architecture information, see [docs/architecture/GCP_INFRASTRUCTURE.md](docs/architecture/GCP_INFRASTRUCTURE.md).
