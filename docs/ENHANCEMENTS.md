# Enhancement Roadmap: Conxian Gateway Alignment with Bitcoin Layers

## 1. Overview
To align with the standards and research presented by [bitcoinlayers.org](https://bitcoinlayers.org/), the Conxian Gateway is evolving its support for Bitcoin Layer 2s and sidechains. This roadmap details the transition from high-level metadata support to deep protocol integration.

## 2. Phase 1: Metadata & Transparency (Completed)
- **Objective:** Provide clear risk and trust model transparency to users.
- **Achievements:**
  - Implementation of enhanced `ServiceStatus` model in the Engine.
  - Integration of Data Availability, Settlement, and Bridge Security metadata aligned with bitcoinlayers.org.
  - Unified API endpoints for all supported layers.

## 3. Phase 2: Active Monitoring (Completed)
- **Objective:** Move from static responses to real-time service probing.
- **Achievements:**
  - Refactored Engine to support dynamic status updates via background monitoring tasks.
  - Implemented simulated polling for Stacks block height and sBTC bridge status.
  - Implemented simulated monitoring for Lightning channel capacity and node connectivity.
  - Added protocol-specific metadata fields to API responses.

## 4. Phase 3: Advanced Risk Scoring & Compliance (Completed)
- **Objective:** Automate risk assessment based on live network data.
- **Dynamic Risk Scoring:** Adjust `risk_level` based on real-time metrics (e.g., federation member count, channel liquidity).
- **Liveness Monitoring:** Real-time alerts if a layer's settlement on Bitcoin is delayed beyond standard thresholds.
- **Compliance Expansion:** Integrate AML/KYC checks specifically for bridge entries and exits.

## 5. Phase 4: Protocol-Specific Handlers (Completed)
- **Objective:** Provide deeper functionality beyond status monitoring.
- **Lightning Payments:** Direct payment routing and invoice generation through the Gateway.
- **Stacks Smart Contracts:** Proxy for interacting with Clarity contracts.
- **RGB/BitVM Support:** Infrastructure for hosting and verifying client-side state or fraud proofs (Implemented basic state/proof handlers).
- **New Layer Integration:** Added support for Babylon, BOB, Merlin, Botanix, B² Network, Citrea, and Bitlayer.
- **Unified Price Feed:** Integrated a simulated real-time price feed for core assets (BTC, STX).

## 6. Alignment with BitcoinLayers.org
The Conxian Gateway remains committed to providing users with the most accurate risk profiles.
- **Data Availability Tracking:** Verified tracking of whether a layer uses Bitcoin for DA or an external committee.
- **Settlement Verification:** Monitoring of withdrawal windows and fraud proof submission periods.
- **Censorship Resistance:** Assessment of sequencer/operator decentralization.

## 7. Phase 5: Ecosystem Expansion & Asset Protocols (In Progress)
- **Objective:** Broaden support for new Bitcoin L2s and emerging asset protocols.
- **New Layer Integration:** Added support for Alpen, Mezo, Zulu Network, Bison, and Hemi Network.
- **Asset Protocols:** Integrated monitoring for Taproot Assets (formerly Taro).
- **Economic Layer Tracking:** Implemented specialized tracking for "Economic Layers" like Mezo.
- **Multi-layer Support:** Added infrastructure for multi-layer L2 architectures like Zulu.
