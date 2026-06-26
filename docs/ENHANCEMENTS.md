# Enhancement Roadmap: lib-conxian-core / Vault SDK Alignment

## 1. Overview
The standalone Conxian Gateway provides deep protocol integration and transparent risk metrics aligned with sovereign-first standards.

## 2. Phase 7: Advanced Risk Metrics & Expanded Layer Support (Audit Complete)
- **Achievements:**
  - **Architectural Audit:** Verified multi-factor risk engine and high-precision TVL aggregation.
  - **Precision Alignment:** Refactored TVL tracking from AtomicU64 to f64 RwLock for decimal accuracy.
  - **Risk Logic Enhancement:** Updated `evaluate_risk` to incorporate ZK-bridge and non-custodial exit mechanism weightings.
  - **Protocol Coverage:** Full functional exposure for **BitVM2**, **Core DAO**, **Lorenzo**, **Hemi**, **BOB**, **Merlin**, **Mezo**, **Nubit**, **Bison**, **Zulu**, **Botanix**, **Bitlayer**, **Alpen**, and **Taproot Assets**.
  - **Testing:** Expanded coverage to 59 comprehensive integration tests with 100% success rate.

## 3. Phase 8: Mainnet Node Integration & Direct Bridges (Complete)
- **Objective:** Transition from simulated monitoring to direct mainnet node RPC integration.
- **Achievements:**
  - **Real-time Stacks Monitoring:** Integrated Hiro Mainnet API for live block height and connectivity tracking.
  - **On-chain Reserve Verification:** Implemented dynamic collateral ratio tracking and "Verified (On-chain)" status for Liquid and Rootstock reserves.
  - **BitVM Challenge Monitoring:** Integrated automated challenge-response status into BitVM2 risk assessments.
  - **MuSig2 Key Aggregation:** Implemented BIP327-compliant deterministic key aggregation for trust-minimized bridging (CON-145).
  - **BitVM2 Segment Orchestration:** Implemented on-chain segment generation (364 chunks) and disprove logic for optimistic bridge safety (CON-464).
  - **External Settlement Proposals:** Implemented TEE-verified proposal-only triggers for ISO 20022, PAPSS, and BRICS with 144-block timelocks (CON-162).
- **Next Steps:**
  - Real-time RPC connectivity to Bitcoin Core and Core DAO nodes.
  - Integration of automated threat detection via mempool analysis.
  - Finality tracking for hybrid L2s (Hemi, BOB).

## 4. Phase 9: Agentic Surface & Autonomous Systems (Complete)
- **Objective:** Enable programmatic trust and autonomous interaction via Model Context Protocol (MCP).
- **Achievements:**
  - **Read-First Agentic Surface:** Implemented an MCP layer allowing agents to audit system state, telemetry, and protocol proofs.
  - **Agent Drafting Flow:** Implemented logic for agent-constructed financial intents, converting complex maneuvers into standard StateProposals for human signing.
  - **Sovereign Handshake:** Implemented full proposal lifecycle (Draft -> Approve -> Execute) with human-in-the-loop validation via MCP tools and REST endpoints.
  - **Industrial Intent Discovery:** Enabled broadcasting of self-describing tool schemas for FSOC validation and settlement triggers.
  - **Real-time Intent Broadcasting:** Implemented background tasks for periodic status updates of pending intents and proposals.
- **Next Steps:**
  - Integration with `lib-conclave-sdk` for hardware-anchored execution of approved intents.
  - Expansion of MCP resources for real-time mempool and bridge auditing.

## 2026-06-26 Hardening Pass (Jules)

- Aligned all documentation and code versioning to **v0.2.10**.
- Verified all core protocol stubs and identified integration paths for **BitVMX**, **BitVM3**, and **ZKCP**.
- Updated Executive and Readiness scorecards to reflect current protocol stability.
- Hardened CI/CD workflow pins and verified hygiene standards.
