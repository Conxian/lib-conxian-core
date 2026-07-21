# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.3.0] - 2026-07-21

### Breaking Changes
- This intentional breaking release bumps `lib-conxian-core` from `0.2.12` to
  `0.3.0`. Update callers for typed fail-closed verifier results before
  upgrading.

### Migration
- `UniversalChainAdapter::{verify_state_proof, get_state_root}` now return
  typed `Result<..., StateProofError>` outcomes. Handle
  `MalformedInput`, `VerificationFailed`, `Unsupported { chain }`, and
  `Unavailable { chain }` instead of treating structural input or static roots
  as proof.
- `Bip322Bridge::verify_message_checked` now returns
  `Result<bool, Bip322VerificationError>` after strict address/base64/witness
  parsing; the deprecated `verify_message` wrapper fails closed.
- FROST operations now return `Result<..., FrostError>`. Structurally valid
  inputs still return `FrostError::Unsupported` until an audited backend exists.
- `DlcManager::verify_oracle_attestation_for_intent` returns typed
  `DlcVerificationError` outcomes and deliberately reports
  `UnsupportedIntentBinding` for an otherwise valid tuple whose signature does
  not commit to the complete intent. `verify_execution_checked` reports
  `UnsupportedExecutionContext`, while deprecated `verify_execution` returns
  `false`.
- RGB checked operations retain typed `RGBError` outcomes: active adapters return
  `VerificationUnavailable`, disabled runtime returns `GatedByRolloutMode`, and
  Shadow observations return `NonAuthoritativeShadow` rather than authorizing.

### Security
- **CON-1509 fail-closed verifier remediation:** Removed BIP-322 prefix and
  non-empty-witness fallbacks; adapter, Babylon, and Liquid state-proof paths
  now return typed malformed/failed/unsupported/unavailable failures; RGB
  Shadow mode can no longer authorize; FROST placeholders now return typed
  unsupported; and shallow DLC execution verification is typed unsupported.
- Preserved the real DLC oracle point-equation primitive and the typed
  intent-binding checks, with mutation coverage. Core advertises no BIP-322
  script types until a real audited script/witness verifier exists.
- Hardened `ProtocolVerifier<B>` with an enforceable consumer façade and a
  lower-level `ProtocolVerifierBackend` hook contract. Capability, request,
  result, state-root, provenance, and finality postconditions now run outside
  backend control.
- Every façade success path now rejects returned trust tiers, verification
  classes, and finality classes missing from the stored capability snapshot,
  and requires result provenance to identify the advertised verifier.
- Added canonical chain-family validation, deterministic future/expiry policy,
  request-aware proof-result checks, and versioned/domain-separated structural
  evidence binding.
- Added adversarial coverage proving invalid backend success cannot pass the
  façade. Evidence binding is structural consistency only; it is not
  cryptographic authenticity or production readiness.

### Changed
- This is an intentional pre-publication API break from the old
  consumer-implemented `ProtocolVerifier` trait to the concrete façade/backend
  API. See [docs/MIGRATION.md](docs/MIGRATION.md).
- Raised the package MSRV to Rust `1.91` so the declared support floor covers
  the locked default and optional `enclave` dependency graphs.
- CI now uses Rust `1.91.0` and runs locked default/all-feature checks, tests,
  and all-target Clippy with `-D warnings`.

### Documentation
- Updated verifier architecture, API, ownership boundaries, migration, and
  Phase 1 roadmap documentation without marking unrelated work complete.
- Added the default/enclave Rust compatibility matrix and release-coordination
  guidance for `conxius-enclave-sdk 2.0.11`.

### Added
- **Fuzz Regression Coverage** (#147): Expanded the current suite to five bounded targets with weekly/manual CI. MuSig2 aggregation and PSBT deserialization targets intentionally cover upstream dependencies, while BitVM2 proof verification remains owned by `conxius-enclave-sdk`.

---

## [v0.2.12] - 2026-07-15

### Added
- **BIP-110 Compliance** (#168): Added `Bip110Compliance` struct to `control_model` with validation helpers:
  - `validate_pushdata(size)` - Max 256-byte pushdata
  - `validate_op_return(size)` - Max 83-byte OP_RETURN
  - `validate_script_pubkey(size)` - Max 34-byte ScriptPubKey
  - `validate_witness_element(size)` - Max 256-byte witness
  - `validate_transaction(...)` - Full transaction validation
- `Bip110ValidationResult` and `Bip110Violation` types for compliance reporting
- 9 comprehensive tests for BIP-110 compliance

### Documentation
- Updated `AGENTS.md` with BIP-110 compliance documentation
- Updated `docs/BIP110_ALIGNMENT.md` with completed items

---

## [v0.2.11] - 2026-07-15

### Breaking Changes
- **Vault SDK Removed**: Deprecated modules (VaultSDK, Musig2, BitVM2, Wallet) have been removed
- All Vault SDK functionality now available in `conxius-enclave-sdk` v2.0.11

### Added
- **Silent Payments Hardening**: Replaced transaction scanning simulation with real summation of input public keys and shared secret derivation via ECC point multiplication (sum(P_in) * user_scan_privkey) to align with BIP-352 (G-05).
- **DLC Hardening**: Implemented real oracle attestation verification (s*G = R + e*P) in `src/protocol/dlc.rs`, resolving skeletal stubs (G-06).
- **RGB Expansion**: Introduced `RGBStockAdapter` for production-ready client-side validation, supporting future `rgb-std` Stock persistence (CON-1407).
- **Fedimint Hardening**: Hardened Fedimint community liquidity adapter with real ECC-based blinding (note = H(secret)*G + r*G), replacing XOR-based stubs (G-16).
- **Fuzz Testing**: Established the initial fuzzing infrastructure in the `fuzz/` directory with intent parsing and direct dependency-level MuSig2 public key aggregation coverage (CON-147).

### Changed
- **Workspace Alignment**: Configured the repository as a Cargo workspace to include the new fuzzing suite.
- **Audit Readiness**: Completed hardening of all core cryptographic paths; all stubs are now resolved or replaced with production-grade logic, unblocking external security audit (CON-1333).

### Migration
- See [docs/MIGRATION.md](docs/MIGRATION.md) for migration instructions

---

## [v0.2.10] - 2026-07-13

### Added
- **Protocol Hardening**: Hardened protocol primitives and established fuzzing suite.
- **CON-700 Compliance**: Architectural boundary enforcement verified via contamination guard.
- **FROST Threshold Signatures**: Full implementation of FROST threshold signatures (G-14).
- **ERC-7683 Support**: Cross-chain order sharing protocol primitives and solver selection algorithm (G-12).
- **OP_CAT Support**: Bitcoin opcode support for covenant primitives and recursive templates (G-15).
- **Control Model Expansion**: Enhanced trust tier taxonomy (Strict/Managed/Expedient).
- **Chain Adapter Refinement**: Improved Bitcoin, RGB, Lightning, Stacks adapters.
- **MuSig2 Hardening**: Implemented BIP-327 signature aggregation logic and Taproot script path integration (G-10).
- **DLC Primitives**: Implemented native Bitcoin finance primitives and mapped them to the USI (G-06).
- **Babylon Staking**: Expanded the Babylon adapter to support fee estimation and universal trait compliance (G-43).
- **Universal Adapters**: Hardened skeletal implementations for Bitcoin, EVM, Cosmos, Solana, Move, and Substrate (CXIP-21).
- **BIP-322 Hardening**: Hardened universal message signing logic and verification (G-09).
- **BitVM2 Multi-Party**: Implemented real MuSig2-based Taproot tree aggregation (G-11).
- **ZKCP Scaffolding**: Initialized research and core library requirements for zero-knowledge contingent payments (G-50).

### Changed
- **CI/CD Hardening**: Enhanced security workflows including CodeQL and Cargo Audit.
- **Repository Standards**: Improved hygiene baseline and documentation.
- **Version Alignment**: MSRV updated to 1.85.
- **Governance Alignment**: Standardized readiness and executive scorecards in `docs/governance/`.

### Fixed
- MSRV CodeQL CI failures.
- MuSig2 aggregation issues.
- Anchor protocol test coverage.

### Security
- Zero Secret Egress compliance verified.
- Secret scanning enabled at org level.
- Dependency vulnerability scanning via Cargo Audit.

---

## [v0.2.9] - 2026-06-21

### Added
- **Vault SDK**: Primary commercial SDK primitive (Hardware-backed Bitcoin signing + policy enforcement).
- **Musig2 Aggregation**: Taproot multi-sig key aggregation.
- **Chain Family Taxonomy**: Universal chain support policy.
- **Nexus zkVM Research**: Expanded verifiable compute primitives for cross-chain state aggregation.

### Changed
- **Control Model**: Split into lifecycle, ops, and trust modules.
- **Repository Alignment**: Aligned with protocol-first pivot.
- **RGB Shadow Mode**: Implemented shadow-mode execution for validation gating (CON-768).
- **Vault SDK Scaffolding**: Hardened primary commercial SDK interfaces.
- **CI/CD Alignment**: Hardened organization-wide GitHub Action pins and hygiene controls.

---

## [v0.2.8] - 2026-06-15

### Added
- **Protocol Gap Remediation**: Addressed priority implementation gaps.
- **CI/CD Pipeline**: Enhanced automated verification.
- **Stacks sBTC Alignment**: Updated adapter for Nakamoto finality and peg-in/out interfaces.
- **Lightning Resilience**: Initialized audit report tracking for cross-chain event distribution.
- **Metric Specifications**: Established execution-ready formulas for ecosystem variables ($C_R, O_C, V_X, A_S, N_E$).

---

## [v0.2.7] - 2026-06-11

### Added
- **Shared Artifact Schemas**: Standardized verification and manifest types.
- **Universal Chain Expansion**: Expanded support for non-Bitcoin chains.
- **Event Bus Runtime**: Implemented subscriber delivery layer for cross-chain event distribution.
- **Audit Reports**: Consolidated security and fail-safe audit findings into `docs/architecture/`.

---

## [v0.2.6] - 2026-06-08

### Added
- **Repository Standards Hardening**: Aligned with public-facing governance and security standards.
- **Package Metadata Alignment**: Hardened Cargo.toml metadata and discovery tags for the flagship SDK.

---

## [v0.2.5] - 2026-01-XX

### Added
- Control-plane modules and SDK policy.
- Gateway extraction complete.
- Vault SDK repositioning.

---

## [v0.1.0] - 2026-04-18

Initial stable release with core protocol primitives.
