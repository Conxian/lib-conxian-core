# Enhancement Roadmap: Conxian Gateway Alignment

## 1. Overview
The Conxian Gateway provides deep protocol integration and transparent risk metrics aligned with sovereign-first standards.

## 2. Phase 7: Advanced Risk Metrics & Expanded Layer Support (Audit Complete)
- **Achievements:**
  - **Architectural Audit:** Verified multi-factor risk engine and high-precision TVL aggregation.
  - **Precision Alignment:** Refactored TVL tracking from AtomicU64 to f64 RwLock for decimal accuracy.
  - **Risk Logic Enhancement:** Updated `evaluate_risk` to incorporate ZK-bridge and non-custodial exit mechanism weightings.
  - **Protocol Coverage:** Full functional exposure for **BitVM2**, **Core DAO**, **Lorenzo**, **Hemi**, **BOB**, **Merlin**, **Mezo**, **Nubit**, **Bison**, **Zulu**, **Botanix**, **Bitlayer**, **Alpen**, and **Taproot Assets**.
  - **Testing:** Expanded coverage to 59 comprehensive integration tests with 100% success rate.

## 3. Phase 8: Mainnet Node Integration & Direct Bridges
- **Objective:** Transition from simulated monitoring to direct mainnet node RPC integration.
- **Plans:**
  - Real-time RPC connectivity to Bitcoin Core, Stacks, and Core DAO nodes.
  - On-chain reserve verification for Liquid, Rootstock, and BOB bridges.
  - Integration of BitVM challenge-response monitoring in the Gateway Engine.
  - Automated threat detection via mempool and protocol-specific finality tracking.
