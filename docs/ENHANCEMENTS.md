# Enhancement Roadmap: Conxian Gateway Alignment

## 1. Overview
The Conxian Gateway is evolving to provide deep protocol integration and transparent risk metrics.

## 2. Phase 7: Advanced Risk Metrics & Expanded Layer Support (Current)
- **Achievements:**
  - Implemented multi-factor risk assessment engine.
  - Added support for **BitVM2** (ZK-Fraud Proofs).
  - Added protocol-specific logic and endpoints for **Core DAO**, **Lorenzo**, **Hemi**, **BOB**, **Merlin**, **Mezo**, **Nubit**, **Bison**, **Zulu**, **Botanix**, **Bitlayer**, **Alpen**, and **Taproot Assets**.
  - Expanded risk model with exit mechanism and operator scores.
  - Refactored Engine to use structured TVL metrics with high precision.
  - Synchronized TypeScript client with new endpoints and helpers.
  - Expanded test coverage to 58 integration tests.

## 3. Phase 8: Mainnet Node Integration & Direct Bridges
- **Objective:** Move from simulated protocol monitoring to direct mainnet node integration.
- **Plans:**
  - Direct RPC connections to Bitcoin, Stacks, and Core DAO nodes.
  - Integration with bridge smart contracts for real-time reserve verification.
  - Automated threat detection based on mempool analysis.
