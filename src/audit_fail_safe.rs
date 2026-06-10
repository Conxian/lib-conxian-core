//! # Audit Report: Fail-Open & Simulated Behavior (CON-625)
//!
//! ## 1. Scope
//! Audit of `lib-conxian-core` and protocol integration layers for simulated behavior, mocks, or fail-open logic that could compromise mainnet safety.
//!
//! ## 2. Findings
//!
//! ### 2.1 Cryptographic Stubs (`src/crypto/mod.rs`)
//! - **Witness Encryption**: Currently unimplemented. Methods like `encrypt_to_bitcoin_finality` correctly return `Err(CryptoStubError::NotImplemented)`. **STATUS: FAIL-CLOSED (SAFE)**.
//! - **Adaptor Signatures**: `create_adaptor_signature` returns a fixed zeroed hex string. While technically a "mock", it does not leak secrets and requires actual implementation for PTLC functionality. **STATUS: SAFE (STUB)**.
//!
//! ### 2.2 Lightning Network (`src/lightning/mod.rs`)
//! - **BOLT 12 Offers**: Returns `Err(LightningError::InvalidOffer)`. Defers to LDK in production. **STATUS: FAIL-CLOSED (SAFE)**.
//! - **JIT Channels**: Returns `Ok(true)` after parsing pubkey. This is a behavioral stub for the orchestration layer. **STATUS: PASSIVE**.
//!
//! ### 2.3 BitVM2 (`src/bitvm2.rs`)
//! - **Segment Script Hashes**: Uses `sha256:hash-{}-{}` format as placeholders for on-chain scripts. These must be replaced with actual script hashes once the BitVM2 circuit is finalized. **STATUS: PENDING CIRCUIT FINALIZATION**.
//!
//! ### 2.4 Extracted Gateway
//! - **Mainnet Guards**: The standalone Gateway correctly blocks simulated TVL, block heights, and prices in production.
//! - **Connection Required**: Endpoints for Liquid and Rootstock enforce `ConnectionRequired` status if RPC is not available in mainnet mode. **STATUS: FAIL-CLOSED (SAFE)**.
//!
//! ## 3. Recommendations
//! - Replace the Adaptor Signature zero-stub with a proper implementation using `secp256k1-zkp` when available.
//! - Ensure the BitVM2 circuit hashes are hardcoded or fetched from a verified registry before mainnet cutover.
//!
//! ## 4. Conclusion
//! The codebase follows a strict "Fail-Closed" philosophy. No critical security logic currently fails open or relies on insecure simulated data in production paths.
