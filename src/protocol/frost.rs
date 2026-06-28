//! FROST: Flexible Round-Optimized Schnorr Threshold Signatures.
//!
//! RFC 9591 threshold signing via `frost-secp256k1-tr` (ZcashFoundation, NCC Group audited).
//! Produces BIP-340 compatible 64-byte Schnorr signatures with t-of-n threshold security.
//!
//! # Example — Trusted dealer 2-of-3 signing
//!
//! ```
//! # use std::collections::BTreeMap;
//! # use frost_secp256k1_tr::rand_core::OsRng;
//! use frost_secp256k1_tr::SigningPackage;
//! use lib_conxian_core::protocol::frost::FrostManager;
//!
//! let mut rng = OsRng;
//! let (shares, pubkey_pkg) = FrostManager::trusted_dealer_keygen(3, 2, &mut rng)
//!     .expect("keygen should succeed");
//!
//! let msg = b"hello world";
//! let mut commitments_map = BTreeMap::new();
//! let mut nonces_map = BTreeMap::new();
//! let mut key_packages = Vec::new();
//!
//! // Round 1: each participant commits
//! for share in shares.values() {
//!     let kp = FrostManager::into_key_package(share, &pubkey_pkg);
//!     let (nonces, comm) = FrostManager::commit(&kp.signing_share(), &mut rng);
//!     commitments_map.insert(*kp.identifier(), comm);
//!     nonces_map.insert(*kp.identifier(), nonces);
//!     key_packages.push(kp);
//! }
//!
//! // Round 2: sign
//! let sig_pkg = SigningPackage::new(commitments_map, msg);
//! let mut sig_shares = BTreeMap::new();
//! for kp in &key_packages {
//!     let nonces = &nonces_map[kp.identifier()];
//!     let share = FrostManager::sign(&sig_pkg, nonces, kp).unwrap();
//!     sig_shares.insert(*kp.identifier(), share);
//! }
//!
//! // Aggregate and verify
//! let sig = frost_secp256k1_tr::aggregate(&sig_pkg, &sig_shares, &pubkey_pkg).unwrap();
//! assert_eq!(sig.serialize().unwrap().len(), 64);
//! FrostManager::verify(pubkey_pkg.verifying_key(), msg, &sig).unwrap();
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use frost_secp256k1_tr as frost;
use frost_secp256k1_tr::rand_core::OsRng;
use frost_secp256k1_tr::{
    keys::{KeyPackage, PublicKeyPackage, SecretShare},
    round1::{SigningCommitments, SigningNonces},
    round2::SignatureShare,
    Identifier, Signature, SigningPackage, VerifyingKey,
};

/// Represents a secret key share in the FROST threshold scheme (Round 2 scaffolding).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostKeyShare {
    /// Participant index (1-indexed).
    pub index: u32,
    /// Secret share of the group key.
    pub share: Vec<u8>,
    /// Public key corresponding to this share.
    pub public_key: Vec<u8>,
}

/// A commitment to a polynomial used in VSS (Verifiable Secret Sharing).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrostShareCommitment {
    pub index: u32,
    pub commitment_points: Vec<Vec<u8>>,
}

/// An encrypted share for distribution during Round 2.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedFrostShare {
    pub from_index: u32,
    pub to_index: u32,
    pub encrypted_payload: Vec<u8>,
}

/// Status of a FROST signing session.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FrostSessionStatus {
    Open,
    Committed,
    Signed,
    Aborted,
}

/// Manager for FROST threshold signature lifecycle.
///
/// Wraps `frost-secp256k1-tr` v3 with a simplified API for key generation,
/// two-round signing, and signature aggregation.
pub struct FrostManager;

impl FrostManager {
    /// Trusted dealer key generation: t-of-n threshold.
    ///
    /// Returns secret shares (one per participant) and the group public key package.
    pub fn trusted_dealer_keygen(
        max_signers: u16,
        min_signers: u16,
        rng: &mut OsRng,
    ) -> Result<(BTreeMap<Identifier, SecretShare>, PublicKeyPackage), String> {
        if min_signers < 2 || max_signers < 2 {
            return Err("min_signers and max_signers must be >= 2 per RFC 9591".into());
        }
        if min_signers > max_signers {
            return Err("min_signers must not exceed max_signers".into());
        }

        let identifiers = frost::keys::IdentifierList::Default;
        frost::keys::generate_with_dealer(max_signers, min_signers, identifiers, rng)
            .map_err(|e| format!("FROST keygen failed: {:?}", e))
    }

    /// Prepares encrypted shares for distribution (Round 2 scaffolding).
    /// This requires a shared secret derived via Diffie-Hellman between participants.
    pub fn prepare_distribution_shares(
        from_share: &FrostKeyShare,
        target_indices: &[u32],
    ) -> Vec<EncryptedFrostShare> {
        target_indices
            .iter()
            .map(|&to_idx| {
                let mut hasher = Sha256::new();
                hasher.update(from_share.share.as_slice());
                hasher.update(to_idx.to_be_bytes());
                let payload = hasher.finalize().to_vec();

                EncryptedFrostShare {
                    from_index: from_share.index,
                    to_index: to_idx,
                    encrypted_payload: payload,
                }
            })
            .collect()
    }

    /// Convert a secret share + public key package into a key package for signing.
    pub fn into_key_package(share: &SecretShare, pubkey_pkg: &PublicKeyPackage) -> KeyPackage {
        let verifying_share = *pubkey_pkg
            .verifying_shares()
            .get(share.identifier())
            .expect("identifier must be in the public key package");

        KeyPackage::new(
            *share.identifier(),
            *share.signing_share(),
            verifying_share,
            *pubkey_pkg.verifying_key(),
            pubkey_pkg.min_signers().unwrap_or(0),
        )
    }

    /// Round 1: generate signing nonces and public commitments.
    pub fn commit(
        signing_share: &frost::keys::SigningShare,
        rng: &mut OsRng,
    ) -> (SigningNonces, SigningCommitments) {
        frost::round1::commit(signing_share, rng)
    }

    /// Build a signing package from one participant's commitments.
    /// In a full multi-party setup, collect commitments from all participants.
    pub fn new_signing_package(
        identifier: Identifier,
        commitments: &SigningCommitments,
        message: &[u8],
    ) -> SigningPackage {
        let commitments_map = BTreeMap::from([(identifier, *commitments)]);
        SigningPackage::new(commitments_map, message)
    }

    /// Round 2: produce a signature share.
    pub fn sign(
        signing_package: &SigningPackage,
        signer_nonces: &SigningNonces,
        key_package: &KeyPackage,
    ) -> Result<SignatureShare, String> {
        frost::round2::sign(signing_package, signer_nonces, key_package)
            .map_err(|e| format!("FROST sign failed: {:?}", e))
    }

    /// Aggregate signature shares into a final BIP-340 Schnorr signature.
    ///
    /// Convenience for a single signer; use `frost_secp256k1_tr::aggregate` directly
    /// for multi-party aggregation.
    pub fn aggregate(
        signing_package: &SigningPackage,
        identifier: Identifier,
        signature_share: &SignatureShare,
        public_key_package: &PublicKeyPackage,
    ) -> Result<Signature, String> {
        let shares_map = BTreeMap::from([(identifier, *signature_share)]);
        frost::aggregate(signing_package, &shares_map, public_key_package)
            .map_err(|e| format!("FROST aggregate failed: {:?}", e))
    }

    /// Verify a FROST signature against the group verifying key.
    pub fn verify(
        verifying_key: &VerifyingKey,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), String> {
        verifying_key
            .verify(message, signature)
            .map_err(|e| format!("FROST verify failed: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use frost_secp256k1_tr::rand_core::OsRng;

    fn do_keygen(max: u16, min: u16) -> (BTreeMap<Identifier, SecretShare>, PublicKeyPackage) {
        let mut rng = OsRng;
        FrostManager::trusted_dealer_keygen(max, min, &mut rng).unwrap()
    }

    #[test]
    fn test_frost_round_2_distribution() {
        // Round 2 scaffolding: create a placeholder FrostKeyShare and test share distribution
        let from_share = FrostKeyShare {
            index: 1,
            share: vec![0u8; 32],
            public_key: vec![0u8; 33],
        };
        let encrypted = FrostManager::prepare_distribution_shares(&from_share, &[2, 3]);
        assert_eq!(encrypted.len(), 2);
        assert_eq!(encrypted[0].to_index, 2);
        assert_eq!(encrypted[0].from_index, 1);
    }

    #[test]
    fn test_trusted_dealer_keygen_3_of_5() {
        let (shares, pkg) = do_keygen(5, 3);
        assert_eq!(shares.len(), 5);
        assert_eq!(pkg.min_signers(), Some(3));
    }

    #[test]
    fn test_trusted_dealer_keygen_2_of_3() {
        let (shares, pkg) = do_keygen(3, 2);
        assert_eq!(shares.len(), 3);
        assert_eq!(pkg.min_signers(), Some(2));
    }

    #[test]
    fn test_trusted_dealer_keygen_2_of_2() {
        let (shares, pkg) = do_keygen(2, 2);
        assert_eq!(shares.len(), 2);
        assert_eq!(pkg.min_signers(), Some(2));
    }

    #[test]
    fn test_trusted_dealer_invalid_min_lt_2() {
        let mut rng = OsRng;
        let result = FrostManager::trusted_dealer_keygen(3, 1, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_trusted_dealer_invalid_max_lt_2() {
        let mut rng = OsRng;
        let result = FrostManager::trusted_dealer_keygen(1, 2, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_trusted_dealer_invalid_min_gt_max() {
        let mut rng = OsRng;
        let result = FrostManager::trusted_dealer_keygen(2, 5, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_into_key_package() {
        let (shares, pkg) = do_keygen(3, 2);
        let share = shares.values().next().unwrap();
        let kp = FrostManager::into_key_package(share, &pkg);
        assert_eq!(kp.identifier(), share.identifier());
        assert_eq!(*kp.min_signers(), 2);
    }

    #[test]
    fn test_full_signing_cycle_2_of_2() {
        let mut rng = OsRng;
        let (shares, pkg) = do_keygen(2, 2);

        let msg = b"test message for FROST";
        let mut commitments_map = BTreeMap::new();
        let mut nonces_map = BTreeMap::new();
        let mut key_packages = Vec::new();

        for share in shares.values() {
            let kp = FrostManager::into_key_package(share, &pkg);
            let (nonces, comm) = FrostManager::commit(kp.signing_share(), &mut rng);
            commitments_map.insert(*kp.identifier(), comm);
            nonces_map.insert(*kp.identifier(), nonces);
            key_packages.push(kp);
        }

        let sig_pkg = SigningPackage::new(commitments_map, msg);

        let mut sig_shares = BTreeMap::new();
        for kp in &key_packages {
            let nonces = &nonces_map[kp.identifier()];
            let share = FrostManager::sign(&sig_pkg, nonces, kp).unwrap();
            sig_shares.insert(*kp.identifier(), share);
        }

        let sig = frost_secp256k1_tr::aggregate(&sig_pkg, &sig_shares, &pkg).unwrap();
        let sig_bytes = sig.serialize().unwrap();
        assert_eq!(sig_bytes.len(), 64);
        assert_ne!(sig_bytes, [0u8; 64], "signature should not be all zeros");

        FrostManager::verify(pkg.verifying_key(), msg, &sig).unwrap();
    }

    #[test]
    fn test_full_signing_cycle_2_of_3() {
        let mut rng = OsRng;
        let (shares, pkg) = FrostManager::trusted_dealer_keygen(3, 2, &mut rng).unwrap();

        let msg = b"2-of-3 threshold signing test";
        let mut commitments_map = BTreeMap::new();
        let mut key_packages = Vec::new();
        let mut nonces_list = Vec::new();

        for share in shares.values() {
            let kp = FrostManager::into_key_package(share, &pkg);
            let (nonces, comm) = FrostManager::commit(kp.signing_share(), &mut rng);
            commitments_map.insert(*kp.identifier(), comm);
            nonces_list.push(nonces);
            key_packages.push(kp);
        }

        let sig_pkg = SigningPackage::new(commitments_map, msg);

        let mut sig_shares = BTreeMap::new();
        for (kp, nonces) in key_packages.iter().zip(nonces_list.iter()) {
            let share = FrostManager::sign(&sig_pkg, nonces, kp).unwrap();
            sig_shares.insert(*kp.identifier(), share);
        }

        let sig = frost_secp256k1_tr::aggregate(&sig_pkg, &sig_shares, &pkg).unwrap();
        let sig_bytes = sig.serialize().unwrap();
        assert_eq!(sig_bytes.len(), 64);
        assert_ne!(sig_bytes, [0u8; 64]);

        FrostManager::verify(pkg.verifying_key(), msg, &sig).unwrap();
    }

    /// Helper: full threshold signing round for a t-of-n setup.
    fn full_sign(
        shares: &BTreeMap<Identifier, SecretShare>,
        pkg: &PublicKeyPackage,
        msg: &[u8],
    ) -> Signature {
        let mut rng = OsRng;
        let mut commitments_map = BTreeMap::new();
        let mut nonces_map = BTreeMap::new();
        let mut kps: Vec<KeyPackage> = Vec::new();

        for share in shares.values() {
            let kp = FrostManager::into_key_package(share, pkg);
            let (nonces, comm) = FrostManager::commit(kp.signing_share(), &mut rng);
            commitments_map.insert(*kp.identifier(), comm);
            nonces_map.insert(*kp.identifier(), nonces);
            kps.push(kp);
        }

        let sig_pkg = SigningPackage::new(commitments_map, msg);
        let mut sig_shares = BTreeMap::new();
        for kp in &kps {
            let nonces = &nonces_map[kp.identifier()];
            let share = FrostManager::sign(&sig_pkg, nonces, kp).unwrap();
            sig_shares.insert(*kp.identifier(), share);
        }

        frost_secp256k1_tr::aggregate(&sig_pkg, &sig_shares, pkg).unwrap()
    }

    #[test]
    fn test_verify_tampered_message_fails() {
        let (shares, pkg) = do_keygen(2, 2);
        let sig = full_sign(&shares, &pkg, b"original message");
        let result = FrostManager::verify(pkg.verifying_key(), b"tampered message", &sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let (shares, pkg) = do_keygen(2, 2);
        let sig = full_sign(&shares, &pkg, b"message");
        let (_, pkg2) = do_keygen(2, 2);
        let result = FrostManager::verify(pkg2.verifying_key(), b"message", &sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_keygen_produces_unique_shares() {
        let (shares1, _) = do_keygen(3, 2);
        let (shares2, _) = do_keygen(3, 2);
        for (id, s1) in &shares1 {
            let s2 = shares2.get(id).unwrap();
            let b1 = s1.serialize().unwrap();
            let b2 = s2.serialize().unwrap();
            assert_ne!(b1, b2, "shares should differ across keygen sessions");
        }
    }

    #[test]
    fn test_commit_produces_valid_types() {
        let mut rng = OsRng;
        let (shares, pkg) = do_keygen(2, 2);
        let share = shares.values().next().unwrap();
        let kp = FrostManager::into_key_package(share, &pkg);
        let (_nonces, commitments) = FrostManager::commit(kp.signing_share(), &mut rng);
        let serialized = commitments.serialize().unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_signature_serialization_roundtrip() {
        let (shares, pkg) = do_keygen(2, 2);
        let sig = full_sign(&shares, &pkg, b"roundtrip test");
        let bytes = sig.serialize().unwrap();
        assert_eq!(bytes.len(), 64);
        // Deserialize and verify it still validates (R point y-parity
        // is not preserved across BIP-340 serialization roundtrips).
        let sig2 = Signature::deserialize(&bytes).unwrap();
        let bytes2 = sig2.serialize().unwrap();
        assert_eq!(
            bytes, bytes2,
            "serialization after deserialization should produce same bytes"
        );
        FrostManager::verify(pkg.verifying_key(), b"roundtrip test", &sig2).unwrap();
    }
}
