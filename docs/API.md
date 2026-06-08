# lib-conxian-core / Vault SDK API Reference v0.2.5

## 1. Overview
This library provides the Rust-native API for Conxian protocol primitives. It is intended to be used as a dependency (`lib-conxian-core`) by the standalone Gateway, Wallet, and third-party integrators.

## 2. Core Modules

### Vault SDK (`sdk_primitive`)
The primary interface for hardware-anchored signing and policy enforcement.
- `VaultSDK::new(policy: SigningPolicy)`: Initializes a new SDK instance.
- `VaultSDK::sign_intent(intent: FinancialIntent)`: Validates and signs a cross-chain intent.

### Control Model (`control_model`)
Canonical types for protocol orchestration and trust.
- `StateProposal`: Unified model for external settlement triggers (CON-162).
- `TrustTier`: Approved classification for bridge/messaging security (CON-791).
- `PartnerLead`: Intake model for ecosystem integrations (CON-63).

### Anchoring (`anchoring`)
Models for decentralized state persistence.
- `AnchoringRequest`: Payload for committing state roots to Tableland or L1.
- `AnchoringReceipt`: Cryptographic proof of state commitment.

### Bitcoin (`bitcoin`)
Advanced Bitcoin-native primitives.
- `MuSig2`: BIP327-compliant key aggregation and signing (CON-145).
- `BitVM2`: Segment generation and optimistic fraud-proof verification (CON-464).

## 3. Trust Tier Policy (CON-791)
The library enforces explicit trust-tier metadata for all cross-domain operations:
- **T1: Strict**: Sovereign verified (e.g., IBC light-clients).
- **T2: Managed**: Hybrid verified with independent attesters.
- **T3: Expedient**: Attester network with caps and kill-switches.
- **T4: ObserverOnly**: Not allowed in production.

## 4. Integration Guidelines
Implementation details for runtime orchestration, network IO, and database persistence live in the standalone `conxian-gateway` repository. This library focuses exclusively on stable interfaces and protocol-bearing logic.
