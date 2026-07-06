# Changelog

## v0.2.11 (2026-07-06)
- **Silent Payments Hardening:** Replaced transaction scanning simulation with real summation of input public keys and shared secret derivation via ECC point multiplication (sum(P_in) * user_scan_privkey) to align with BIP-352 (G-05).
- **DLC Hardening:** Implemented real oracle attestation verification (s*G = R + e*P) in `src/protocol/dlc.rs`, resolving skeletal stubs (G-06).
- **RGB Expansion:** Introduced `RGBStockAdapter` for production-ready client-side validation, supporting future `rgb-std` Stock persistence (CON-1407).
- **Fedimint Hardening:** Hardened Fedimint community liquidity adapter with real ECC-based blinding (note = H(secret)*G + r*G), replacing XOR-based stubs (G-16).
- **Fuzz Testing:** Established fuzzing infrastructure in the `fuzz/` directory with targets for ERC-7683 intent parsing and MuSig2 public key aggregation (CON-147).
- **Workspace Alignment:** Configured the repository as a Cargo workspace to include the new fuzzing suite.
- **Audit Readiness:** Completed hardening of all core cryptographic paths; all stubs are now resolved or replaced with production-grade logic, unblocking external security audit (CON-1333).

## v0.2.10 (2026-06-26)
- **MuSig2 Hardening:** Implemented BIP-327 signature aggregation logic and Taproot script path integration (G-10).
- **DLC Primitives:** Implemented native Bitcoin finance primitives and mapped them to the USI (G-06).
- **FROST Hardening:** Hardened threshold signature primitives and aggregation logic (G-14).
- **OP_CAT Hardening:** Hardened recursive covenant templates and script construction (G-15).
- **Babylon Staking:** Expanded the Babylon adapter to support fee estimation and universal trait compliance (G-43).
- **ERC-7683 Integration:** Implemented solver selection and bidding algorithm for cross-chain intents (G-12).
- **Universal Adapters:** Hardened skeletal implementations for Bitcoin, EVM, Cosmos, Solana, Move, and Substrate (CXIP-21).
- **BIP-322 Hardening:** Hardened universal message signing logic and verification (G-09).
- **BitVM2 Multi-Party:** Implemented real MuSig2-based Taproot tree aggregation (G-11).
- **ZKCP Scaffolding:** Initialized research and core library requirements for zero-knowledge contingent payments (G-50).
- **Governance Alignment:** Standardized readiness and executive scorecards in `docs/governance/`.

## v0.2.9 (2026-06-21)
- **Nexus zkVM Research:** Expanded verifiable compute primitives for cross-chain state aggregation.
- **RGB Shadow Mode:** Implemented shadow-mode execution for validation gating (CON-768).
- **Vault SDK Scaffolding:** Hardened primary commercial SDK interfaces.
- **CI/CD Alignment:** Hardened organization-wide GitHub Action pins and hygiene controls.

## v0.2.8 (2026-06-15)
- **Stacks sBTC Alignment:** Updated adapter for Nakamoto finality and peg-in/out interfaces.
- **Lightning Resilience:** Initialized audit report tracking for cross-chain event distribution.
- **Metric Specifications:** Established execution-ready formulas for ecosystem variables (, O_C, V_X, A_S, N_E$).

## v0.2.7 (2026-06-11)
- **Event Bus Runtime:** Implemented subscriber delivery layer for cross-chain event distribution.
- **Audit Reports:** Consolidated security and fail-safe audit findings into `docs/architecture/`.

## v0.2.6 (2026-06-08)
- **Organization Hardening:** Aligned repository with public-facing governance and security standards.
- **Cargo.toml Hardening:** Updated metadata and discovery tags for the flagship SDK.
