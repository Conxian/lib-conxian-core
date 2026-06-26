## [0.2.9] - 2026-06-26
### Added
- Real MuSig2-based Taproot tree aggregation logic in `src/bitvm2.rs` (CON-1306).
- Hardened BIP-322 universal message signing with tagged hash commitments (CON-1266).
- Bridging helpers in `src/musig2.rs` to resolve `secp256k1` and `bitcoin` crate version conflicts.
- Updated Gap Analysis and Implementation Scoring framework.
# Changelog

All notable changes to the lib-conxian-core project will be documented in this file.

## [0.2.8] - 2026-06-25
### Added
- Implemented skeletal Universal Chain Adapters for Cosmos, Solana, Move, and Substrate (CXIP-21).
- Added BitVM2 Multi-Party Aggregation stubs and tests (CON-1306).
- Added BIP-322 Universal Message Signing stubs and tests (CON-1266).
- Initialized Babylon Bitcoin Staking adapter (G-43/CON-1312).
- Initialized Fedimint Community Liquidity adapter (G-16/CON-1304).
- Created `docs/GAP_ANALYSIS_AND_SCORING.md` for protocol implementation tracking.

### Changed
- Expanded `docs/UNIVERSAL_SUPPORT_RESEARCH.md` with BitVMX and BitVM3 research.
- Hardened CI/CD workflows by pinning `actions/checkout` to v4.1.7 (SHA).

## [0.2.7] - 2026-06-19
### Added
- Implemented shared schemas for `DeploymentManifest` and `VerificationResult` in `src/deployment.rs` (CON-1237).
- Expanded `control_model.rs` with additional chain support: CosmosHub, Osmosis, Celestia, Solana, and Eclipse (CON-810).
- Added `BridgeSystem::Bitvm2` variant to support optimistic bridge logic.
- Comprehensive unit tests for new shared schemas and universal chain support.

### Changed
- Updated `docs/API.md` to reflect v0.2.7 updates and new schema definitions.
- Hardened GitHub Action workflows with pinned SHAs and verified repo hygiene.

[... previous entries ...]
