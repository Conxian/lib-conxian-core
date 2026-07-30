# CXIP-21: Universal Chain Adapter Standard

## 1. Abstract
This proposal defines a standardized interface (trait) for multi-chain support within the Conxian Vault SDK. By establishing a common contract for Bitcoin, EVM, and other chain families, we enable the SDK to coordinate cross-chain operations with uniform risk and trust-tier enforcement.

## 2. Motivation
Conxian aims to be the universal payment infrastructure for the sovereign enterprise. Currently, chain-specific logic is fragmented. A universal adapter standard allows integrators to interact with multiple ecosystems (e.g., Bitcoin L2s, EVM rollups) using a single, predictable API.

## 3. Specification

### 3.1 The `UniversalChainAdapter` Trait
The core of this standard is the `UniversalChainAdapter` trait, which must be implemented by all chain-specific modules.

```rust
pub trait UniversalChainAdapter {
    /// Returns the chain family (e.g., Bitcoin, EVM).
    fn family(&self) -> ChainFamily;

    /// Returns the specific chain identifier.
    fn chain(&self) -> Chain;

    /// Validates an address for the target chain.
    fn validate_address(&self, address: &str) -> Result<(), String>;

    /// Estimates the fee for a transaction.
    fn estimate_fee(&self, tx_params: &TxParams) -> Result<u64, String>;

    /// Returns the trust tier of the chain's bridge/messaging lane.
    fn trust_tier(&self) -> TrustTier;
}
```

### 3.2 Metadata Requirements
Every adapter must expose metadata that aligns with `control_model.rs`, including support for the T1–T4 trust taxonomy.

## 4. Implementation Strategy
1. Define the trait in `src/adapters/mod.rs`.
2. Implement `BitcoinAdapter` in `src/bitcoin/mod.rs`.
3. Implement `EvmAdapter` in `src/evm/mod.rs` (New).
4. Update `conxius-enclave-sdk` to consume these adapters via a registry.

## 5. Backward Compatibility
This proposal is additive and does not break existing Vault SDK signatures.

## 6. Security Considerations
Adapters must fail closed if trust metadata is missing or if verification requirements (e.g., light-client proofs) are not met.
