# SLOs and Telemetry Baseline for Core Repositories (CON-560)

This document defines the Service Level Objectives (SLOs) and telemetry standards for the Conxian core repository set.

## 1. Reliability SLOs

| Metric | Target | Window | Description |
| :--- | :--- | :--- | :--- |
| **Build Success Rate** | > 99% | 30 Days | Percentage of successful CI builds on `main` and `staged`. |
| **Test Pass Rate** | 100% | Per Commit | No code is merged with failing unit or integration tests. |
| **API Availability** | 99.9% | Monthly | Uptime for the Conxian Gateway (standalone) production endpoints. |
| **Audit Readiness** | 100% | Continuous | All P0 code must have a corresponding audit trail or PR review. |

## 2. Delivery SLOs

| Metric | Target | Description |
| :--- | :--- | :--- |
| **Lead Time to Change** | < 48 Hours | Time from PR approval to deployment on `staged`. |
| **Change Failure Rate** | < 5% | Percentage of deployments to `staged` that require a rollback. |
| **Remediation Time** | < 24 Hours | Time to fix critical hygiene or security regressions. |

## 3. Telemetry Baseline

### 3.1 Metrics (Prometheus/OpenTelemetry)
- **standalone Gateway Health**: `/api/v1/health` (Liveness/Readiness).
- **Latency**: Per-service request latency (Bisq, RGB, Stacks, etc.).
- **Throughput**: Requests per second (RPS) per endpoint.
- **Risk Gauges**: Real-time evaluation of protocol bridge health.

### 3.2 Logging (Structured JSON)
- **Trace ID**: Mandatory for all cross-service requests.
- **Severity**: Debug, Info, Warn, Error, Critical.
- **Audit Logs**: TEE-verified state changes and financial intents.

### 3.3 Alerting
- **Critical**: standalone Gateway unreachable, MuSig2 aggregation failure, BitVM2 fraud proof detected.
- **Warning**: TVL drift, high latency (> 500ms), Hiro API connectivity issues.
