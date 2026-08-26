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

## 12. Research Update (2026-08-19 Session Synthesis): Multi-Cloud & Neon DB Infrastructure Mapping
An exhaustive audit of the Conxian Labs organization cloud infrastructure (`org-silent-sun-00457600`) confirms six dedicated Neon PostgreSQL project environments supporting the microservice architecture:

| Project Name | Neon Project ID | Region | PG Version | Purpose / Architectural Layer |
| :--- | :--- | :--- | :--- | :--- |
| `corelibs` | `sparkling-sunset-69236559` | `aws-us-east-2` | 18 | `lib-conxian-core` persistent state models & protocol verification schemas |
| `Software dev kit` | `weathered-night-98492579` | `aws-us-east-2` | 18 | `conxius-enclave-sdk` Vault SDK state, DKG sessions, and attestation logs |
| `Business Operating System` | `noisy-flower-17484435` | `aws-us-east-2` | 18 | BOS enterprise risk control plane, policy enforcement & billing |
| `market` | `small-math-44741750` | `aws-eu-central-1` | 18 | Cross-chain intent orderbooks, solver liquidity & market routing |
| `Gateway` | `noisy-cloud-41146057` | `aws-ap-southeast-1` | 18 | `conxian-gateway` runtime state, rate limiting, and client sessions |
| `Conxian Nexus` | `orange-paper-76209725` | `aws-eu-central-1` | 17 | Nexus zkVM proof aggregation, state roots, and logical replication |

### Core Infrastructure & Security Directives
1. **Zero Secret Egress (ZSE)**: Database persistence records only state commitments, proof roots, and public keys. Private keys and ephemeral signing material are restricted strictly to enclave RAM and `conxius-enclave-sdk`.
2. **Fail-Closed Relational Verification**: All foreign key constraints and index bounds across the 6 database environments mirror the Rust `lib-conxian-core` fail-closed invariant types (`Bip110Compliance`, `DlcVerificationError`, `ProofVerificationRequest`).


## 13. Research Update (2026-08-26): DLC CET Structure Validation Hardening
- **CET Construction Verification**: Hardened `DlcManager::validate_cet_structure` in `src/protocol/dlc.rs` to validate payout distributions against intent collateral satoshis.
- **Invariant Enforcement**: Enforces non-empty recipient scripts, non-zero payout amounts, checked addition overflow guards, and total payout bounds (`total_payout <= collateral_sats`).
- **Cloud Infrastructure Alignment**: Verified fail-closed relational integrity across Neon PostgreSQL database `corelibs` (`sparkling-sunset-69236559`) and Supabase BOS project `yauldfcpswnufgwfvnlr`.

## 14. Research Update (2026-08-26 Session 58 Synthesis): ERC-7683 Intent Mapping & Transport Neutrality
- **ERC-7683 Bidirectional Serialization**: Confirmed `Erc7683CrossChainOrder` in `src/chain/erc7683.rs` enables seamless conversion between Conxian `CrossChainIntent` structs and standard ERC-7683 EVM solver payloads while strictly validating open/fill deadlines.
- **Transport Capability Isolation**: Re-verified `TransportAdapter` and `TransportCapability` in `src/chain/transport.rs` enforce transport neutrality by abstracting UTXO queries, transaction broadcasts, and fee estimation away from core protocol types, ensuring strict compliance with CON-700 architectural boundary rules.
- **Ecosystem Topology Synchronization**: Confirmed state root schema compatibility across all 6 Neon database projects (`sparkling-sunset-69236559`, `weathered-night-98492579`, `noisy-flower-17484435`, `small-math-44741750`, `noisy-cloud-41146057`, `orange-paper-76209725`) and Supabase projects (`yauldfcpswnufgwfvnlr`, `iczqutrbbfudfzfplymc`).
