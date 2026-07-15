//! # DEPRECATED: Wallet Primitives
//!
//! ⚠️ **This module is deprecated and will be removed in v0.3.0**
//!
//! For key management, use the `k256` crate directly or `bdk_wallet`.
//!
//! ## Migration
//!
//! ```rust
//! // OLD (deprecated)
//! use lib_conxian_core::Wallet;
//! let wallet = Wallet::from_private_key_hex("...")?;
//! let signature = wallet.sign(message);
//!
//! // NEW (production)
//! use k256::ecdsa::{signature::Signer, Signature, SigningKey};
//! use sha2::{Digest, Sha256};
//!
//! let signing_key = SigningKey::from_slice(&hex::decode("...")?)?;
//! let mut hasher = Sha256::new();
//! hasher.update(message.as_bytes());
//! let signature: Signature = signing_key.sign(&hasher.finalize());
//! ```

use anyhow::Context;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

/// **DEPRECATED**: Use `k256` crate directly for key management.
///
/// This struct is kept for backwards compatibility but will be removed in v0.3.0.
#[deprecated(
    since = "0.2.10",
    note = "Use k256 crate directly. See docs/MIGRATION.md"
)]
#[derive(Clone)]
pub struct Wallet {
    signing_key: SigningKey,
}

impl Wallet {
    #[deprecated(
        since = "0.2.10",
        note = "Use k256::ecdsa::SigningKey::from_slice() instead"
    )]
    pub fn from_private_key_hex(hex_key: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_key.trim()).with_context(|| "invalid hex in private key")?;
        Self::from_private_key_bytes(&bytes)
    }

    #[deprecated(
        since = "0.2.10",
        note = "Use k256::ecdsa::SigningKey::from_slice() instead"
    )]
    pub fn from_private_key_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let signing_key = SigningKey::from_slice(bytes).with_context(|| "invalid private key")?;
        Ok(Self { signing_key })
    }

    #[deprecated(
        since = "0.2.10",
        note = "Use k256::ecdsa::VerifyingKey directly"
    )]
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_sec1_point(true)
            .as_bytes()
            .to_vec()
    }

    #[deprecated(
        since = "0.2.10",
        note = "Use k256 crate directly"
    )]
    pub fn public_key(&self) -> String {
        hex::encode(self.public_key_bytes())
    }

    #[deprecated(
        since = "0.2.10",
        note = "Implement in application layer"
    )]
    pub fn stacks_address_hash(&self) -> String {
        let pubkey = self.public_key_bytes();
        let sha2 = Sha256::digest(&pubkey);
        let hash160 = Ripemd160::digest(sha2);
        hex::encode(hash160)
    }

    #[deprecated(
        since = "0.2.10",
        note = "Use k256::ecdsa::SigningKey::sign() instead"
    )]
    pub fn sign(&self, message: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let digest = hasher.finalize();
        let signature: Signature = self.signing_key.sign(&digest);
        hex::encode(signature.to_bytes())
    }
}

/// A "Sovereign Handshake" visualizing state change for hardware approval.
pub struct SovereignHandshake {
    pub proposal_id: String,
    pub intent_type: String,
    pub state_change_summary: String,
    pub timelock_end_block: u64,
}

impl SovereignHandshake {
    pub fn new(proposal_id: String, intent_type: String, summary: String, end_block: u64) -> Self {
        Self {
            proposal_id,
            intent_type,
            state_change_summary: summary,
            timelock_end_block: end_block,
        }
    }

    pub fn visualize(&self) -> String {
        format!(
            "SOVEREIGN HANDSHAKE REQUIRED\n             ===========================\n             Proposal ID: {}\n             Intent Type: {}\n             Change:      {}\n             Timelock:    Until block {}\n             ===========================\n             Approve this agent-drafted action?",
            self.proposal_id, self.intent_type, self.state_change_summary, self.timelock_end_block
        )
    }
}
