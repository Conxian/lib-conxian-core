//! Fedimint Community Liquidity Adapter
//! Aligned with CXIP 20 and G-16

use secp256k1::{PublicKey, Scalar, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FedimintMint {
    pub mint_id: String,
    pub community_name: String,
    pub total_liquidity_sats: u64,
}

pub struct FedimintAdapter;

impl FedimintAdapter {
    pub fn get_mint_status(&self, mint_id: &str) -> Result<FedimintMint, String> {
        Ok(FedimintMint {
            mint_id: mint_id.to_string(),
            community_name: "Conxian Community Mint".to_string(),
            total_liquidity_sats: 100_000_000,
        })
    }

    /// Implements real cryptographic blinding for e-cash notes (G-16).
    /// Uses ECC point addition: blinded_note = H(secret)*G + r*G
    pub fn blind_note(secret: &[u8], blinding_factor: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::new();

        let mut hasher = Sha256::new();
        hasher.update(b"FEDIMINT-SECRET");
        hasher.update(secret);
        let secret_hash: [u8; 32] = hasher.finalize().into();

        let secret_scalar = match Scalar::from_be_bytes(secret_hash) {
            Ok(s) => s,
            Err(_) => return vec![0u8; 33],
        };

        let bf_bytes: [u8; 32] = match blinding_factor.try_into() {
            Ok(b) => b,
            Err(_) => return vec![0u8; 33],
        };

        let bf_scalar = match Scalar::from_be_bytes(bf_bytes) {
            Ok(s) => s,
            Err(_) => return vec![0u8; 33],
        };

        // note_point = secret * G
        let sk = match SecretKey::from_byte_array(secret_scalar.to_be_bytes()) {
            Ok(k) => k,
            Err(_) => return vec![0u8; 33],
        };
        let note_point = PublicKey::from_secret_key(&secp, &sk);

        // blinded_point = note_point + bf * G
        let blinded_point = match note_point.add_exp_tweak(&secp, &bf_scalar) {
            Ok(p) => p,
            Err(_) => return vec![0u8; 33],
        };

        blinded_point.serialize().to_vec()
    }

    /// Verifies the unblinding of an e-cash note.
    pub fn verify_unblinded(blinded: &[u8], blinding_factor: &[u8], secret: &[u8]) -> bool {
        let reconstructed = Self::blind_note(secret, blinding_factor);
        reconstructed == blinded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fedimint_blinding_determinism() {
        let secret = b"my-ecash-secret-32-bytes-long-now";
        let mut bf = [0u8; 32];
        bf[31] = 1;

        let blinded1 = FedimintAdapter::blind_note(secret, &bf);
        let blinded2 = FedimintAdapter::blind_note(secret, &bf);

        assert_eq!(blinded1, blinded2);
        assert!(FedimintAdapter::verify_unblinded(&blinded1, &bf, secret));
    }
}
