# Product Requirements Document (PRD): Conxian Gateway

## 1. Executive Summary
The Conxian Gateway is the core infrastructure component of the Conxian network, providing a unified, secure, and audit-ready entry point for all sovereign services and Bitcoin/Stacks state logic. It replaces legacy architectures (Anya-core, OPSource) with a high-performance Rust-based implementation.

## 2. System Architecture
### 2.1. Unified API
The Gateway exposes a RESTful API under the `/api/v1` prefix. All external service requests are routed through this gateway to ensure consistent authentication, logging, and monitoring.

### 2.2. Core Components
- **API Layer (`gateway/src/api`):** Actix-web based handlers for service routing, health checks, and metrics.
- **Engine Layer (`gateway/src/engine`):** Core logic for managing service states, request tracking, and Bitcoin/Stacks integration.
- **Infrastructure:** Managed via GCP using modular configurations in `gateway/infrastructure/gcp/`.

## 3. Supported Services
Currently, the Gateway supports the following sovereign services:
- **Bisq:** Decentralized Bitcoin exchange.
- **RGB:** Scalable and confidential smart contracts for Bitcoin and Lightning.
- **BitVM:** A computing paradigm to express any program as a Bitcoin script.
- **Changelly:** Integration for instant cryptocurrency exchange services.

## 4. Monitoring & Compliance
- **Health Check:** `/api/v1/health` for service availability.
- **System Status:** `/api/v1/status` for real-time system metrics and uptime.
- **Compliance:** `/api/v1/compliance` for KYC/AML and network integrity monitoring.
- **Metrics:** `/api/v1/metrics` providing Prometheus-compatible metrics (uptime, request counts).

## 5. Client Integration
The `services/network.ts` library provides a standard way for client applications to route requests through the Gateway, supporting both local and production environments.

## 6. Security & Auditing
- Unified binary approach simplifies the attack surface.
- Rust-based implementation ensures memory safety and high performance.
- Centralized logging and metrics for comprehensive auditing.
