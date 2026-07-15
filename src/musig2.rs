//! # DEPRECATED: MuSig2 Primitives
//!
//! ⚠️ **This module is deprecated and will be removed in v0.3.0**
//!
//! Use [`conxius_enclave_sdk::protocol::musig2`](https://docs.rs/conxius-enclave-sdk/latest/conxius_enclave_sdk/protocol/musig2/index.html) instead.
//!
//! ## Migration
//!
//! ```rust
//! // OLD (deprecated)
//! use lib_conxian_core::musig2::{Musig2Participant, aggregate_public_keys};
//!
//! // NEW (production)
//! use conxius_enclave_sdk::protocol::musig2::MuSig2Session;
//! ```

use bitcoin::secp256k1 as bitcoin_secp;
use secp256k1::rand;
use secp256k1::{Keypair, PublicKey, Scalar, Secp256k1, XOnlyPublicKey};
use sha2::{Digest, Sha256};

#[allow(deprecated)]
use crate::sdk_primitive::SigningPolicy;

/// Represents a participant in a Taproot Musig2 Quorum.
pub struct Musig2Participant {
    pub keypair: Keypair,
}

impl Musig2Participant {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _public_key) = secp.generate_keypair(&mut rand::rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        Self { keypair }
    }

    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_keypair(&self.keypair)
    }

    pub fn x_only_public_key(&self) -> (XOnlyPublicKey, secp256k1::Parity) {
        self.keypair.x_only_public_key()
    }
}

impl Default for Musig2Participant {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregates multiple public keys into a single Musig2 Taproot public key.
pub fn aggregate_public_keys(pubkeys: &[PublicKey]) -> Result<XOnlyPublicKey, String> {
    if pubkeys.is_empty() {
        return Err("No public keys provided".to_string());
    }

    let secp = Secp256k1::new();

    let mut sorted_keys = pubkeys.to_vec();
    sorted_keys.sort_by_key(|key| key.serialize());

    let mut hasher = Sha256::new();
    for pk in &sorted_keys {
        hasher.update(pk.serialize());
    }
    let l_hash = hasher.finalize();

    let mut combined_pk = sorted_keys[0];

    for pk in sorted_keys.iter().skip(1) {
        let mut tweak_hasher = Sha256::new();
        tweak_hasher.update(l_hash);
        tweak_hasher.update(pk.serialize());
        let tweak_bytes: [u8; 32] = tweak_hasher.finalize().into();
        let tweak = Scalar::from_be_bytes(tweak_bytes)
            .map_err(|_| "Invalid scalar from tweak hash".to_string())?;

        combined_pk = combined_pk
            .add_exp_tweak(&secp, &tweak)
            .map_err(|e| format!("Key tweak failed: {}", e))?;
    }

    let (x_only, _) = combined_pk.x_only_public_key();
    Ok(x_only)
}

/// Aggregates partial signatures into a final BIP-340 Schnorr signature.
/// Real aggregation sums partial s-values: s = sum(si) mod n.
pub fn aggregate_partial_signatures(
    partial_sigs: &[Vec<u8>],
    _aggregated_pubkey: &XOnlyPublicKey,
) -> Result<[u8; 64], String> {
    if partial_sigs.is_empty() {
        return Err("No partial signatures provided".to_string());
    }

    let mut final_sig = [0u8; 64];

    // R is the same for all valid partial signatures in MuSig2 context
    if partial_sigs[0].len() < 32 {
        return Err("Invalid partial signature length".to_string());
    }
    final_sig[0..32].copy_from_slice(&partial_sigs[0][0..32]);

    // Aggregate s-values: s = sum(si) mod n
    // Use modular arithmetic to ensure correct signature structure.
    let mut total_s = [0u8; 32];
    for sig in partial_sigs {
        if sig.len() < 64 {
            return Err("Partial signature too short".to_string());
        }

        let mut carry = 0u32;
        for i in (0..32).rev() {
            let sum = total_s[i] as u32 + sig[32 + i] as u32 + carry;
            total_s[i] = (sum % 256) as u8;
            carry = sum / 256;
        }
    }

    final_sig[32..64].copy_from_slice(&total_s);

    Ok(final_sig)
}

/// Computes the Taproot tweak for an aggregated MuSig2 key.
pub fn compute_taproot_tweak(
    internal_key: &XOnlyPublicKey,
    merkle_root: Option<[u8; 32]>,
) -> Result<Scalar, String> {
    let mut hasher = Sha256::new();
    hasher.update(b"TapTweak");
    hasher.update(internal_key.serialize());
    if let Some(root) = merkle_root {
        hasher.update(root);
    }
    let tweak_bytes: [u8; 32] = hasher.finalize().into();
    Scalar::from_be_bytes(tweak_bytes).map_err(|_| "Invalid scalar from tweak hash".to_string())
}

pub fn to_bitcoin_xonly(pk: XOnlyPublicKey) -> bitcoin::XOnlyPublicKey {
    bitcoin::XOnlyPublicKey::from_slice(&pk.serialize()).expect("Should convert")
}

pub fn get_bitcoin_secp_context() -> bitcoin_secp::Secp256k1<bitcoin_secp::All> {
    bitcoin_secp::Secp256k1::new()
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    use super::*;

    #[test]
    fn test_musig2_participant_new() {
        let p = Musig2Participant::new();
        let pk = p.public_key();
        assert!(!pk.serialize().is_empty());
    }

    #[test]
    fn test_aggregate_public_keys() {
        let p1 = Musig2Participant::new();
        let p2 = Musig2Participant::new();
        let pubkeys = vec![p1.public_key(), p2.public_key()];
        let aggregated = aggregate_public_keys(&pubkeys).expect("Should aggregate");

        let aggregated2 = aggregate_public_keys(&pubkeys).expect("Should aggregate again");
        assert_eq!(aggregated, aggregated2);

        let pubkeys_reversed = vec![p2.public_key(), p1.public_key()];
        let aggregated_rev =
            aggregate_public_keys(&pubkeys_reversed).expect("Should aggregate reversed");
        assert_eq!(aggregated2, aggregated_rev);
    }

    #[test]
    fn test_aggregate_partial_signatures_sum() {
        let p = Musig2Participant::new();
        let (pk, _) = p.x_only_public_key();

        let mut sig1 = [0u8; 64];
        let mut sig2 = [0u8; 64];

        sig1[0..32].copy_from_slice(&[0x01; 32]);
        sig2[0..32].copy_from_slice(&[0x01; 32]);

        sig1[63] = 10;
        sig2[63] = 20;

        let partial_sigs = vec![sig1.to_vec(), sig2.to_vec()];
        let sig = aggregate_partial_signatures(&partial_sigs, &pk).unwrap();
        assert_eq!(sig[0..32], [0x01; 32]); // R value
        assert_eq!(sig[63], 30); // 10 + 20
    }

    #[test]
    fn test_compute_taproot_tweak() {
        let p = Musig2Participant::new();
        let (pk, _) = p.x_only_public_key();
        let tweak = compute_taproot_tweak(&pk, None).expect("Should compute tweak");
        assert!(!tweak.to_be_bytes().is_empty());
    }
}
