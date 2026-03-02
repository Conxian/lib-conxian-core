# Product Requirements Document (PRD): Conxian Gateway

## 1. Executive Summary
The Conxian Gateway is the core infrastructure component of the Conxian network, providing a unified, secure, and audit-ready entry point for all sovereign services, Bitcoin/Stacks state logic, affiliate networks, and marketing outreach.

## 2. System Architecture
### 2.1. Unified API
The Gateway exposes a RESTful API under the `/api/v1` prefix. All external service requests are routed through this gateway to ensure consistent authentication, logging, and monitoring.

### 2.2. Core Components
- **API Layer (`gateway/src/api`):** Actix-web based handlers for service routing, health checks, compliance, affiliate management, and metrics.
- **Engine Layer (`gateway/src/engine`):** Core logic for managing service states, request tracking, Bitcoin/Stacks integration, and ecosystem metrics.
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
- **Babylon:** Bitcoin staking protocol (Staking/Security Shared).
- **BOB (Build on Bitcoin):** Hybrid L2 combining Bitcoin security with Ethereum EVM (Optimistic/Rollup).
- **Merlin Chain:** A leading Bitcoin ZK-Rollup focused on the Bitcoin ecosystem (ZK).
- **Botanix:** An EVM-equivalent Layer 2 on Bitcoin using the Spiderchain decentralized primitive (Spiderchain).
- **B² Network:** A ZK-Rollup on Bitcoin that leverages ZK-proofs for secure and scalable transactions (ZK).
- **Citrea:** The first ZK-Rollup on Bitcoin (ZK).
- **Bitlayer:** The first Bitcoin Layer 2 based on BitVM (Optimistic).
- **Alpen:** A ZK Rollup focusing on Bitcoin scalability (ZK).
- **Mezo:** A Bitcoin Economic Layer using tBTC for yield (Economic Layer).
- **Zulu Network:** A multi-layer Bitcoin L2 supporting both EVM and native Bitcoin programmability (Multi-layer).
- **Bison:** A ZK Rollup on Bitcoin (ZK).
- **Hemi Network:** A modular L2 combining Bitcoin security with Ethereum's flexibility (ZK).
- **Taproot Assets:** Protocol for issuing assets on Bitcoin and Lightning (Client-side).
- **Nubit:** A Bitcoin-native Data Availability (DA) layer (DA).
- **Lorenzo:** A Bitcoin staking and restaking protocol (Staking).

## 4. Monitoring, Compliance & Ecosystem
The Gateway provides detailed status information for each service, including:
- **Health Check:** `/api/v1/health` for service availability.
- **System Status:** `/api/v1/status` for real-time system metrics and uptime.
- **Asset Reserves:** `/api/v1/reserves` provides real-time audit data for pegged assets.
- **Service Status:** Detailed metadata per service (e.g., trust model, risk level, DA, settlement, bridge security).
- **Compliance:** `/api/v1/compliance` for KYC/AML and network integrity monitoring.
- **Affiliates:** `/api/v1/affiliates` for managing partnership networks and commissions.
- **Marketing:** `/api/v1/marketing` for tracking campaign status and reach across various channels.
- **Metrics:** `/api/v1/metrics` providing Prometheus-compatible metrics.

## 5. Client Integration
The `services/network.ts` library provides a standard way for client applications to route requests through the Gateway, supporting both local and production environments.

## 6. Security & Auditing
- Unified binary approach simplifies the attack surface.
- Rust-based implementation ensures memory safety and high performance.
- Metadata for trust models and risk levels provided via API for transparency.
