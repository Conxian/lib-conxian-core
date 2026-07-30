# Conxian Agent Guidelines: lib-conxian-core

This repository is the canonical home of **protocol primitives** and shared type contracts for the Conxian ecosystem. It is a "protocol-first" library — types, invariants, fail-closed boundaries. No network IO, no persistence, no hardware.

## Architecture (v0.3.0)
- **Core (`src/`):** Canonical types, state machines, invariant validation, interface contracts. 48 chains, 17 families, 15 adapters (all fail-closed).
- **SDK (`conxius-enclave-sdk` v2.0.11):** Hardware signing, attestation, FROST DKG, MuSig2, BitVM2. Core optionally depends on SDK (`enclave` feature) for type re-exports.
- **Gateway (`conxian-gateway`):** Runtime orchestration, persistence, external side effects. Has `core_compat` bridge to Core types.
- **Nexus (`conxian-nexus`):** Chain observation, proof verification, headers, finality evidence. Has `core_types` re-exports from Core.
- **Rule:** Network IO, database access, or environment-specific branching → Gateway/Nexus, not Core. Enforced by `scripts/verify_contamination_guard.py`.

## Fail-Closed Posture
Every verification boundary returns typed errors, never fabricated success. 65 fail-closed return sites across production code. Deprecated boolean wrappers return `false`. See `docs/VERIFIER_INVENTORY.md` for the complete inventory.

## Trust Tier Taxonomy (CON-791)
- `Strict` (T1): Full validation, no trust assumptions
- `Managed` (T2): Consortium/multi-sig governance
- `Expedient` (T3): Economic security, fast-finality
- `ObserverOnly` (T4): Not allowed in production

## Universal Chain Coverage (bitcoinlayers.org aligned, v0.3.0)

### 17 families, 48 chains

| Family | Chains |
|--------|--------|
| `BitcoinUtxo` | Bitcoin, Lightning |
| `Statechain` | Spark, MercuryLayer |
| `Ark` | Second, Arkade |
| `BPoS` | Babylon, Core, Arch, Midl, Nomic, SideProtocol |
| `Federation` | Liquid, Botanix, Bitlayer, Mezo |
| `MergeMined` | Rootstock, Fractal |
| `Anchor` | Stacks |
| `Rollup` | Citrea, Alpen, Alkanes |
| `AltRollup` | Bob, Bsquared, Hemi, Corn, Merlin, Rollux, Starknet |
| `AltLayer1` | Bevm, Goat |
| `Csv` | Rgb |
| `Hybrid` | InternetComputer, Flashnet |
| `Evm` | Ethereum, Base, Arbitrum, Optimism, Polygon |
| `CosmosIbc` | CosmosHub, Osmosis, Celestia |
| `SolanaSvm` | Solana, Eclipse |
| `Move` | Aptos, Sui |
| `Substrate` | Polkadot, Kusama |

## Key Files
- Chain taxonomy: `src/control_model/trust.rs` (Chain, ChainFamily, BridgeSystem, TrustTier)
- Risk profiles: `data/risk_profiles/v1.json` (65 entries, all not_assessed)
- Verifier contracts: `src/verifier.rs` (ProtocolVerifier façade)
- Signing contracts: `src/signing.rs` (UniversalChainSigner, 21 AddressFormats)
- Adapters: `src/adapters/mod.rs` (15 UniversalChainAdapter impls)
- Verifier inventory: `docs/VERIFIER_INVENTORY.md`

## Cross-Repo State (2026-07-30)

| Repo | PR | Tag | crates.io |
|------|-----|-----|-----------|
| lib-conxian-core | [#236](https://github.com/Conxian/lib-conxian-core/pull/236) | `v0.3.0` | Pending |
| conxius-enclave-sdk | [#257](https://github.com/Conxian/conxius-enclave-sdk/pull/257) | `v2.0.12` | Pending |
| conxian-gateway | [#304](https://github.com/Conxian/conxian-gateway/pull/304) | `v0.1.5` | Pending |
| conxian-nexus | [#189](https://github.com/Conxian/conxian-nexus/pull/189) | `v0.4.22` | Existing |

## Remaining: crates.io Publishing
```bash
# Core — rerun release-only recovery (cargo publish succeeded, verification timed out)
gh workflow run crates-publish.yml -R Conxian/lib-conxian-core \
  -f mode=release-only -f release_tag=v0.3.0

# SDK — fix rebuild verification in release-strict.yml line 414-430, then publish
# (cargo package --locked non-determinism between SBOM and publish jobs)

# Gateway — dispatch crates.io publish
gh workflow run release.yml -R Conxian/conxian-gateway \
  -f release_version=0.1.5 -f publish_to_crates_io=true

# After SDK on crates.io: bump Core pin
# conxius-enclave-sdk = { version = "=2.0.12" }
```

## Workflow
- `cargo fmt --check && cargo clippy --all-features -- -D warnings && cargo test --workspace`
- Source of truth: `bitcoinlayers.org` for Bitcoin L2 taxonomy
- New chain/protocol → add canonical types here first → adapters in gateway/nexus
- Never track environment files, private keys, or credentials
- Use `41502979+botshelomokoka@users.noreply.github.com` for commits to bypass email privacy
