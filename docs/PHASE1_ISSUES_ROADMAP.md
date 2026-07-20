# PHASE 1: Issue Creation & Discovery - Conxian Ecosystem

**Status**: 📋 Planning Phase  
**Date**: 2026-07-20  
**Scope**: Core protocol library alignment & foundation

## Overview

This document tracks Phase 1 issues to be created across the Conxian ecosystem. Phase 1 focuses on **discovery, alignment, and gap identification** without implementation work. Issues created here will inform Phase 2 (implementation) and Phase 3+ (research/expansion).

### Principle
- **Create, don't implement** in Phase 1
- Gather requirements via issues
- Document dependencies
- Establish acceptance criteria
- Enable parallel planning

---

## LIB-CONXIAN-CORE Issues

### Category: Signing & UCS (Universal Chain Signing)

#### [CORE-001] Define Universal Chain Signing (UCS) Interface Spec
**Type**: Architecture  
**Priority**: 🔴 CRITICAL  
**Depends On**: None  
**GitHub Issue**: [#174](https://github.com/Conxian/lib-conxian-core/issues/174)

**Implementation Note**: The implementation PR delivers the platform-neutral
`UniversalChainSigner` contract, explicit signing/verification/address models,
versioned capability discovery, secret-safe fail-closed errors, deterministic
mock contract tests, and signing ownership documentation. Hardware-backed
signing, key custody, network I/O, and runtime/provider orchestration remain
outside core.

**Description**:
Define shared trait interface for signing operations across all chains. This is the foundation for SDK and Gateway adapter coordination.

**Acceptance Criteria**:
- [ ] Trait definition: `UniversalChainSigner` with required methods
- [ ] Support for all chain families: Bitcoin, Stacks, Ethereum, Solana, Babylon, Liquid
- [ ] Method signatures for:
  - `sign_message(&self, chain: ChainId, msg: &[u8]) -> Result<Signature>`
  - `verify_signature(&self, chain: ChainId, sig: &Signature) -> Result<bool>`
  - `get_address(&self, chain: ChainId, path: DerivationPath) -> Result<Address>`
- [ ] Error taxonomy mapping per chain
- [ ] Documentation with 3+ chain examples

**Related Code**:
- `src/stacks/mod.rs` (trait `StacksAdapter`)
- `src/control_model/trust.rs` (RailCustodyModel::signer_architecture)

---

#### [CORE-002] BIP-341/342 Miniscript Support Validation
**Type**: Enhancement  
**Priority**: 🔴 CRITICAL  
**Depends On**: [CORE-001]  

**Description**:
Verify that core protocol handles BIP-341 (Taproot) and BIP-342 (Tapscript) signing flows correctly. Document any gaps or unsupported operations.

**Acceptance Criteria**:
- [ ] Audit existing `TaprootManager` in SDK against spec
- [ ] Document supported script types:
  - [ ] P2TR (pay-to-Taproot)
  - [ ] Tapscript leaf signing
  - [ ] Key path spending vs script path
- [ ] Define Miniscript constraint language support
- [ ] Test matrix for script combinations

**Related Code**:
- SDK: `src/protocol/bitcoin.rs` (TaprootManager)
- Core: `docs/BIP110_ALIGNMENT.md`

---

#### [CORE-003] Document Protocol Signing Flows Per Chain Family
**Type**: Documentation  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-001]  

**Description**:
Create comprehensive guide showing how each chain family (Bitcoin, Stacks, Babylon, Liquid, RGB, DLC) integrates with UCS.

**Acceptance Criteria**:
- [ ] Bitcoin: BIP-341/342 + BIP-110 compliance
- [ ] Stacks: sBTC peg signing coordination
- [ ] Babylon: BTC header chain + EOTS verification
- [ ] Liquid: Peg-in/peg-out signing
- [ ] RGB: State transition signing
- [ ] DLC: CET construction + oracle signatures

**Format**: Chain-specific docs in `docs/signing/<CHAIN>.md`

---

### Category: BIP-110 & Data Compliance

#### [CORE-004] BIP-110 Compliance Matrix Documentation
**Type**: Research  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-001]  

**Description**:
Document impact of BIP-110 (Reduced Data Softfork) on all protocol operations. Identify transaction composition limits and mitigation strategies.

**Acceptance Criteria**:
- [ ] Data limit matrix per operation:
  - Max 256-byte pushdata
  - 83-byte OP_RETURN
  - 34-byte ScriptPubKey
- [ ] Impact analysis on:
  - [ ] Miniscript complexity
  - [ ] Taproot leaf count
  - [ ] DLC commitment storage
- [ ] Fallback strategies documented
- [ ] Version compatibility matrix

**Related Code**:
- `src/protocol/bip110.rs` (validation logic)

---

#### [CORE-005] Integrate BIP-110 into Transaction Builder
**Type**: Enhancement  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-004]  

**Description**:
Ensure transaction builder automatically enforces BIP-110 limits and suggests optimizations.

**Acceptance Criteria**:
- [ ] Builder validates all operations against BIP-110
- [ ] Pre-construction checks warn before limit violations
- [ ] Optimization suggestions for data-heavy flows
- [ ] Unit tests for all data limit boundaries

---

### Category: Verification & Trust Model

#### [CORE-006] Formalize Protocol Verification Interface ([GitHub #180](https://github.com/Conxian/lib-conxian-core/issues/180))
**Type**: Architecture  
**Priority**: 🔴 CRITICAL  
**Depends On**: [CORE-001]  

**Description**:
Define `ProtocolVerifier` trait for cross-chain state verification. This bridges core with Nexus (UCV).

**Acceptance Criteria**:
- [x] Trait methods:
  - `verify_chain_state(&self, request: &ProofVerificationRequest) -> Result<ProofVerificationResult, ProtocolVerifierError>`
  - `get_latest_verified_block(&self, chain: &ChainId) -> Result<LatestVerifiedBlock, ProtocolVerifierError>`
  - `verify_transaction_finality(&self, request: &TransactionFinalityRequest) -> Result<TransactionFinalityResult, ProtocolVerifierError>`
- [x] Family-neutral chain identifiers and capability advertisement for all existing `ChainFamily` variants
- [x] Error taxonomy per verification failure mode
- [x] Documentation with 3+ examples

**Related Code**:
- `src/control_model/trust.rs` (VerificationClass, RiskAssessment)

**Delivered in #180**: Core now defines the platform-neutral `ProtocolVerifier`
contract, capability advertisement, proof and latest-block references,
transaction finality statuses, typed fail-closed errors, and invariant tests.
Runtime proof acquisition and orchestration remain outside Core in Nexus,
Gateway, or downstream adapters.

---

#### [CORE-007] Document Chain Family Risk Profiles
**Type**: Documentation  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-006]  

**Description**:
For each chain family, document risk assessment scores (DA, settlement, bridge, etc.) and their implications for UCV routing.

**Acceptance Criteria**:
- [ ] Risk score definitions for all dimensions
- [ ] Chain-specific profiles:
  - Bitcoin: DA=100, Settlement=100
  - Stacks: DA=85, Settlement=80 (Nakamoto)
  - Babylon: DA=90, Settlement=70
  - Liquid: DA=60, Settlement=75
- [ ] Implications for routing logic

---

### Category: Testing & Verification

#### [CORE-008] Define Core Unit Test Coverage Targets
**Type**: Testing  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-001], [CORE-002], [CORE-004]  

**Description**:
Establish test coverage requirements and baseline for core library.

**Acceptance Criteria**:
- [ ] Target coverage: 85% minimum
- [ ] Critical paths: 100% coverage (signing, verification)
- [ ] Test matrix for:
  - [ ] All chain families
  - [ ] All BIP-110 edge cases
  - [ ] All UCS method combinations
- [ ] CI gate: coverage reports required on PR

---

#### [CORE-009] Create Integration Test Framework
**Type**: Testing  
**Priority**: 🟠 HIGH  
**Depends On**: [CORE-001], [CORE-006]  

**Description**:
Define framework for testing core ↔ SDK ↔ Gateway ↔ Nexus integration flows.

**Acceptance Criteria**:
- [ ] Mock implementations of all adapters
- [ ] Test fixtures for common scenarios
- [ ] CI matrix for cross-repo testing
- [ ] Performance benchmarks

---

## Summary

### Issues to Create (9 total)

| ID | Title | Priority | Status |
|:---|:------|:--------:|:-------|
| CORE-001 | UCS Interface Spec | 🔴 | ⏳ Create |
| CORE-002 | BIP-341/342 Validation | 🔴 | ⏳ Create |
| CORE-003 | Signing Flows Docs | 🟠 | ⏳ Create |
| CORE-004 | BIP-110 Compliance Matrix | 🟠 | ⏳ Create |
| CORE-005 | BIP-110 TX Builder Integration | 🟠 | ⏳ Create |
| CORE-006 | Protocol Verifier Trait | 🔴 | ✅ Contract defined in [#180](https://github.com/Conxian/lib-conxian-core/issues/180) |
| CORE-007 | Chain Risk Profiles | 🟠 | ⏳ Create |
| CORE-008 | Test Coverage Targets | 🟠 | ⏳ Create |
| CORE-009 | Integration Test Framework | 🟠 | ⏳ Create |

### Next Steps

1. **Create issues in GitHub** for all items above
2. **Assign reviewers** from core team
3. **Schedule architecture review** for CORE-001 & CORE-006
4. **Generate Phase 1 summary** for SDK/Gateway alignment

