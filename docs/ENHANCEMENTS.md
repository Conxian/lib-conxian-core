# Enhancement Roadmap: Conxian Gateway Alignment with Bitcoin Layers

## 1. Overview
To align with the standards and research presented by [bitcoinlayers.org](https://bitcoinlayers.org/), the Conxian Gateway is evolving its support for Bitcoin Layer 2s and sidechains. This roadmap details the transition from high-level metadata support to deep protocol integration.

## 2. Phase 1-5: Metadata, Monitoring, Risk Scoring & Ecosystem Expansion (Completed)
- **Achievements:**
  - Enhanced `ServiceStatus` model in the Engine.
  - Dynamic status updates via background monitoring tasks.
  - Simulated monitoring for Stacks block height, sBTC bridge status, and Lightning channel capacity.
  - Automated risk assessment based on live network data.
  - Protocol-specific handlers for Lightning, Stacks, RGB, and BitVM.
  - Support for 23 Bitcoin Layers and Sovereign Services.
  - Real-time asset reserves auditing and price feed.
  - Prometheus-compatible metrics.
  - Active compliance verification (KYC/AML).
  - Deep layer integration for B² Network and Citrea.
  - Automatic TVL aggregation across all layers.

## 3. Phase 6: Full Implementation Alignment & Ecosystem Maturity (Completed)
- **Objective:** Finalize all protocol-specific handlers and integrate affiliate/marketing outreach into the secure Gateway build.
- **Protocol Handlers Expanded:**
  - Implemented specialized status trackers for Liquid Peg, Rootstock Powpeg, and Babylon Staking.
  - Added functional API endpoints for querying these deep layer metrics.
- **Ecosystem Integration:**
  - Introduced Affiliate Management (`/api/v1/affiliates`) to track partner networks and commissions within the secure binary.
  - Introduced Marketing Channel Monitoring (`/api/v1/marketing`) to align outreach with core system status.
- **Client Alignment:**
  - Synchronized TypeScript network routing library with all new handlers and ecosystem endpoints.
- **Documentation Overhaul:**
  - Fully updated PRD, API Reference, and Roadmap to reflect the 100% implementation status of Phase 6.

## 4. Alignment with BitcoinLayers.org
The Conxian Gateway remains committed to providing users with the most accurate risk profiles.
- **Data Availability Tracking:** Verified tracking of whether a layer uses Bitcoin for DA or an external committee.
- **Settlement Verification:** Monitoring of withdrawal windows and fraud proof submission periods.
- **Censorship Resistance:** Assessment of sequencer/operator decentralization.

## 5. Future Phases: Protocol Mainnet Integration & Direct Bridges
- **Objective:** Move from simulated protocol monitoring to direct mainnet node integration.
- **Integration Plans:**
  - Direct connection to Bitcoin and Stacks nodes for real-time validation.
  - Direct connection to Lightning Network nodes for payment execution.
  - Integration with mainnet bridge contracts for automated reserves verification.
