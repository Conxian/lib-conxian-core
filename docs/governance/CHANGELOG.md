# Changelog

All notable changes to the lib-conxian-core project will be documented in this file.

## [0.2.10] - 2026-06-26
### Added
- Implemented full Babylon Bitcoin Staking Adapter (G-43) using CXIP-21 standard.
- Hardened FROST and OP_CAT protocol primitives with validation logic and expanded templates.
- Established Cross-Lane Readiness Scorecard (CON-1273) and Executive Operating Scorecard (CON-1271).
- Updated CXIP Index and API documentation to reflect latest implementation status.

## [0.2.9] - 2026-06-26
### Added
- Implemented ERC-7683 Solver Selection Algorithm (G-12 / CON-1307).
- Implemented FROST Threshold Signature primitives (G-14 / CON-1302).
- Implemented OP_CAT Recursive Covenant script templates (G-15 / CON-1303).
- Real MuSig2-based Taproot tree aggregation logic in `src/bitvm2.rs` (CON-1306).
- Hardened BIP-322 universal message signing with tagged hash commitments (CON-1266).
- Bridging helpers in `src/musig2.rs` to resolve `secp256k1` and `bitcoin` crate version conflicts.
- Updated Gap Analysis and Implementation Scoring framework.
- Expanded research and implementation paths for ZKCP, BitVMX, and BitVM3 in `docs/UNIVERSAL_SUPPORT_RESEARCH.md` (CON-810).

## [0.2.8] - 2026-06-25
### Added
- Implemented skeletal Universal Chain Adapters for Cosmos, Solana, Move, and Substrate (CXIP-21).
- Added BitVM2 Multi-Party Aggregation stubs and tests (CON-1306).
- Added BIP-322 Universal Message Signing stubs and tests (CON-1266).
- Initialized Babylon Bitcoin Staking adapter (G-43/CON-1312).
- Initialized Fedimint Community Liquidity adapter (G-16/CON-1304).
- Created `docs/GAP_ANALYSIS_AND_SCORING.md` for protocol implementation tracking.
