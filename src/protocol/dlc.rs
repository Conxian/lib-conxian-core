//! DLC: Discreet Log Contracts
//! Native Bitcoin finance primitives aligned with G-06.

use secp256k1::{PublicKey, Scalar, Secp256k1};
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
        }
    }
}

impl std::error::Error for DlcVerificationError {}

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

        let secp = Secp256k1::new();

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
        let ep = match pk.mul_tweak(&secp, &e) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let rhs = match r_point.combine(&ep) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Left side: s*G
        let lhs = match secp256k1::SecretKey::from_byte_array(s_bytes) {
            Ok(sk) => PublicKey::from_secret_key(&secp, &sk),
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
        if intent.oracle_pubkey.is_empty()
            || intent.collateral_sats == 0
            || intent.expiry_block == 0
        {
            return Err(DlcVerificationError::MalformedIntent);
        }
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
        if intent.collateral_sats == 0 || intent.oracle_pubkey.is_empty() {
            return Err(DlcVerificationError::MalformedIntent);
        }
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
        if intent.oracle_pubkey.is_empty()
            || intent.collateral_sats == 0
            || intent.expiry_block == 0
        {
            return Err(DlcVerificationError::MalformedIntent);
        }
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
    fn test_dlc_intent_creation() {
        let oracle_pk = vec![0x02; 33];
        let outcome = [0xaa; 32];
        let intent = DlcManager::create_intent(&oracle_pk, 100_000, outcome, 1000);
        assert_eq!(intent.collateral_sats, 100_000);
        assert_eq!(intent.outcome_hash, outcome);
    }

    #[test]
    fn test_oracle_attestation_verification() {
        let secp = Secp256k1::new();

        // Oracle setup
        // Use deterministic scalars for testing
        let oracle_sk = SecretKey::from_byte_array([0x01; 32]).unwrap();
        let oracle_pk = PublicKey::from_secret_key(&secp, &oracle_sk);

        let nonce_sk = SecretKey::from_byte_array([0x02; 32]).unwrap();
        let nonce_pk = PublicKey::from_secret_key(&secp, &nonce_sk);

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
            .add_tweak(&Scalar::from_be_bytes(nonce_sk.secret_bytes()).unwrap())
            .unwrap();

        let s_bytes = s_sk.secret_bytes();

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
