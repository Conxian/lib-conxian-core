# Changelog

All notable changes to the lib-conxian-core project will be documented in this file.

## [0.2.5] - 2026-06-13
### Added
- Standardized Lightning 'Build-now' lane with `LightningAdapter` trait and `LightningMetrics` model (CON-708).
- Defined core interface for production-grade Lightning backends (SRL-10).
- Added observability data for node health and liquidity (SRL-9).

### Changed
- Decoupled audit reports from crate modules and relocated to `docs/architecture/` for public root decluttering (CON-818).
- Standardized governance files (`CHANGELOG.md`, `SECURITY.md`, etc.) within `docs/governance/` (CON-1186).
- Updated `README.md` and `.github/CODEOWNERS` to align with new file structure.
- Hardened `.gitignore` and verified Zero Secret Egress compliance (CON-1184, CON-1185).

## [0.2.5] - 2026-05-27
- Defined SDK ownership and version policy (CON-1178).
### Added
- Restored Unified Theory of Sovereign Enterprise v2.0 (CON-684).
- Established execution-ready metric specifications and data contracts for CR, OC, VX, AS, and NE (CON-682).
- Established formal lib-conxian-core release process (CON-218).

### Changed
- Aligned repository documentation (README, PRD, Architecture) to reflect gateway extraction.
- Refactored workspace configuration and src/gateway.rs to enforce core/gateway boundaries.
- Relocated services/network.ts to the consumer layer (CON-661).

[... previous entries ...]
