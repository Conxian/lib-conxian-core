# GCP Infrastructure Documentation

## Deployment Topology
The Conxian network infrastructure has been migrated to a modular submodule architecture centered around the **Conxian Gateway**.

### Unified Entry Point
The Gateway serves as the single unified network entry point for all sovereign services:
- Bisq
- RGB
- BitVM
- Changelly

### Modular Infrastructure
GCP infrastructure code is no longer located at the root level. It is now modularized within the `gateway/` submodule to ensure audit-readiness and centralized logic.

**Path**: `gateway/infrastructure/gcp/`

### Network Routing
All service requests are routed through `/api/v1/...` endpoints managed by the Gateway binary.
