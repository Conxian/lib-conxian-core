# Changelog

All notable changes to the lib-conxian-core project will be documented in this file.

## [0.2.5] - 2026-05-06
### Added
- SAB Migration Timeline & Cutover Plan (CON-332): Established four-wave strategy for mainnet activation.
- Flagship Repository Selection (CON-298): Standardized pinned repo ordering and narrative classification.
- Supplier-State SLO (CON-542): Defined service level objectives for security and hygiene remediation.
- Minimalist Reference Wallet Scope (CON-629): Defined technical boundary for the conxius-wallet.
- Routing-Fee Economics Analysis (CON-631): Modeled SDK business sustainability and failure modes.

### Changed
- Synchronized Gateway REST MCP handler with full Phase 9 state proposal tool parity.
- Bumped system-wide version to v0.2.5.

## [0.2.4] - 2026-05-06
### Added
- Vault SDK Primitive (CON-633): Production-ready hardware-backed signing with policy enforcement and BIP327 MuSig2 key aggregation.
- Internal Audit Reports: Integrated SDK extraction viability (CON-627) and fail-safe logic (CON-625) as crate-private modules.
- New integration tests for Vault SDK policy enforcement and MuSig2 aggregation.

### Changed
- Positioning Rewrite (CON-632): Aligned documentation (README.md, docs/PRD.md, AGENTS.md) with 'Native Bitcoin Apps' strategy.
- Transitioned Gateway and Protocol to supporting infrastructure in canonical maps (CON-636).
- Hardened Gateway integration tests to ensure consistent administrative authentication and service state initialization.

## [0.2.3] - 2026-04-30
### Added
- Real-time RPC connectivity for Bitcoin Core and Core DAO nodes in Gateway Engine.
- Mempool congestion analysis and automated threat detection for Bitcoin.
- Cross-layer block finality tracking for Hemi and BOB hybrid L2s.
- Full state proposal lifecycle (Approve/Execute) tools in MCP layer.
- REST endpoints for proposal approval and execution.
- Real-time intent broadcasting background task for Phase 9.

### Changed
- Transitioned Phase 9 to "Complete" in system documentation.
- Expanded Gateway integration tests to cover new RPC and proposal logic.

[... previous entries ...]

## [0.2.2] - 2026-04-24
### Added
- BitVM2 segment orchestration logic (364 chunks) in lib-conxian-core.
- /api/v1/bitvm2/segments/{state_root} endpoint in Conxian Gateway.
- getBitvm2Segments function in TypeScript network service.
- Unit tests for BitVM2 orchestration and Gateway segment API.

### Changed
- Updated system documentation (PRD, API, ENHANCEMENTS) to v0.2.2.
- Transitioned Phase 8 to "Complete" status.
- Hardened Gateway Engine risk assessment and status reporting logic.

## [0.2.1] - 2026-04-18
### Added
- Integrated real-time Stacks monitoring via Hiro Mainnet API.
- Implemented TEE-verified external settlement proposals (ISO 20022, PAPSS, BRICS) with 144-block timelocks (CON-162).
- Added BIP327-compliant MuSig2 key aggregation logic (CON-145).
- Expanded support for Core DAO, Lorenzo, Hemi, and BitVM2.
- Implemented BitVM2 segment orchestration and disprove logic (CON-464).
- Added support for controlled partner intake v1 flow.

### Changed
- Refactored Gateway Engine for Phase 8 mainnet alignment and modularity.
- Updated TVL aggregation to high-precision float metrics (f64).
- Hardened environment separation (CON-488) and remediation logic.
- Consolidated repository ownership in CODEOWNERS.

### Fixed
- Improved BNS and ENS identity resolution accuracy.
- Standardized security reporting and governance documentation.

## [0.2.0] - 2026-04-12
### Added
- Multi-factor risk engine supporting ZK-fraud proofs and non-custodial bridge weightings.
- Protocol-specific functional handlers for 13+ Bitcoin layers.
- Real-time Stacks block height monitoring via Hiro API.
- ZKML verification logic for Guardian attestation (CON-70).

### Changed
- Refactored Gateway Engine for Phase 8 alignment.
- Updated repository governance and security guidelines.

## [0.1.0] - 2026-03-01
### Added
- Initial release of Conxian Gateway core logic.
- Basic support for Stacks, Lightning, and Liquid protocols.
- Unified REST API structure at /api/v1.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
