//! DLC: Discreet Log Contracts
//! Native Bitcoin finance primitives aligned with G-06.

use secp256k1::{PublicKey, Scalar, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a DLC Intent in the Universal Settlement Interface (USI).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DlcIntent {
    pub oracle_pubkey: Vec<u8>,
    pub collateral_sats: u64,
    pub outcome_hash: [u8; 32],
    pub expiry_block: u32,
}

impl DlcIntent {
    /// Validates internal intent invariants in a fail-closed manner.
    ///
    /// Ensures that:
    /// 1. `oracle_pubkey` is non-empty and a valid secp256k1 public key (33 or 65 bytes).
    /// 2. `collateral_sats` is strictly greater than zero.
    /// 3. `outcome_hash` is not the all-zero array (`[0u8; 32]`).
    /// 4. `expiry_block` is strictly greater than zero.
    pub fn validate(&self) -> Result<(), DlcVerificationError> {
        if self.oracle_pubkey.is_empty()
            || self.collateral_sats == 0
            || self.expiry_block == 0
            || self.outcome_hash == [0u8; 32]
        {
            return Err(DlcVerificationError::MalformedIntent);
        }

        if PublicKey::from_slice(&self.oracle_pubkey).is_err() {
            return Err(DlcVerificationError::MalformedIntent);
        }

        Ok(())
    }
}

/// Status of a DLC contract.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DlcStatus {
    Offered,
    Accepted,
    Signed,
    Executed,
    Refunded,
}

pub struct DlcManager;

/// Typed failures for DLC attestation and execution checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DlcVerificationError {
    /// The intent does not contain the minimum policy-bearing fields.
    MalformedIntent,
    /// The attestation/signature input is structurally invalid.
    MalformedAttestation,
    /// The outcome message does not match the intent commitment.
    OutcomeMismatch,
    /// The intent is no longer within its expiry block.
    Expired,
    /// The real cryptographic equation rejected the attestation.
    VerificationFailed,
    /// Existing oracle tuple inputs do not cryptographically bind the full intent.
    UnsupportedIntentBinding,
    /// The compatibility API lacks the context required to verify execution.
    UnsupportedExecutionContext,
    /// The Contract Execution Transaction (CET) structure or payout distribution is invalid.
    InvalidCetStructure,
    /// The adaptor (pre-)signature is structurally invalid or fails its check.
    InvalidAdaptorSignature,
    /// The completed CET signature is not a valid Schnorr signature for the signer.
    InvalidCetSignature,
}

impl std::fmt::Display for DlcVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedIntent => write!(f, "malformed DLC intent"),
            Self::MalformedAttestation => write!(f, "malformed DLC oracle attestation"),
            Self::OutcomeMismatch => write!(f, "DLC outcome does not match intent commitment"),
            Self::Expired => write!(f, "DLC intent has expired"),
            Self::VerificationFailed => write!(f, "DLC oracle attestation verification failed"),
            Self::UnsupportedIntentBinding => {
                write!(f, "DLC attestation is not bound to the complete intent")
            }
            Self::UnsupportedExecutionContext => {
                write!(f, "DLC execution verification context is unsupported")
            }
            Self::InvalidCetStructure => {
                write!(f, "invalid DLC CET structure or payout distribution")
            }
            Self::InvalidAdaptorSignature => {
                write!(f, "invalid DLC adaptor (pre-)signature")
            }
            Self::InvalidCetSignature => {
                write!(f, "completed DLC CET signature failed verification")
            }
        }
    }
}

impl std::error::Error for DlcVerificationError {}

/// secp256k1 group order `n`, big-endian, for hash→scalar reduction.
const SECP256K1_N: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
];

/// Reduces a 32-byte big-endian value modulo the secp256k1 order.
///
/// A 256-bit input is strictly less than `2n`, so a single conditional
/// subtraction of `n` is exact.
fn scalar_from_hash_reduced(bytes: [u8; 32]) -> Scalar {
    let mut value = bytes;
    if value >= SECP256K1_N {
        let mut borrow = 0u16;
        for i in (0..32).rev() {
            let diff = value[i] as u16 + 256 - SECP256K1_N[i] as u16 - borrow;
            value[i] = (diff & 0xff) as u8;
            borrow = if diff & 0x100 == 0 { 1 } else { 0 };
        }
    }
    Scalar::from_be_bytes(value).expect("reduced value is below the curve order")
}

/// Computes the DLC challenge scalar `e` bound to the nonce point and signer key.
///
/// Uses a domain-separated SHA-256 over the compressed nonce point, the
/// compressed public key, and the message, then reduces modulo the curve order.
fn dlc_challenge(nonce_point: &PublicKey, pubkey: &PublicKey, message: &[u8]) -> Scalar {
    let mut hasher = Sha256::new();
    hasher.update(b"DLC-ADAPTOR-V1/challenge");
    hasher.update(nonce_point.serialize());
    hasher.update(pubkey.serialize());
    hasher.update(message);
    let digest: [u8; 32] = hasher.finalize().into();
    scalar_from_hash_reduced(digest)
}

/// A real BIP340-style Schnorr adaptor (pre-)signature for a DLC Contract
/// Execution Transaction.
///
/// Construction (dlcspecs / DLC-Link point-locking): the oracle announces a
/// per-outcome nonce point `R_oracle` and key `P_oracle`. The signer derives the
/// oracle's *signature point*
/// `T = R_oracle + e_o·P_oracle`, where `e_o = H(R_oracle ‖ P_oracle ‖ m)` is the
/// challenge the oracle will sign with. The signer then picks a nonce `k`,
/// forms the adapted nonce `R_adapted = k·G + T`, and produces the adaptor
/// scalar `s' = k + e·d` with `e = H(R_adapted ‖ P ‖ m)`.
///
/// `s'` alone is **not** a valid signature. Only when the oracle reveals its
/// attestation scalar `s_oracle = k_oracle + e_o·d_oracle` can the CET signature
/// be completed as `s = s' + s_oracle`, which verifies as
/// `s·G = R_adapted + e·P` under the signer key `P`. This is the cryptographic
/// lock that makes a DLC CET un-broadcastable until the oracle attests to the
/// outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdaptorSignature {
    /// Adapted nonce point `R_adapted = k·G + T` (33-byte compressed point).
    pub adapted_nonce: Vec<u8>,
    /// The adaptor scalar `s' = k + e·d` (32 bytes, big-endian).
    pub adaptor_scalar: [u8; 32],
}

/// A completed DLC CET signature: the adapted nonce point plus the completed
/// scalar `s = s' + s_oracle`, over the message it commits to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedCetSignature {
    /// Adapted nonce point `R_adapted` (33-byte compressed point).
    pub adapted_nonce: Vec<u8>,
    /// The completed signature scalar `s = s' + s_oracle` (32 bytes, big-endian).
    pub signature_scalar: [u8; 32],
    /// The CET message that was signed.
    pub message: Vec<u8>,
}

impl DlcManager {
    /// Computes the oracle signature point `T = R_oracle + e_o·P_oracle`.
    fn oracle_signature_point(
        oracle_nonce: &[u8],
        oracle_pubkey: &[u8],
        message: &[u8],
    ) -> Result<PublicKey, DlcVerificationError> {
        let r = PublicKey::from_slice(oracle_nonce)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let p = PublicKey::from_slice(oracle_pubkey)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let e_o = dlc_challenge(&r, &p, message);
        let ep = p
            .mul_tweak(&e_o)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        r.combine(&ep)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)
    }

    /// Creates an adaptor (pre-)signature over `message` for the CET.
    ///
    /// `signer_secret`/`signer_nonce` are the signer's key and signing nonce;
    /// `oracle_nonce`/`oracle_pubkey` are the oracle's announcement. The returned
    /// signature cannot be completed without the oracle's attestation scalar.
    pub fn create_adaptor_signature(
        signer_secret: &SecretKey,
        signer_nonce: &SecretKey,
        oracle_nonce: &[u8],
        oracle_pubkey: &[u8],
        message: &[u8],
    ) -> Result<AdaptorSignature, DlcVerificationError> {
        let signer_pubkey = PublicKey::from_secret_key(signer_secret);
        let t = Self::oracle_signature_point(oracle_nonce, oracle_pubkey, message)?;
        let k_g = PublicKey::from_secret_key(signer_nonce);
        let adapted_nonce = k_g
            .combine(&t)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let e = dlc_challenge(&adapted_nonce, &signer_pubkey, message);

        // s' = k + e·d
        let ed = signer_secret
            .mul_tweak(&e)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let ed_scalar = Scalar::from_be_bytes(ed.to_secret_bytes())
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let adaptor = signer_nonce
            .add_tweak(&ed_scalar)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;

        Ok(AdaptorSignature {
            adapted_nonce: adapted_nonce.serialize().to_vec(),
            adaptor_scalar: adaptor.to_secret_bytes(),
        })
    }

    /// Verifies an adaptor (pre-)signature against the signer key and the oracle
    /// announcement, **without** the oracle's attestation.
    ///
    /// Checks `s'·G + T = R_adapted + e·P`, i.e. the adaptor is exactly one oracle
    /// scalar away from a valid signature.
    pub fn verify_adaptor_signature(
        signature: &AdaptorSignature,
        signer_pubkey: &[u8],
        oracle_nonce: &[u8],
        oracle_pubkey: &[u8],
        message: &[u8],
    ) -> Result<bool, DlcVerificationError> {
        let p = PublicKey::from_slice(signer_pubkey)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let adapted_nonce = PublicKey::from_slice(&signature.adapted_nonce)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let t = Self::oracle_signature_point(oracle_nonce, oracle_pubkey, message)?;
        let e = dlc_challenge(&adapted_nonce, &p, message);

        // lhs = s'·G + T
        let s_prime = SecretKey::from_secret_bytes(signature.adaptor_scalar)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let s_g = PublicKey::from_secret_key(&s_prime);
        let lhs = s_g
            .combine(&t)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;

        // rhs = R_adapted + e·P
        let ep = p
            .mul_tweak(&e)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let rhs = adapted_nonce
            .combine(&ep)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;

        Ok(lhs == rhs)
    }

    /// Completes an adaptor signature with the oracle's revealed attestation
    /// scalar: `s = s' + s_oracle`.
    pub fn complete_adaptor_signature(
        signature: &AdaptorSignature,
        oracle_attestation_scalar: &[u8],
        message: &[u8],
    ) -> Result<CompletedCetSignature, DlcVerificationError> {
        let s_oracle_bytes: [u8; 32] = oracle_attestation_scalar
            .try_into()
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let s_prime = SecretKey::from_secret_bytes(signature.adaptor_scalar)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let s_oracle = Scalar::from_be_bytes(s_oracle_bytes)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;
        let s = s_prime
            .add_tweak(&s_oracle)
            .map_err(|_| DlcVerificationError::InvalidAdaptorSignature)?;

        Ok(CompletedCetSignature {
            adapted_nonce: signature.adapted_nonce.clone(),
            signature_scalar: s.to_secret_bytes(),
            message: message.to_vec(),
        })
    }

    /// Verifies a completed CET signature: `s·G = R_adapted + e·P`.
    pub fn verify_completed_cet_signature(
        completed: &CompletedCetSignature,
        signer_pubkey: &[u8],
    ) -> Result<bool, DlcVerificationError> {
        let p = PublicKey::from_slice(signer_pubkey)
            .map_err(|_| DlcVerificationError::InvalidCetSignature)?;
        let adapted_nonce = PublicKey::from_slice(&completed.adapted_nonce)
            .map_err(|_| DlcVerificationError::InvalidCetSignature)?;
        let s = SecretKey::from_secret_bytes(completed.signature_scalar)
            .map_err(|_| DlcVerificationError::InvalidCetSignature)?;
        let e = dlc_challenge(&adapted_nonce, &p, &completed.message);

        let lhs = PublicKey::from_secret_key(&s);
        let ep = p
            .mul_tweak(&e)
            .map_err(|_| DlcVerificationError::InvalidCetSignature)?;
        let rhs = adapted_nonce
            .combine(&ep)
            .map_err(|_| DlcVerificationError::InvalidCetSignature)?;

        Ok(lhs == rhs)
    }
}

impl DlcManager {
    /// Creates a new DLC Intent.
    pub fn create_intent(
        oracle_pubkey: &[u8],
        collateral: u64,
        outcome: [u8; 32],
        expiry: u32,
    ) -> DlcIntent {
        DlcIntent {
            oracle_pubkey: oracle_pubkey.to_vec(),
            collateral_sats: collateral,
            outcome_hash: outcome,
            expiry_block: expiry,
        }
    }

    /// Verifies an oracle signature (attestation) for a given outcome (G-06).
    /// This implementation performs real cryptographic verification: s*G = R + H(R, m)*P.
    /// In DLCs, the oracle provides (R, s).
    pub fn verify_oracle_attestation(
        oracle_pubkey: &[u8],
        nonce_point: &[u8],
        outcome_msg: &[u8],
        signature_scalar: &[u8],
    ) -> bool {
        if oracle_pubkey.is_empty()
            || nonce_point.is_empty()
            || outcome_msg.is_empty()
            || signature_scalar.len() != 32
        {
            return false;
        }

        let pk = match PublicKey::from_slice(oracle_pubkey) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let r_point = match PublicKey::from_slice(nonce_point) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let s_bytes: [u8; 32] = match signature_scalar.try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };

        let _s_scalar = match Scalar::from_be_bytes(s_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Compute H(R, m)
        let mut hasher = Sha256::new();
        hasher.update(nonce_point);
        hasher.update(outcome_msg);
        let hash: [u8; 32] = hasher.finalize().into();
        let e = match Scalar::from_be_bytes(hash) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Right side: R + e*P
        let ep = match pk.mul_tweak(&e) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let rhs = match r_point.combine(&ep) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Left side: s*G
        let lhs = match secp256k1::SecretKey::from_secret_bytes(s_bytes) {
            Ok(sk) => PublicKey::from_secret_key(&sk),
            Err(_) => return false,
        };

        lhs == rhs
    }

    /// Verifies an oracle attestation against the intent's oracle key, outcome
    /// commitment, collateral policy, and expiry boundary.
    pub fn verify_oracle_attestation_for_intent(
        intent: &DlcIntent,
        current_block: u32,
        nonce_point: &[u8],
        outcome_msg: &[u8],
        signature_scalar: &[u8],
    ) -> Result<bool, DlcVerificationError> {
        intent.validate()?;
        if current_block > intent.expiry_block {
            return Err(DlcVerificationError::Expired);
        }
        if nonce_point.is_empty() || signature_scalar.len() != 32 || outcome_msg.is_empty() {
            return Err(DlcVerificationError::MalformedAttestation);
        }

        let outcome_hash: [u8; 32] = Sha256::digest(outcome_msg).into();
        if outcome_hash != intent.outcome_hash {
            return Err(DlcVerificationError::OutcomeMismatch);
        }

        if !Self::verify_oracle_attestation(
            &intent.oracle_pubkey,
            nonce_point,
            outcome_msg,
            signature_scalar,
        ) {
            Err(DlcVerificationError::VerificationFailed)
        } else {
            // The existing oracle tuple signs only `outcome_msg`; it does not
            // cryptographically commit to collateral, expiry, or the complete
            // intent. Do not promote the valid primitive into intent authority.
            Err(DlcVerificationError::UnsupportedIntentBinding)
        }
    }

    /// Checked compatibility API for execution verification.
    ///
    /// The legacy input contains no nonce point, outcome message, CET, expiry
    /// height, or transaction binding, so a well-shaped byte string is not
    /// enough to authorize execution. This method reports that limitation
    /// explicitly instead of accepting random signatures.
    /// Validates the structure and payout distribution of a Contract Execution Transaction (CET).
    ///
    /// Ensures that:
    /// 1. Intent collateral is greater than zero and oracle key is present.
    /// 2. Payout list is non-empty and recipient scripts are valid.
    /// 3. Total payout satoshis does not exceed the total committed collateral.
    pub fn validate_cet_structure(
        intent: &DlcIntent,
        payouts: &[(Vec<u8>, u64)],
    ) -> Result<bool, DlcVerificationError> {
        intent.validate()?;
        if payouts.is_empty() {
            return Err(DlcVerificationError::InvalidCetStructure);
        }

        let mut total_payout: u64 = 0;
        for (recipient, amount) in payouts {
            if recipient.is_empty() || *amount == 0 {
                return Err(DlcVerificationError::InvalidCetStructure);
            }
            total_payout = total_payout
                .checked_add(*amount)
                .ok_or(DlcVerificationError::InvalidCetStructure)?;
        }

        if total_payout > intent.collateral_sats {
            return Err(DlcVerificationError::InvalidCetStructure);
        }

        Ok(true)
    }

    pub fn verify_execution_checked(
        intent: &DlcIntent,
        oracle_signature: &[u8],
    ) -> Result<bool, DlcVerificationError> {
        intent.validate()?;
        if oracle_signature.len() < 32 {
            return Err(DlcVerificationError::MalformedAttestation);
        }

        Err(DlcVerificationError::UnsupportedExecutionContext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::SecretKey;

    #[test]
    fn test_dlc_intent_validate_success() {
        let valid_pk = vec![0x02; 33];
        let outcome = [0x11; 32];
        let intent = DlcManager::create_intent(&valid_pk, 100_000, outcome, 100);
        assert_eq!(intent.validate(), Ok(()));
    }

    #[test]
    fn test_dlc_intent_validate_rejections() {
        let valid_pk = vec![0x02; 33];
        let outcome = [0x11; 32];

        // Empty pubkey
        let intent = DlcManager::create_intent(&[], 100_000, outcome, 100);
        assert_eq!(
            intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );

        // Invalid pubkey bytes
        let intent = DlcManager::create_intent(&[0xff; 10], 100_000, outcome, 100);
        assert_eq!(
            intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );

        // Zero collateral
        let intent = DlcManager::create_intent(&valid_pk, 0, outcome, 100);
        assert_eq!(
            intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );

        // All-zero outcome hash
        let intent = DlcManager::create_intent(&valid_pk, 100_000, [0u8; 32], 100);
        assert_eq!(
            intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );

        // Zero expiry block
        let intent = DlcManager::create_intent(&valid_pk, 100_000, outcome, 0);
        assert_eq!(
            intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );
    }

    #[test]
    fn test_dlc_intent_creation() {
        let oracle_pk = vec![0x02; 33];
        let outcome = [0xaa; 32];
        let intent = DlcManager::create_intent(&oracle_pk, 100_000, outcome, 1000);
        assert_eq!(intent.collateral_sats, 100_000);
        assert_eq!(intent.outcome_hash, outcome);
    }

    #[test]
    fn test_oracle_attestation_verification() {
        // Oracle setup
        // Use deterministic scalars for testing
        let oracle_sk = SecretKey::from_secret_bytes([0x01; 32]).unwrap();
        let oracle_pk = PublicKey::from_secret_key(&oracle_sk);

        let nonce_sk = SecretKey::from_secret_bytes([0x02; 32]).unwrap();
        let nonce_pk = PublicKey::from_secret_key(&nonce_sk);

        let msg = b"outcome-a";

        // Compute e = H(R, m)
        let mut hasher = Sha256::new();
        hasher.update(nonce_pk.serialize());
        hasher.update(msg);
        let e_bytes: [u8; 32] = hasher.finalize().into();
        let e = Scalar::from_be_bytes(e_bytes).unwrap();

        // s = k + e*a
        // We simulate this by taking the nonce point and adding e*P
        // To get 's', we'd need to do scalar math: s = k + e*a
        // Since Scalar doesn't expose mul/add easily, we use SecretKey tweaks.
        let mut s_sk = oracle_sk;
        // s_sk = a * e
        s_sk = s_sk.mul_tweak(&e).unwrap();
        // s_sk = a*e + k
        s_sk = s_sk
            .add_tweak(&Scalar::from_be_bytes(nonce_sk.to_secret_bytes()).unwrap())
            .unwrap();

        let s_bytes = s_sk.to_secret_bytes();

        assert!(DlcManager::verify_oracle_attestation(
            &oracle_pk.serialize(),
            &nonce_pk.serialize(),
            msg,
            &s_bytes
        ));

        let outcome_hash: [u8; 32] = Sha256::digest(msg).into();
        let intent = DlcManager::create_intent(&oracle_pk.serialize(), 100_000, outcome_hash, 100);
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &intent,
                50,
                &nonce_pk.serialize(),
                msg,
                &s_bytes
            ),
            Err(DlcVerificationError::UnsupportedIntentBinding)
        );
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &intent,
                50,
                &nonce_pk.serialize(),
                b"outcome-b",
                &s_bytes
            ),
            Err(DlcVerificationError::OutcomeMismatch)
        );
        let mut mutated_signature = s_bytes;
        mutated_signature[31] ^= 1;
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &intent,
                50,
                &nonce_pk.serialize(),
                msg,
                &mutated_signature
            ),
            Err(DlcVerificationError::VerificationFailed)
        );
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &intent,
                101,
                &nonce_pk.serialize(),
                msg,
                &s_bytes
            ),
            Err(DlcVerificationError::Expired)
        );
    }

    #[test]
    fn test_validate_cet_structure() {
        let oracle_pk = vec![0x02; 33];
        let outcome = [0xaa; 32];
        let intent = DlcManager::create_intent(&oracle_pk, 100_000, outcome, 1000);

        let valid_payouts = vec![(vec![0x76, 0xa9, 0x14], 60_000), (vec![0x51, 0x20], 40_000)];
        assert_eq!(
            DlcManager::validate_cet_structure(&intent, &valid_payouts),
            Ok(true)
        );

        let empty_payouts: Vec<(Vec<u8>, u64)> = vec![];
        assert_eq!(
            DlcManager::validate_cet_structure(&intent, &empty_payouts),
            Err(DlcVerificationError::InvalidCetStructure)
        );

        let zero_amount_payouts = vec![(vec![0x51, 0x20], 0)];
        assert_eq!(
            DlcManager::validate_cet_structure(&intent, &zero_amount_payouts),
            Err(DlcVerificationError::InvalidCetStructure)
        );

        let empty_recipient_payouts = vec![(vec![], 50_000)];
        assert_eq!(
            DlcManager::validate_cet_structure(&intent, &empty_recipient_payouts),
            Err(DlcVerificationError::InvalidCetStructure)
        );

        let excessive_payouts = vec![(vec![0x51, 0x20], 100_001)];
        assert_eq!(
            DlcManager::validate_cet_structure(&intent, &excessive_payouts),
            Err(DlcVerificationError::InvalidCetStructure)
        );
    }

    #[test]
    fn test_execution_verification_rejects_random_signature() {
        let intent = DlcManager::create_intent(&[0x02; 33], 50_000, [0xbb; 32], 2_000);

        assert_eq!(
            DlcManager::verify_execution_checked(&intent, &[0x01; 32]),
            Err(DlcVerificationError::UnsupportedExecutionContext)
        );
        assert_eq!(
            DlcManager::verify_execution_checked(&intent, &[0x01; 31]),
            Err(DlcVerificationError::MalformedAttestation)
        );
    }

    #[test]
    fn test_adaptor_signature_roundtrip_completes_to_valid_cet_signature() {
        // Oracle announcement.
        let oracle_sk = SecretKey::from_secret_bytes([0x11; 32]).unwrap();
        let oracle_pk = PublicKey::from_secret_key(&oracle_sk);
        let oracle_nonce_sk = SecretKey::from_secret_bytes([0x22; 32]).unwrap();
        let oracle_nonce_pk = PublicKey::from_secret_key(&oracle_nonce_sk);

        // Signer.
        let signer_sk = SecretKey::from_secret_bytes([0x33; 32]).unwrap();
        let signer_pk = PublicKey::from_secret_key(&signer_sk);
        let signer_nonce = SecretKey::from_secret_bytes([0x44; 32]).unwrap();

        let msg = b"cet:outcome=up,payout=60000";

        let adaptor = DlcManager::create_adaptor_signature(
            &signer_sk,
            &signer_nonce,
            &oracle_nonce_pk.serialize(),
            &oracle_pk.serialize(),
            msg,
        )
        .expect("adaptor signature creation succeeds");

        // The pre-signature verifies without the oracle scalar...
        assert!(DlcManager::verify_adaptor_signature(
            &adaptor,
            &signer_pk.serialize(),
            &oracle_nonce_pk.serialize(),
            &oracle_pk.serialize(),
            msg,
        )
        .unwrap());

        // ...but is not yet a valid CET signature (it lacks the oracle scalar).
        let premature = CompletedCetSignature {
            adapted_nonce: adaptor.adapted_nonce.clone(),
            signature_scalar: adaptor.adaptor_scalar,
            message: msg.to_vec(),
        };
        assert!(
            !DlcManager::verify_completed_cet_signature(&premature, &signer_pk.serialize())
                .unwrap()
        );

        // Oracle reveals its attestation scalar s_oracle = k_o + e_o*d_o.
        let e_o = dlc_challenge(&oracle_nonce_pk, &oracle_pk, msg);
        let d_o_e = oracle_sk.mul_tweak(&e_o).unwrap();
        let e_o_scalar = Scalar::from_be_bytes(d_o_e.to_secret_bytes()).unwrap();
        let s_oracle = oracle_nonce_sk.add_tweak(&e_o_scalar).unwrap();

        let completed =
            DlcManager::complete_adaptor_signature(&adaptor, &s_oracle.to_secret_bytes(), msg)
                .expect("completion succeeds with the oracle scalar");

        assert!(
            DlcManager::verify_completed_cet_signature(&completed, &signer_pk.serialize()).unwrap()
        );

        // A wrong oracle scalar (different outcome) does not produce a valid CET signature.
        let wrong_oracle = SecretKey::from_secret_bytes([0x55; 32]).unwrap();
        let wrong =
            DlcManager::complete_adaptor_signature(&adaptor, &wrong_oracle.to_secret_bytes(), msg)
                .unwrap();
        assert!(
            !DlcManager::verify_completed_cet_signature(&wrong, &signer_pk.serialize()).unwrap()
        );
    }

    #[test]
    fn test_adaptor_signature_rejects_tampered_scalar_and_bad_inputs() {
        let oracle_sk = SecretKey::from_secret_bytes([0x11; 32]).unwrap();
        let oracle_pk = PublicKey::from_secret_key(&oracle_sk);
        let oracle_nonce_pk =
            PublicKey::from_secret_key(&SecretKey::from_secret_bytes([0x22; 32]).unwrap());
        let signer_sk = SecretKey::from_secret_bytes([0x33; 32]).unwrap();
        let signer_pk = PublicKey::from_secret_key(&signer_sk);
        let signer_nonce = SecretKey::from_secret_bytes([0x44; 32]).unwrap();
        let msg = b"cet:outcome=down,payout=40000";

        let mut adaptor = DlcManager::create_adaptor_signature(
            &signer_sk,
            &signer_nonce,
            &oracle_nonce_pk.serialize(),
            &oracle_pk.serialize(),
            msg,
        )
        .unwrap();

        // Tampering the scalar breaks the adaptor check.
        adaptor.adaptor_scalar[31] ^= 1;
        assert!(!DlcManager::verify_adaptor_signature(
            &adaptor,
            &signer_pk.serialize(),
            &oracle_nonce_pk.serialize(),
            &oracle_pk.serialize(),
            msg,
        )
        .unwrap());

        // A different message breaks it too.
        assert!(!DlcManager::verify_adaptor_signature(
            &adaptor,
            &signer_pk.serialize(),
            &oracle_nonce_pk.serialize(),
            &oracle_pk.serialize(),
            b"other",
        )
        .unwrap());

        // Malformed oracle nonce is rejected.
        assert_eq!(
            DlcManager::create_adaptor_signature(
                &signer_sk,
                &signer_nonce,
                &[0xff; 3],
                &oracle_pk.serialize(),
                msg,
            ),
            Err(DlcVerificationError::InvalidAdaptorSignature)
        );
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_dlc_intent_malformed_and_edge_case_validation() {
        let valid_pk = vec![0x02; 33];
        let outcome = [0x11; 32];
        let valid_intent = DlcManager::create_intent(&valid_pk, 100_000, outcome, 100);

        // Malformed intents: empty oracle_pubkey, 0 collateral, 0 expiry
        let empty_pk_intent = DlcManager::create_intent(&[], 100_000, outcome, 100);
        let zero_collateral_intent = DlcManager::create_intent(&valid_pk, 0, outcome, 100);
        let zero_expiry_intent = DlcManager::create_intent(&valid_pk, 100_000, outcome, 0);

        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &empty_pk_intent,
                10,
                &[0x02; 33],
                b"msg",
                &[0x01; 32]
            ),
            Err(DlcVerificationError::MalformedIntent)
        );
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &zero_collateral_intent,
                10,
                &[0x02; 33],
                b"msg",
                &[0x01; 32]
            ),
            Err(DlcVerificationError::MalformedIntent)
        );
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &zero_expiry_intent,
                10,
                &[0x02; 33],
                b"msg",
                &[0x01; 32]
            ),
            Err(DlcVerificationError::MalformedIntent)
        );

        // Malformed attestations: empty nonce point, invalid signature length
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &valid_intent,
                10,
                &[],
                b"msg",
                &[0x01; 32]
            ),
            Err(DlcVerificationError::MalformedAttestation)
        );
        assert_eq!(
            DlcManager::verify_oracle_attestation_for_intent(
                &valid_intent,
                10,
                &[0x02; 33],
                b"msg",
                &[0x01; 31]
            ),
            Err(DlcVerificationError::MalformedAttestation)
        );

        // Direct verify_oracle_attestation with invalid bytes
        assert!(!DlcManager::verify_oracle_attestation(
            &[0xff; 10], // invalid pubkey
            &[0x02; 33],
            b"msg",
            &[0x01; 32]
        ));
        assert!(!DlcManager::verify_oracle_attestation(
            &[0x02; 33],
            &[0xff; 10], // invalid nonce point
            b"msg",
            &[0x01; 32]
        ));
        assert!(!DlcManager::verify_oracle_attestation(
            &[0x02; 33],
            &[0x02; 33],
            b"msg",
            &[0x01; 10] // invalid sig scalar len
        ));

        // verify_execution_checked malformed intent check
        assert_eq!(
            DlcManager::verify_execution_checked(&empty_pk_intent, &[0x01; 32]),
            Err(DlcVerificationError::MalformedIntent)
        );

        // DlcIntent::validate test
        assert_eq!(valid_intent.validate(), Ok(()));
        assert_eq!(
            empty_pk_intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );
        let zero_outcome_intent = DlcManager::create_intent(&valid_pk, 100_000, [0u8; 32], 100);
        assert_eq!(
            zero_outcome_intent.validate(),
            Err(DlcVerificationError::MalformedIntent)
        );

        // Display formatting check
        assert_eq!(
            DlcVerificationError::MalformedIntent.to_string(),
            "malformed DLC intent"
        );
        assert_eq!(
            DlcVerificationError::MalformedAttestation.to_string(),
            "malformed DLC oracle attestation"
        );
        assert_eq!(
            DlcVerificationError::OutcomeMismatch.to_string(),
            "DLC outcome does not match intent commitment"
        );
        assert_eq!(
            DlcVerificationError::Expired.to_string(),
            "DLC intent has expired"
        );
        assert_eq!(
            DlcVerificationError::VerificationFailed.to_string(),
            "DLC oracle attestation verification failed"
        );
        assert_eq!(
            DlcVerificationError::UnsupportedIntentBinding.to_string(),
            "DLC attestation is not bound to the complete intent"
        );
        assert_eq!(
            DlcVerificationError::UnsupportedExecutionContext.to_string(),
            "DLC execution verification context is unsupported"
        );
    }
}
