# Conxian Agent Guidelines

## Conxian Gateway Architecture
The Conxian Gateway consolidates core Bitcoin/Stacks state logic (internal/engine) and API/Auth layers (internal/api) into a singular, audit-ready Rust binary.

### Deprecation Notice
- **Anya-core**: Deprecated.
- **OPSource**: Deprecated.

### Workflow Instructions
- **State Monitoring**: Point to the Conxian Gateway API at `/api/v1` for state monitoring and compliance pipes.
- **Service Access**: All sovereign services (Bisq, RGB, BitVM, Changelly) are now unified under the Gateway.
- **Infrastructure**: GCP infrastructure configurations are now located in `gateway/infrastructure/gcp/`.
