# Universal Chain & Asset Support Research (CON-810)

This document summarizes the research and strategy for Conxian's expansion into universal blockchain and asset support.

## 1. Tier 1 Chain Families (ADR-006)
The following families are prioritized for initial execution adapters:

1. **Bitcoin / UTXO**: Core to Conxian's sovereignty. Includes Native BTC, Stacks (sBTC), Liquid, Babylon, BOB, Mezo, and Citrea.
2. **EVM (Ethereum Virtual Machine)**: Highest TVL and ecosystem breadth. Includes Ethereum L1, Base, Arbitrum, Optimism, Polygon, and Botanix.
3. **Cosmos / IBC**: Best-in-class T1 (Strict) trust model for interchain communication.

## 2. Universal Interoperability Patterns
Research into cross-chain messaging protocols identified core integration patterns:

- **LayerZero V2**: Employs an Endpoint-Peer model with configurable Decentralized Verifier Networks (DVNs). Suitable for T2 (Managed) trust tier.
- **Axelar**: Uses an Amplifier/Verifier model for verifiable message routing. Aligns with T3 (Expedient) trust tier.
- **Circle CCTP**: Native burn-and-mint institutional liquidity for USDC across supported chains.

## 3. Verifiable Compute & State
- **Nexus Network**: Provides a decentralized proving system for zkVM programs. Enables verifiable off-chain maneuver orchestration and universal system state tracking.

## 4. Trust Tier Mapping
| Trust Tier | Mechanism | Example Systems |
| :--- | :--- | :--- |
| **T1: Strict** | Light-Client / IBC | Cosmos IBC, sBTC |
| **T2: Managed** | Hybrid Proof + Attesters | LayerZero V2, TEE-attested sidechains |
| **T3: Expedient** | External Quorum / Intents | Axelar, Hyperlane, ERC-7683 Solvers |

## 5. Asset Registry Expansion
The `AssetRegistry` (managed in standalone Gateway) is expanded to support regional stablecoins and major global assets, positioning Conxian as the universal payment infrastructure.

- **Africa**: ZARP, NGNC, cKES.
- **Latin America**: BRLA, cREAL.
- **Asia-Pacific**: JPYC, XSGD, KRW.
- **Europe**: EURC, GBPT.

## 6. Implementation Guardrails
- **Fail-Closed**: All cross-chain operations must fail closed on missing or invalid trust metadata.
- **Non-Custodial**: Conxian does not take possession of customer funds. All signing occurs in sovereign hardware via the Vault SDK.

## 7. Nexus zkVM Integration Research
The Nexus zkVM is a modular, extensible, and performant zkVM. It allows us to:
- **Verifiable Maneuvers**: Execute complex financial logic off-chain and provide a zero-knowledge proof of correct execution to the on-chain vault.
- **State Aggregation**: Prove the state of multiple chains simultaneously, enabling atomic-like cross-chain operations without a trusted intermediary.
- **Proof Aggregation**: Use Nexus to aggregate proofs from different L2s (BitVM2, Stacks, EVM Rollups) into a single succinct proof for the mainnet vault.

## 8. Advanced Protocol Research: BitVMX & BitVM3

### BitVMX: High-Efficiency Adaptive Proofs (G-44)
BitVMX represents a significant optimization over BitVM2 by introducing **Adaptive Proofs**.
- **Mechanism**: Introduces a bisection game over the execution trace, reducing the data required for on-chain resolution.
- **Implementation Path**: Requires `src/protocol/bitvmx.rs` to manage the challenge-response state machine and sub-segment proof verification.
- **Candidate Score**: Strategic 40, Readiness 15, Demand 30 (Total: 85) — Active research target for v0.3.x floor.

### BitVM3: Optimized Settlement Floor (G-20)
BitVM3 targets the theoretical limit of Bitcoin-native optimistic settlement.
- **ZKP-Enabled**: Explores using SNARKs/STARKs directly within the challenge tree to collapse verification steps.
- **Recursive Finality**: Targets < 1-hour settlement finality for high-value vaults.
- **Candidate Score**: Strategic 40, Readiness 10, Demand 30 (Total: 80) — Long-term directional research.

## 9. Zero-Knowledge Contingent Payments (ZKCP) (G-50)
ZKCP allows for the atomic exchange of a secret (e.g., a digital good) for a payment, without either party trusting the other.
- **Requirement**: Core library must support SHA256-preimage verification scripts and homomorphic commitment schemes.
- **Status**: Scaffolding exists in `src/control_model.rs` via the `ZkVerified` class. Core library provides fail-closed contract interfaces; full cryptographic execution remains downstream.
- **Candidate Score**: Strategic 35, Readiness 15, Demand 20 (Total: 70) — Researching.

## 10. Research Update (2026-06-28): v2.0.4 Hardening Findings
- **FROST Round 2**: Identified requirement for encrypted share distribution to prevent MITM attacks during key generation.
- **X.509 DER**: Verified that enclave certificate chains require full ASN.1 SEQUENCE parsing to enforce hardware attestation boundaries.

## 11. Research Update (2026-08-19): v0.3.2 Ecosystem Audit & Gap Alignment
- **RGB Integration Boundary**: Hardened `RGBStockAdapter` in `src/rgb/mod.rs` to ensure contract ID lookups fail-closed with `RGBError::InvalidContractId` on empty or whitespace strings.
- **DLC CET & Oracle Verification**: Core equations and intent validation are enforced; CET construction and oracle attestation verification remain downstream in `conxian-gateway`.
- **SDK Boundary**: Confirmed `conxius-enclave-sdk` v2.0.16 as the canonical signing and attestation layer, maintaining `lib-conxian-core` as a Zero Secret Egress protocol primitives provider.
