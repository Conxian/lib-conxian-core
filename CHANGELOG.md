# Changelog

All notable changes to the `lib-conxian-core` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-04-15
### Added
- Canonical BOS Runtime Ownership Map (CON-413).
- Verification of Zero Secret Egress (ZSE) compliance for core protocol paths (CON-188, CON-191).
- Security Boundary Audit report (docs/architecture/SECURITY_BOUNDARY_AUDIT.md).

## [0.2.0] - 2026-04-12
### Added
- Multi-factor risk engine supporting ZK-fraud proofs and non-custodial bridge weightings.
- Protocol-specific functional handlers for 13+ Bitcoin layers.
- Real-time Stacks block height monitoring via Hiro API.
- TEE-verified external settlement proposals (CON-162).
- ZKML verification logic for Guardian attestation (CON-70).
- MuSig2 (BIP327) participant and key structure (Alpha).

### Changed
- Refactored Gateway Engine for Phase 8 mainnet alignment.
- Updated TVL aggregation to high-precision float metrics.
- Hardened repository governance and security guidelines.

## [0.1.0] - 2026-03-01
### Added
- Initial release of Conxian Gateway core logic.
- Basic support for Stacks, Lightning, and Liquid protocols.
- Unified REST API structure at `/api/v1`.
