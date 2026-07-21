//! DLC: Discreet Log Contracts.
//!
//! The oracle attestation primitive below performs the real secp256k1 point
//! equation check. The legacy shallow execution entry point is retained only
//! as a typed unsupported boundary because its arguments cannot bind the
//! outcome message, nonce point, expiry, and oracle key together.

use secp256k1::{PublicKey, Scalar, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Typed failures returned by DLC verification.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DlcError {
    InvalidIntent {
        reason: String,
    },
    InvalidOraclePublicKey,
    InvalidNoncePoint,
    InvalidSignatureScalar,
    OutcomeMismatch,
    InvalidAttestation,
    Expired {
        expiry_block: u32,
        current_block: u32,
    },
    UnsupportedExecutionVerification,
}

impl fmt::Display for DlcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIntent { reason } => write!(f, "invalid DLC intent: {reason}"),
            Self::InvalidOraclePublicKey => write!(f, "invalid DLC oracle public key"),
            Self::InvalidNoncePoint => write!(f, "invalid DLC oracle nonce point"),
            Self::InvalidSignatureScalar => write!(f, "invalid DLC signature scalar"),
            Self::OutcomeMismatch => write!(f, "DLC outcome does not match intent"),
            Self::InvalidAttestation => write!(f, "invalid DLC oracle attestation"),
            Self::Expired {
                expiry_block,
                current_block,
            } => write!(
                f,
                "DLC intent expired at block {expiry_block}; current block is {current_block}"
            ),
            Self::UnsupportedExecutionVerification => {
                write!(f, "shallow DLC execution verification is unsupported")
            }
        }
    }
}

impl std::error::Error for DlcError {}

/// Represents a DLC Intent in the Universal Settlement Interface (USI).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DlcIntent {
    pub oracle_pubkey: Vec<u8>,
    pub collateral_sats: u64,
    pub outcome_hash: [u8; 32],
    pub expiry_block: u32,
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

    /// Compatibility wrapper for callers that only accept a boolean.
    /// Use [`Self::try_verify_oracle_attestation`] for typed failure details.
    pub fn verify_oracle_attestation(
        oracle_pubkey: &[u8],
        nonce_point: &[u8],
        outcome_msg: &[u8],
        signature_scalar: &[u8],
    ) -> bool {
        Self::try_verify_oracle_attestation(
            oracle_pubkey,
            nonce_point,
            outcome_msg,
            signature_scalar,
        )
        .is_ok()
    }

    /// Verifies the DLC oracle equation `s*G = R + H(R, m)*P`.
    pub fn try_verify_oracle_attestation(
        oracle_pubkey: &[u8],
        nonce_point: &[u8],
        outcome_msg: &[u8],
        signature_scalar: &[u8],
    ) -> Result<(), DlcError> {
        let secp = Secp256k1::new();
        let pk =
            PublicKey::from_slice(oracle_pubkey).map_err(|_| DlcError::InvalidOraclePublicKey)?;
        let r_point =
            PublicKey::from_slice(nonce_point).map_err(|_| DlcError::InvalidNoncePoint)?;
        let s_bytes: [u8; 32] = signature_scalar
            .try_into()
            .map_err(|_| DlcError::InvalidSignatureScalar)?;
        let s = Scalar::from_be_bytes(s_bytes).map_err(|_| DlcError::InvalidSignatureScalar)?;

        let mut hasher = Sha256::new();
        hasher.update(nonce_point);
        hasher.update(outcome_msg);
        let e_bytes: [u8; 32] = hasher.finalize().into();
        let e = Scalar::from_be_bytes(e_bytes).map_err(|_| DlcError::InvalidAttestation)?;

        let ep = pk
            .mul_tweak(&secp, &e)
            .map_err(|_| DlcError::InvalidAttestation)?;
        let rhs = r_point
            .combine(&ep)
            .map_err(|_| DlcError::InvalidAttestation)?;
        let lhs_secret = secp256k1::SecretKey::from_byte_array(s.to_be_bytes())
            .map_err(|_| DlcError::InvalidSignatureScalar)?;
        let lhs = PublicKey::from_secret_key(&secp, &lhs_secret);

        if lhs == rhs {
            Ok(())
        } else {
            Err(DlcError::InvalidAttestation)
        }
    }

    /// Verifies a fully bound oracle-backed execution.
    ///
    /// This helper binds the outcome message to the intent hash, checks
    /// collateral and expiry, and then verifies the real oracle equation. It
    /// intentionally does not claim to verify funding/CET transactions or
    /// Bitcoin finality; those remain downstream responsibilities.
    pub fn verify_execution_attestation(
        intent: &DlcIntent,
        nonce_point: &[u8],
        outcome_msg: &[u8],
        signature_scalar: &[u8],
        current_block: u32,
    ) -> Result<(), DlcError> {
        if intent.collateral_sats == 0 {
            return Err(DlcError::InvalidIntent {
                reason: "collateral must be greater than zero".to_string(),
            });
        }
        if current_block >= intent.expiry_block {
            return Err(DlcError::Expired {
                expiry_block: intent.expiry_block,
                current_block,
            });
        }
        let outcome_hash: [u8; 32] = Sha256::digest(outcome_msg).into();
        if outcome_hash != intent.outcome_hash {
            return Err(DlcError::OutcomeMismatch);
        }

        Self::try_verify_oracle_attestation(
            &intent.oracle_pubkey,
            nonce_point,
            outcome_msg,
            signature_scalar,
        )
    }

    /// Legacy shallow execution verification is intentionally unsupported.
    /// Its arguments cannot bind the required nonce, outcome message, oracle
    /// key, or expiry context, so it must never return success.
    pub fn verify_execution(
        _intent: &DlcIntent,
        _oracle_signature: &[u8],
    ) -> Result<bool, DlcError> {
        Err(DlcError::UnsupportedExecutionVerification)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::SecretKey;

    fn signed_outcome() -> (SecretKey, PublicKey, PublicKey, Vec<u8>, [u8; 32], [u8; 32]) {
        let secp = Secp256k1::new();
        let oracle_sk = SecretKey::from_byte_array([0x01; 32]).unwrap();
        let oracle_pk = PublicKey::from_secret_key(&secp, &oracle_sk);
        let nonce_sk = SecretKey::from_byte_array([0x02; 32]).unwrap();
        let nonce_pk = PublicKey::from_secret_key(&secp, &nonce_sk);
        let msg = b"outcome-a".to_vec();

        let mut hasher = Sha256::new();
        hasher.update(nonce_pk.serialize());
        hasher.update(&msg);
        let e_bytes: [u8; 32] = hasher.finalize().into();
        let e = Scalar::from_be_bytes(e_bytes).unwrap();
        let mut s_sk = oracle_sk;
        s_sk = s_sk.mul_tweak(&e).unwrap();
        s_sk = s_sk
            .add_tweak(&Scalar::from_be_bytes(nonce_sk.secret_bytes()).unwrap())
            .unwrap();

        let outcome_hash: [u8; 32] = Sha256::digest(&msg).into();
        (
            oracle_sk,
            oracle_pk,
            nonce_pk,
            s_sk.secret_bytes().to_vec(),
            outcome_hash,
            e_bytes,
        )
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
    fn test_oracle_attestation_verification_and_mutations() {
        let (_, oracle_pk, nonce_pk, signature, _, _) = signed_outcome();
        let msg = b"outcome-a";

        assert!(DlcManager::try_verify_oracle_attestation(
            &oracle_pk.serialize(),
            &nonce_pk.serialize(),
            msg,
            &signature
        )
        .is_ok());

        let mut mutated_signature = signature.clone();
        mutated_signature[31] ^= 1;
        assert!(matches!(
            DlcManager::try_verify_oracle_attestation(
                &oracle_pk.serialize(),
                &nonce_pk.serialize(),
                msg,
                &mutated_signature,
            ),
            Err(DlcError::InvalidAttestation | DlcError::InvalidSignatureScalar)
        ));
        assert!(matches!(
            DlcManager::try_verify_oracle_attestation(
                &oracle_pk.serialize(),
                &nonce_pk.serialize(),
                b"wrong-outcome",
                &signature,
            ),
            Err(DlcError::InvalidAttestation)
        ));

        let mut mutated_nonce = nonce_pk.serialize().to_vec();
        mutated_nonce[10] ^= 1;
        assert!(DlcManager::try_verify_oracle_attestation(
            &oracle_pk.serialize(),
            &mutated_nonce,
            msg,
            &signature,
        )
        .is_err());
    }

    #[test]
    fn test_dlc_execution_binds_outcome_and_expiry() {
        let (_, oracle_pk, nonce_pk, signature, outcome_hash, _) = signed_outcome();
        let intent = DlcManager::create_intent(&oracle_pk.serialize(), 50_000, outcome_hash, 2000);

        assert!(DlcManager::verify_execution_attestation(
            &intent,
            &nonce_pk.serialize(),
            b"outcome-a",
            &signature,
            1999,
        )
        .is_ok());

        assert_eq!(
            DlcManager::verify_execution(&intent, &signature),
            Err(DlcError::UnsupportedExecutionVerification)
        );

        let wrong_intent =
            DlcManager::create_intent(&oracle_pk.serialize(), 50_000, [0xaa; 32], 2000);
        assert_eq!(
            DlcManager::verify_execution_attestation(
                &wrong_intent,
                &nonce_pk.serialize(),
                b"outcome-a",
                &signature,
                1999,
            ),
            Err(DlcError::OutcomeMismatch)
        );

        let expired = DlcManager::create_intent(&oracle_pk.serialize(), 50_000, outcome_hash, 2000);
        assert_eq!(
            DlcManager::verify_execution_attestation(
                &expired,
                &nonce_pk.serialize(),
                b"outcome-a",
                &signature,
                2000,
            ),
            Err(DlcError::Expired {
                expiry_block: 2000,
                current_block: 2000,
            })
        );
    }
}
