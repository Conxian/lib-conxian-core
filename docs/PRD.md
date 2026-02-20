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
The Gateway supports both sovereign services and Bitcoin Layer 2/sidechain integrations, with metadata aligned with research from **bitcoinlayers.org**:

### 3.1. Sovereign Services
- **Bisq:** Decentralized Bitcoin exchange (P2P).
- **RGB:** Scalable and confidential smart contracts for Bitcoin and Lightning (Client-side).
- **BitVM:** A computing paradigm to express any program as a Bitcoin script (Optimistic).
- **Changelly:** Integration for instant cryptocurrency exchange services (Centralized).

### 3.2. Bitcoin Layers (Aligned with BitcoinLayers.org)
- **Stacks:** Layer for smart contracts and Bitcoin-backed assets (PoX).
- **Lightning Network:** Instant, low-cost Bitcoin payments via state channels.
- **Liquid Network:** Federated sidechain for confidential transactions and issued assets.
- **Rootstock:** EVM-compatible smart contracts secured by merge-mining (Powpeg).

## 4. Monitoring & Compliance
The Gateway provides detailed status information for each service, including:
- **Health Check:** `/api/v1/health` for service availability.
- **System Status:** `/api/v1/status` for real-time system metrics and uptime.
- **Service Status:** Detailed metadata per service (e.g., `/api/v1/stacks`), including:
    - **Trust Model:** Categorization based on bitcoinlayers.org (e.g., Federated, State Channels, PoX).
    - **Risk Level:** Qualitative risk assessment.
    - **Data Availability:** On-chain, Off-chain, or Federated.
    - **Settlement:** The layer where finality is achieved (typically Bitcoin).
    - **Bridge Security:** Mechanism for moving assets (e.g., sBTC, Powpeg, Strong Federation).
- **Compliance:** `/api/v1/compliance` for KYC/AML and network integrity monitoring.
- **Metrics:** `/api/v1/metrics` providing Prometheus-compatible metrics (uptime, request counts).

## 5. Client Integration
The `services/network.ts` library provides a standard way for client applications to route requests through the Gateway, supporting both local and production environments.

## 6. Security & Auditing
- Unified binary approach simplifies the attack surface.
- Rust-based implementation ensures memory safety and high performance.
- Metadata for trust models and risk levels provided via API for transparency, enabling users to make informed decisions based on the risk profiles documented by the Bitcoin community.
