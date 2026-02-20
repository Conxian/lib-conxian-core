# Enhancement Roadmap: Conxian Gateway Alignment with Bitcoin Layers

## 1. Overview
To align with the standards and research presented by [bitcoinlayers.org](https://bitcoinlayers.org/), the Conxian Gateway will expand its support for Bitcoin Layer 2s and sidechains. This roadmap focuses on improving trust models, data availability, and settlement integration.

## 2. Planned Layer Integrations
### 2.1. Stacks (L2/Sidechain)
- **Objective:** Support Stacks for smart contracts and Bitcoin-backed assets (sBTC).
- **Alignment:** Focus on Proof-of-Transfer (PoX) and the upcoming Nakamoto release for faster settlement.
- **Endpoint:** `/api/v1/stacks`

### 2.2. Lightning Network (L2)
- **Objective:** Enable instant, low-cost Bitcoin payments.
- **Alignment:** Integration with LDK (Lightning Development Kit) or Cln/LND nodes for payment channel management.
- **Endpoint:** `/api/v1/lightning`

### 2.3. Liquid Network (Sidechain)
- **Objective:** Support confidential transactions and issued assets.
- **Alignment:** Federated trust model with Strong Federations.
- **Endpoint:** `/api/v1/liquid`

### 2.4. Rootstock (RSK)
- **Objective:** EVM-compatible smart contracts secured by Bitcoin merge-mining.
- **Alignment:** Powpeg trust model.
- **Endpoint:** `/api/v1/rootstock`

## 3. Technical Enhancements
### 3.1. Unified State Monitoring
- Implementation of a real-time monitor for Bitcoin L2 state transitions.
- Verification of trustless bridges and withdrawal security as defined by BitcoinLayers risk profiles.

### 3.2. Advanced Compliance
- Automated risk scoring based on BitcoinLayers metrics (Liveness, Data Availability, Censorship Resistance).

### 3.3. Performance Optimization
- Transition from mock responses to actual service probing in the Engine layer.
- Implementation of asynchronous state updates for better throughput.

## 4. Alignment with BitcoinLayers.org
The Conxian Gateway aims to provide users with clear information regarding the risk profiles of the layers they interact with.
- **Trust Model Transparency:** API responses now include metadata about the layer's trust model (e.g., "Federated", "Optimistic", "Client-side") and risk levels.
- **Security Metrics:** Integration of liveness and security monitoring for supported layers.
