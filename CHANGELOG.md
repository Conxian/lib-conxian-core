# Changelog

All notable changes to the `lib-conxian-core` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Unified REST API structure at `/api/v1`.
