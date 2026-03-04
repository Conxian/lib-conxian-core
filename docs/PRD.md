# Product Requirements Document (PRD): Conxian Gateway v2

## 1. Executive Summary
The Conxian Gateway is the core infrastructure component of the Conxian network, providing a unified, secure, and audit-ready entry point for all sovereign services, Bitcoin/Stacks state logic, affiliate networks, and marketing outreach. Version 2 introduces granular risk assessment and deeper protocol support.

## 2. System Architecture
### 2.1. Unified API
The Gateway exposes a RESTful API under the `/api/v1` prefix. All external service requests are routed through this gateway to ensure consistent authentication, logging, and monitoring.

### 2.2. Core Components
- **API Layer (`gateway/src/api`):** Actix-web based handlers for service routing, health checks, compliance, affiliate management, risk assessment, and metrics.
- **Engine Layer (`gateway/src/engine`):** Core logic for managing service states, request tracking, Bitcoin/Stacks integration, and ecosystem metrics. Includes a multi-factor risk engine.
- **Infrastructure:** Managed via GCP using modular configurations in `gateway/infrastructure/gcp/`.

## 3. Supported Services
The Gateway supports both sovereign services and Bitcoin Layer 2/sidechain integrations, with metadata aligned with research from **bitcoinlayers.org**:

### 3.1. Sovereign Services
- **Bisq:** Decentralized Bitcoin exchange (P2P).
- **RGB:** Scalable and confidential smart contracts for Bitcoin and Lightning (Client-side).
- **BitVM / BitVM2:** Computing paradigms for Bitcoin programs (Optimistic/ZK-Fraud Proofs).
- **Changelly:** Integration for instant cryptocurrency exchange services (Centralized).

### 3.2. Bitcoin Layers (Aligned with BitcoinLayers.org)
- **Stacks:** Layer for smart contracts and Bitcoin-backed assets (PoX).
- **Lightning Network:** Instant, low-cost Bitcoin payments via state channels.
- **Liquid Network:** Federated sidechain for confidential transactions and issued assets.
- **Rootstock:** EVM-compatible smart contracts secured by merge-mining (Powpeg).
- **Babylon:** Bitcoin staking protocol (Staking/Security Shared).
- **BOB:** Hybrid L2 combining Bitcoin security with Ethereum EVM (Optimistic/Rollup).
- **Merlin Chain:** Bitcoin ZK-Rollup (ZK).
- **Botanix:** EVM-equivalent Layer 2 on Bitcoin using Spiderchain (Spiderchain).
- **B² Network:** ZK-Rollup on Bitcoin (ZK).
- **Citrea:** ZK-Rollup on Bitcoin (ZK).
- **Bitlayer:** Bitcoin Layer 2 based on BitVM (Optimistic).
- **Alpen:** ZK Rollup for Bitcoin scalability (ZK).
- **Mezo:** Bitcoin Economic Layer using tBTC (Economic Layer).
- **Zulu Network:** Multi-layer Bitcoin L2 (Multi-layer).
- **Bison:** ZK Rollup on Bitcoin (ZK).
- **Hemi Network:** Modular L2 combining Bitcoin and Ethereum security (ZK).
- **Taproot Assets:** Protocol for issuing assets on Bitcoin/Lightning (Client-side).
- **Nubit:** Bitcoin-native Data Availability layer (DA).
- **Lorenzo:** Bitcoin staking and restaking protocol (Staking).
- **Core DAO:** Bitcoin-secured EVM-compatible sidechain (Satoshi Plus).

## 4. Monitoring & Transparency
The Gateway provides detailed status information for each service, including:
- **Health Check:** `/api/v1/health` for service availability.
- **System Status:** `/api/v1/status` for real-time system metrics and uptime.
- **Risk Assessment:** `/api/v1/risk-assessment` provides a multi-factor breakdown of security risks (DA, Settlement, Bridge).
- **Compliance:** `/api/v1/compliance` for KYC/AML and network integrity monitoring.

## 5. Security & Auditing
- Unified binary approach simplifies the attack surface.
- Rust-based implementation ensures memory safety and high performance.
- Multi-factor risk engine provides unprecedented transparency for users.
