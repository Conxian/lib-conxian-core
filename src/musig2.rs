use bitcoin::secp256k1 as bitcoin_secp;
use k256::elliptic_curve::PrimeField;
use k256::Scalar as K256Scalar;
use secp256k1::rand;
use secp256k1::{Keypair, PublicKey, Scalar, Secp256k1, XOnlyPublicKey};
use sha2::{Digest, Sha256};

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

/// Aggregates MuSig2 partial signatures into a final BIP-340 Schnorr signature.
///
/// Each partial signature is a 32-byte scalar `s_i`. This function sums all
/// `s_i` modulo the secp256k1 curve order and returns the complete `(R, s)`
/// BIP-340 signature using the provided aggregated nonce.
///
/// BIP-327: `PartialSigAgg(psig_1, ..., psig_n) → (R, s)` where
/// `s = (s_1 + ... + s_n) mod p`.
pub fn aggregate_partial_signatures(
    partial_sigs: &[Vec<u8>],
    aggregated_nonce: &XOnlyPublicKey,
) -> Result<[u8; 64], String> {
    if partial_sigs.is_empty() {
        return Err("No partial signatures provided".to_string());
    }

    let mut sum = K256Scalar::ZERO;
    for psig in partial_sigs {
        if psig.len() != 32 {
            return Err(format!(
                "Partial signature must be 32 bytes, got {}",
                psig.len()
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(psig);
        let fb = k256::FieldBytes::from(arr);
        let ct = K256Scalar::from_repr(fb);
        if ct.is_none().into() {
            return Err("Partial signature scalar out of range".to_string());
        }
        sum += ct.unwrap();
    }

    let mut sig = [0u8; 64];
    sig[..32].copy_from_slice(&aggregated_nonce.serialize());
    let sum_bytes: [u8; 32] = k256::FieldBytes::from(&sum).into();
    sig[32..].copy_from_slice(&sum_bytes);
    Ok(sig)
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
    fn test_aggregate_partial_signatures_deterministic() {
        let p = Musig2Participant::new();
        let (nonce, _) = p.x_only_public_key();

        // Two known scalars: 1 and 2. Sum = 3.
        let s1 = {
            let mut bytes = [0u8; 32];
            bytes[31] = 1;
            bytes.to_vec()
        };
        let s2 = {
            let mut bytes = [0u8; 32];
            bytes[31] = 2;
            bytes.to_vec()
        };

        let partial_sigs = vec![s1.clone(), s2.clone()];
        let sig = aggregate_partial_signatures(&partial_sigs, &nonce).unwrap();
        assert_eq!(sig.len(), 64);

        // R should be the nonce
        assert_eq!(&sig[..32], &nonce.serialize());

        // s = 1 + 2 = 3
        assert_eq!(sig[63], 3);

        // Determinism: same inputs produce same output
        let sig2 = aggregate_partial_signatures(&partial_sigs, &nonce).unwrap();
        assert_eq!(sig, sig2);

        // Order-independent: same sum regardless of input order
        let sig_rev = aggregate_partial_signatures(&[s2, s1], &nonce).unwrap();
        assert_eq!(sig, sig_rev);
    }

    #[test]
    fn test_aggregate_partial_signatures_commutative() {
        let p1 = Musig2Participant::new();
        let (nonce, _) = p1.x_only_public_key();

        // Three random scalars: 5, 10, 15. Sum = 30 in any order.
        let make_scalar = |v: u8| {
            let mut bytes = [0u8; 32];
            bytes[31] = v;
            bytes.to_vec()
        };

        let a = make_scalar(5);
        let b = make_scalar(10);
        let c = make_scalar(15);

        let sig_abc =
            aggregate_partial_signatures(&[a.clone(), b.clone(), c.clone()], &nonce).unwrap();
        let sig_cba = aggregate_partial_signatures(&[c, b, a], &nonce).unwrap();

        assert_eq!(sig_abc, sig_cba);
        assert_eq!(sig_abc[63], 30);
    }

    #[test]
    fn test_aggregate_partial_signatures_empty() {
        let p = Musig2Participant::new();
        let (nonce, _) = p.x_only_public_key();
        let err = aggregate_partial_signatures(&[], &nonce).unwrap_err();
        assert!(err.contains("No partial signatures"));
    }

    #[test]
    fn test_aggregate_partial_signatures_invalid_length() {
        let p = Musig2Participant::new();
        let (nonce, _) = p.x_only_public_key();
        let bad = vec![vec![0u8; 31]]; // one byte short
        let err = aggregate_partial_signatures(&bad, &nonce).unwrap_err();
        assert!(err.contains("must be 32 bytes"));
    }

    #[test]
    fn test_aggregate_partial_signatures_modular_arithmetic() {
        let p = Musig2Participant::new();
        let (nonce, _) = p.x_only_public_key();

        let make_scalar = |v: u8| {
            let mut bytes = [0u8; 32];
            bytes[31] = v;
            bytes.to_vec()
        };

        // (order - 1) + 1 ≡ 0 (mod n) — wraparound
        // secp256k1 order - 1 as big-endian bytes
        let n_minus_1 =
            hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364140")
                .unwrap();

        let sig1 =
            aggregate_partial_signatures(&[n_minus_1.clone(), make_scalar(1)], &nonce).unwrap();
        let sig2 = aggregate_partial_signatures(&[make_scalar(1), make_scalar(1)], &nonce).unwrap();

        // (n-1 + 1) mod n ≡ 0, while 1+1 = 2
        assert_ne!(sig1, sig2, "modular wraparound vs simple addition");
        assert_eq!(sig2[63], 2, "1+1 = 2");
    }

    #[test]
    fn test_aggregate_partial_signatures_out_of_range_rejected() {
        let p = Musig2Participant::new();
        let (nonce, _) = p.x_only_public_key();

        // All-0xFF bytes exceed the secp256k1 order → invalid scalar
        let invalid = vec![vec![0xFFu8; 32]];
        let err = aggregate_partial_signatures(&invalid, &nonce).unwrap_err();
        assert!(err.contains("out of range"));
    }

    #[test]
    fn test_compute_taproot_tweak() {
        let p = Musig2Participant::new();
        let (pk, _) = p.x_only_public_key();
        let tweak = compute_taproot_tweak(&pk, None).expect("Should compute tweak");
        assert!(!tweak.to_be_bytes().is_empty());
    }
}
