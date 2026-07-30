# Conxian Agent Guidelines: lib-conxian-core

This repository is the canonical home of the **Vault SDK** and shared protocol primitives. It is a "protocol-first" library.

## Strategic Priority (Vault SDK)
Prioritize the development and hardening of the `VaultSDK` primitive (`src/sdk_primitive.rs`). This is the primary commercial interface for the Conxian ecosystem.

## Architectural Boundaries (CON-700)
- **Core (`src/`):** Ownership of canonical types, state machines, invariant validation, and interface contracts.
- **Gateway:** Runtime orchestration, persistence, and external side effects live in the standalone `conxian-gateway` repository.
- **Rule:** If a change needs network IO, database access, or environment-specific branching, it belongs in the Gateway, not here. This is automatically enforced by `scripts/verify_contamination_guard.py`.

## Trust Policy Enforcement (CON-791)
Ensure all cross-domain bridge or messaging metadata aligns with the approved trust-tier taxonomy in `control_model.rs`:
- `Strict` (T1)
- `Managed` (T2)
- `Expedient` (T3)

## Protocol Coverage — SDK → Core Alignment

The Conxius Enclave SDK (`lib-conclave-sdk` v0.2.5) defines the canonical 41-chain `AssetRegistry` and 33 protocol modules. lib-conxian-core is the **shared type system** consumed by both nexus and gateway — it must provide canonical types for every chain and protocol the ecosystem touches.

### Core Type Coverage Requirements

| Domain | Core Types | Consumers |
|--------|-----------|-----------|
| Bitcoin L1 | Block, Tx, Utxo, Script | nexus, gateway |
| Stacks | StacksBlock, ClarityValue | nexus, gateway |
| EVM-compatible | EvmBlock, EvmTx, EvmReceipt | nexus, gateway |
| Cosmos | CosmosBlock, CosmosTx | nexus |
| Lightning | LightningInvoice, LnPayment, LnChannel | nexus, gateway |
| MMR | MmrNode, MmrProof, StateRoot | nexus |
| RGB | RgbContract, RgbConsignment, RgbStash | gateway, nexus |
| BitVM2 | BitVmProof, BitVmGate, BitVmChallenge | gateway, nexus |
| DLC | DlcContract, DlcOracle, DlcOutcome | gateway, nexus |
| Fedimint | FedimintModule, FedimintConsensus | gateway, nexus |
| Canton | UniversalContractRef, CantonDomainRef, CantonStateTranslation | gateway |
| Machine Economy | MachineIdentity, MachineRwaRevenue, M2MSettlement | gateway |
| Settlement | SettlementRail, RailPlan, SettlementReceipt | gateway, platform |
| Identity | DidDocument, DidResolution, IdentityProof | gateway, platform |

### Trust-Tier to Protocol Mapping

| Tier | Protocols | Characteristics |
|------|-----------|----------------|
| T1 Strict | Bitcoin L1 finality, BitVM2 SNARK verification, RGB state proofs, Statechains, Ark, BPoS (Babylon/Core), Merge-mined (Rootstock/Fractal) | Full validation, no trust assumptions |
| T2 Managed | Lightning channel state, Fedimint consensus, Federated sidechains (Liquid/Botanix/Bitlayer/Mezo), Anchor (Stacks) | Consortium/multi-sig governance |
| T3 Expedient | EVM L2 bridges, Cosmos IBC, Solana, Bitcoin rollups (Citrea/Alpen/Alkanes), Alt rollups (BOB/Merlin/Starknet), Alt L1s (BEVM/GOAT) | Optimistic or fast-finality, economic security |

## Universal Chain Coverage (bitcoinlayers.org aligned)

### Current Taxonomy: 17 families, 48 chains

| Family | Chains | bitcoinlayers.org Category |
|--------|--------|---------------------------|
| `BitcoinUtxo` | Bitcoin, Lightning | Bitcoin L1 + Native |
| `Statechain` | Spark, MercuryLayer | Bitcoin Native |
| `Ark` | Second, Arkade | Bitcoin Native |
| `BPoS` | Babylon, Core, Arch, Midl, Nomic, SideProtocol | Sidesystems + Other |
| `Federation` | Liquid, Botanix, Bitlayer, Mezo | Sidesystems + Other |
| `MergeMined` | Rootstock, Fractal | Sidesystems + Other |
| `Anchor` | Stacks | Sidesystems |
| `Rollup` | Citrea, Alpen, Alkanes | Sidesystems + Other |
| `AltRollup` | Bob, Bsquared, Hemi, Corn, Merlin, Rollux, Starknet | Other |
| `AltLayer1` | Bevm, Goat | Other |
| `Csv` | Rgb | Other |
| `Hybrid` | InternetComputer, Flashnet | Other |
| `Evm` | Ethereum, Base, Arbitrum, Optimism, Polygon | Cross-ecosystem |
| `CosmosIbc` | CosmosHub, Osmosis, Celestia | Cross-ecosystem |
| `SolanaSvm` | Solana, Eclipse | Cross-ecosystem |
| `Move` | Aptos, Sui | Cross-ecosystem |
| `Substrate` | Polkadot, Kusama | Cross-ecosystem |

## Workflow Instructions
- **Verification:** Always run `cargo test --workspace` to verify changes.
- **ZSE:** Adhere to Zero Secret Egress standards. Never track environment files or private keys.
- **Source of Truth:** Refer to `bitcoinlayers.org` for the latest Bitcoin Layer 2 research.
- **Protocol Coverage:** When adding a new chain or protocol to the ecosystem, first add canonical types here, then implement adapters in gateway/nexus.
