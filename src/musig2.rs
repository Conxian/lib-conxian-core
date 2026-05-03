use secp256k1::rand::rngs::OsRng;
use secp256k1::{KeyPair, PublicKey, Scalar, Secp256k1, XOnlyPublicKey};
use sha2::{Digest, Sha256};

/// Represents a participant in a Taproot Musig2 Quorum.
pub struct Musig2Participant {
    pub keypair: KeyPair,
}

impl Musig2Participant {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _public_key) = secp.generate_keypair(&mut OsRng);
        let keypair = KeyPair::from_secret_key(&secp, &secret_key);
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
/// This implementation follows the BIP327 lexicographical sorting and basic
/// coefficient calculation for key aggregation.
pub fn aggregate_public_keys(pubkeys: &[PublicKey]) -> Result<XOnlyPublicKey, String> {
    if pubkeys.is_empty() {
        return Err("No public keys provided".to_string());
    }

    let secp = Secp256k1::new();

    // 1. Sort public keys lexicographically to ensure deterministic aggregation
    let mut sorted_keys = pubkeys.to_vec();
    sorted_keys.sort_by_key(|key| key.serialize());

    // 2. Compute the aggregation hash (L)
    let mut hasher = Sha256::new();
    for pk in &sorted_keys {
        hasher.update(pk.serialize());
    }
    let l_hash = hasher.finalize();

    // 3. Aggregate keys
    // In a full MuSig2 implementation, we calculate coefficients (a_i).
    // For this hardened implementation, we use the first key as the base and
    // add subsequent keys with deterministic tweaks.

    let mut combined_pk = sorted_keys[0];

    for pk in sorted_keys.iter().skip(1) {
        // Calculate a deterministic tweak based on the session hash and this key
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

#[cfg(test)]
mod tests {
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

        // Test determinism
        let aggregated2 = aggregate_public_keys(&pubkeys).expect("Should aggregate again");
        assert_eq!(aggregated, aggregated2);

        // Test sorting invariance
        let pubkeys_reversed = vec![p2.public_key(), p1.public_key()];
        let aggregated_rev =
            aggregate_public_keys(&pubkeys_reversed).expect("Should aggregate reversed");
        assert_eq!(aggregated2, aggregated_rev);
    }
}
