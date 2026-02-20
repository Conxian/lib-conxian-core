# GCP Infrastructure Documentation

## Deployment Topology
The Conxian network infrastructure has been migrated to a modular submodule architecture centered around the **Conxian Gateway**.

### Unified Entry Point
The Gateway serves as the single unified network entry point for all sovereign services and Bitcoin Layer 2s:
- **Sovereign Services:** Bisq, RGB, BitVM, Changelly
- **Bitcoin Layers:** Stacks, Lightning Network, Liquid, Rootstock

### Modular Infrastructure
GCP infrastructure code is no longer located at the root level. It is now modularized within the `gateway/` submodule to ensure audit-readiness and centralized logic.

**Path**: `gateway/infrastructure/gcp/`

### Network Routing
All service requests are routed through `/api/v1/...` endpoints managed by the Gateway binary. The infrastructure supports high availability with multiple replicas and automated health monitoring via Kubernetes probes.

### Monitoring & Metrics
- **Health Probes:** Liveness and readiness probes are configured at `/api/v1/health`.
- **Metrics:** Prometheus-compatible metrics are exposed at `/api/v1/metrics`.
