# Changelog

All notable changes to the lib-conxian-core project will be documented in this file.


## [0.2.5] - 2026-05-27
### Added
- Restored Unified Theory of Sovereign Enterprise v2.0 (CON-684).
- Established execution-ready metric specifications and data contracts for CR, OC, VX, AS, and NE (CON-682).
- Established formal lib-conxian-core release process (CON-218).

### Changed
- Aligned repository documentation (README, PRD, Architecture) to reflect gateway extraction.
- Refactored workspace configuration and src/gateway.rs to enforce core/gateway boundaries.
- Relocated services/network.ts to the consumer layer (CON-661).


## [0.2.5] - 2026-05-06
### Added
- Established Phase 5/6 governance framework: Created Risk Register (CON-675) and KPI Scorecard (CON-674).
- Linked governance documentation in root README.md.
- Aligned Gateway REST API with documented PRD and TypeScript client requirements.
- Enforced administrative authentication via GATEWAY_ADMIN_API_KEY for sensitive endpoints.
- Implemented missing telemetry and financial intelligence handlers in the Gateway.
- SAB Migration Timeline & Cutover Plan (CON-332): Established four-wave strategy for mainnet activation.
- Flagship Repository Selection (CON-298): Standardized pinned repo ordering and narrative classification.
- Supplier-State SLO (CON-542): Defined service level objectives for security and hygiene remediation.
- Minimalist Reference Wallet Scope (CON-629): Defined technical boundary for the conxius-wallet.
- Routing-Fee Economics Analysis (CON-631): Modeled SDK business sustainability and failure modes.

### Changed
- Synchronized Gateway REST MCP handler with full Phase 9 state proposal tool parity.
- Bumped system-wide version to v0.2.5.

### Fixed
- Resolved compilation failure in `src/musig2.rs` by updating `secp256k1` random generation to v0.31 standards.
- Remediated deprecation warning for `SecretKey::from_slice` in `src/crypto/mod.rs` by migrating to `from_byte_array`.

## [0.2.4] - 2026-05-06
### Added
- Vault SDK Primitive (CON-633): Production-ready hardware-backed signing with policy enforcement and BIP327 MuSig2 key aggregation.
- Internal Audit Reports: Integrated SDK extraction viability (CON-627) and fail-safe logic (CON-625) as crate-private modules.
- New integration tests for Vault SDK policy enforcement and MuSig2 aggregation.

### Changed
- Positioning Rewrite (CON-632): Aligned documentation (README.md, docs/PRD.md, AGENTS.md) with 'Native Bitcoin Apps' strategy.
- Transitioned Gateway and Protocol to supporting infrastructure in canonical maps (CON-636).
- Hardened Gateway integration tests to ensure consistent administrative authentication and service state initialization.

[... previous entries ...]
