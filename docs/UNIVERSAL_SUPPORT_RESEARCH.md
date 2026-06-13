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
