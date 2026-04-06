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

## 3. Phase 8: Mainnet Node Integration & Direct Bridges (In Progress)
- **Objective:** Transition from simulated monitoring to direct mainnet node RPC integration.
- **Achievements:**
  - **Real-time Stacks Monitoring:** Integrated Hiro Mainnet API for live block height and connectivity tracking.
  - **On-chain Reserve Verification:** Implemented dynamic collateral ratio tracking and "Verified (On-chain)" status for Liquid and Rootstock reserves.
  - **BitVM Challenge Monitoring:** Integrated automated challenge-response status into BitVM2 risk assessments.
  - **External Settlement Proposals:** Implemented TEE-verified proposal-only triggers for ISO 20022, PAPSS, and BRICS with 144-block timelocks (CON-162).
- **Next Steps:**
  - Real-time RPC connectivity to Bitcoin Core and Core DAO nodes.
  - Integration of automated threat detection via mempool analysis.
  - Finality tracking for hybrid L2s (Hemi, BOB).
