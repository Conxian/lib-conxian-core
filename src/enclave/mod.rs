//! Enclave Architecture and Zero-Knowledge Compliance (ZKC)
//! Aligned with CXIP 20 Section 2.0 and CON-1329

use der::{Any, Decode, Tag, Tagged};
use serde::{Deserialize, Serialize};

/// Represents an X.509 DER-encoded certificate for hardware attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationCertificate {
    pub raw_der: Vec<u8>,
}

/// Typed failures for Core's enclave-verification boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnclaveVerificationError {
    /// Evidence is absent.
    EmptyEvidence,
    /// Evidence cannot be parsed as the expected DER container.
    MalformedDer(String),
    /// A provider-backed verifier must run in the enclave SDK/downstream.
    UnsupportedProvider,
}

impl std::fmt::Display for EnclaveVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyEvidence => write!(f, "enclave verification evidence is empty"),
            Self::MalformedDer(reason) => write!(f, "malformed attestation DER: {reason}"),
            Self::UnsupportedProvider => write!(
                f,
                "provider-backed enclave verification is unsupported in Core"
            ),
        }
    }
}

impl std::error::Error for EnclaveVerificationError {}

pub struct HeadlessEnclave;

impl HeadlessEnclave {
    pub fn execute_stateless<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // In a real SGX environment, this would be an ecall
        f()
    }

    /// Parses the DER container but does not claim certificate-chain
    /// authenticity. Provider-backed attestation verification belongs in the
    /// production enclave SDK.
    pub fn verify_attestation_chain(
        cert: &AttestationCertificate,
    ) -> Result<bool, EnclaveVerificationError> {
        if cert.raw_der.is_empty() {
            return Err(EnclaveVerificationError::EmptyEvidence);
        }

        let any = Any::from_der(&cert.raw_der)
            .map_err(|e| EnclaveVerificationError::MalformedDer(e.to_string()))?;

        if any.tag() != Tag::Sequence {
            return Err(EnclaveVerificationError::MalformedDer(
                "expected SEQUENCE".to_string(),
            ));
        }

        Err(EnclaveVerificationError::UnsupportedProvider)
    }
}

pub struct ZKCompliance;

impl ZKCompliance {
    /// Checks input presence and reports that AML proof verification must be
    /// supplied by an audited downstream/provider implementation.
    pub fn verify_aml_stateless_checked(
        identity_commitment: &str,
        tx_metadata: &str,
    ) -> Result<bool, EnclaveVerificationError> {
        if identity_commitment.trim().is_empty() || tx_metadata.trim().is_empty() {
            return Err(EnclaveVerificationError::EmptyEvidence);
        }
        Err(EnclaveVerificationError::UnsupportedProvider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_attestation_chain_parse_only_is_unsupported() {
        let cert = AttestationCertificate {
            raw_der: vec![0x30, 0x00],
        };
        assert_eq!(
            HeadlessEnclave::verify_attestation_chain(&cert),
            Err(EnclaveVerificationError::UnsupportedProvider)
        );
    }

    #[test]
    fn test_verify_attestation_chain_invalid_der() {
        let cert = AttestationCertificate {
            raw_der: vec![0xff, 0xff],
        };
        assert!(matches!(
            HeadlessEnclave::verify_attestation_chain(&cert),
            Err(EnclaveVerificationError::MalformedDer(_))
        ));
    }

    #[test]
    fn test_verify_attestation_chain_truncated() {
        // SEQUENCE tag with length 5 but only 1 byte follows
        let cert = AttestationCertificate {
            raw_der: vec![0x30, 0x05, 0x01],
        };
        assert!(matches!(
            HeadlessEnclave::verify_attestation_chain(&cert),
            Err(EnclaveVerificationError::MalformedDer(_))
        ));
    }

    #[test]
    fn test_aml_non_empty_input_is_unsupported() {
        assert_eq!(
            ZKCompliance::verify_aml_stateless_checked("id", "metadata"),
            Err(EnclaveVerificationError::UnsupportedProvider)
        );
        assert_eq!(
            ZKCompliance::verify_aml_stateless_checked("", "metadata"),
            Err(EnclaveVerificationError::EmptyEvidence)
        );
    }
}
