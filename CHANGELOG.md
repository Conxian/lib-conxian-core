# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.10] - 2026-07-13

### Added
- **Protocol Hardening**: Hardened protocol primitives and established fuzzing suite
- **CON-700 Compliance**: Architectural boundary enforcement verified via contamination guard
- **FROST Threshold Signatures**: Full implementation of FROST threshold signatures
- **ERC-7683 Support**: Cross-chain order sharing protocol primitives
- **OP_CAT Support**: Bitcoin opcode support for covenant primitives
- **Control Model Expansion**: Enhanced trust tier taxonomy (Strict/Managed/Expedient)
- **Chain Adapter Refinement**: Improved Bitcoin, RGB, Lightning, Stacks adapters

### Changed
- **CI/CD Hardening**: Enhanced security workflows including CodeQL and Cargo Audit
- **Repository Standards**: Improved hygiene baseline and documentation
- **Version Alignment**: MSRV updated to 1.85

### Fixed
- MSRV CodeQL CI failures
- MuSig2 aggregation issues
- Anchor protocol test coverage

### Security
- Zero Secret Egress compliance verified
- Secret scanning enabled at org level
- Dependency vulnerability scanning via Cargo Audit

---

## [v0.2.9] - 2026-05-XX

### Added
- **Vault SDK**: Primary commercial SDK primitive (Hardware-backed Bitcoin signing + policy enforcement)
- **Musig2 Aggregation**: Taproot multi-sig key aggregation
- **Chain Family Taxonomy**: Universal chain support policy

### Changed
- Control model split into lifecycle, ops, and trust modules
- Repository alignment with protocol-first pivot

---

## [v0.2.8] - 2026-04-XX

### Added
- Protocol gap remediation
- Enhanced CI/CD pipeline

---

## [v0.2.7] - 2026-03-XX

### Added
- Shared artifact schemas
- Universal chain expansion

---

## [v0.2.6] - 2026-02-XX

### Added
- Repository standards hardening
- Package metadata alignment

---

## [v0.2.5] - 2026-01-XX

### Added
- Control-plane modules and SDK policy
- Gateway extraction complete
- Vault SDK repositioning

---

## [v0.1.0] - 2026-04-18

Initial stable release with core protocol primitives.
