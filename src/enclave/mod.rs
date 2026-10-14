//! Enclave Architecture, Hardware Chipset Drivers, and Zero-Knowledge Compliance (ZKC)
//! Aligned with CXIP 20 Section 2.0, CON-1329, and CON-1336 (Hardware Chipset Driver & Enclave Boundary Expansion)

use der::{Any, Decode, Tag, Tagged};
use serde::{Deserialize, Serialize};

/// Supported hardware chipsets and enclave execution environments across cloud,
/// mobile, desktop, server, and specialized hardware signers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareChipset {
    /// AWS Nitro Enclaves (Annapurna Labs PCIe ASICs, `/dev/nitro_enclave`)
    AwsNitro,
    /// Android KeyMint / StrongBox (Titan M2 / Knox SE / TrustZone)
    AndroidStrongBox,
    /// Intel SGX Architectural Enclaves (`/dev/sgx_enclave`, DCAP)
    IntelSgx,
    /// Intel Trust Domain Extensions (`/dev/tdx_guest`, DCAP Quote)
    IntelTdx,
    /// AMD SEV-SNP Confidential VMs (`/dev/sev-guest`, VCEK/ASK/ARK)
    AmdSevSnp,
    /// Apple Secure Enclave Processor (A/M-series SoCs, AppAttest)
    AppleSecureEnclave,
    /// PKCS#11 Hardware Security Modules (YubiHSM 2, Thales Luna, AWS CloudHSM)
    Pkcs11Hsm,
    /// RISC-V Keystone Enclaves (`/dev/keystone_enclave`, Security Monitor)
    RiscVKeystone,
    /// Discrete & Firmware TPM 2.0 (`/dev/tpmrm0`, TSS2)
    Tpm20,
    /// ARM TrustZone / OP-TEE TEE OS (`/dev/tee0`, SMC interface)
    ArmTrustZone,
    /// Hardware Wallets & Secure Elements (Keystone Pro/3, Ledger ST33, Trezor)
    HardwareWallet,
}

impl HardwareChipset {
    /// Returns the canonical device or driver endpoint prefix expected for this chipset.
    pub fn default_device_node_prefix(&self) -> &'static str {
        match self {
            Self::AwsNitro => "/dev/nitro_enclave",
            Self::AndroidStrongBox => "hal:android.hardware.security.keymint",
            Self::IntelSgx => "/dev/sgx_enclave",
            Self::IntelTdx => "/dev/tdx_guest",
            Self::AmdSevSnp => "/dev/sev-guest",
            Self::AppleSecureEnclave => "sys:apple.framework.localauth",
            Self::Pkcs11Hsm => "/usr/lib/libpkcs11",
            Self::RiscVKeystone => "/dev/keystone_enclave",
            Self::Tpm20 => "/dev/tpmrm0",
            Self::ArmTrustZone => "/dev/tee0",
            Self::HardwareWallet => "/dev/hidraw",
        }
    }
}

/// Hardware attestation evidence structure formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationType {
    NitroCoseSign1,
    KeyMintX509Chain,
    IntelDcapQuote,
    AmdSnpReport,
    AppleAppAttest,
    Pkcs11SlotAttestation,
    KeystoneReport,
    Tpm2Quote,
    OpTeeAttestation,
    SecureElementCert,
}

impl AttestationType {
    /// Validates whether this attestation format is valid for the given hardware chipset.
    pub fn is_compatible_with(&self, chipset: HardwareChipset) -> bool {
        matches!(
            (self, chipset),
            (Self::NitroCoseSign1, HardwareChipset::AwsNitro)
                | (Self::KeyMintX509Chain, HardwareChipset::AndroidStrongBox)
                | (Self::IntelDcapQuote, HardwareChipset::IntelSgx)
                | (Self::IntelDcapQuote, HardwareChipset::IntelTdx)
                | (Self::AmdSnpReport, HardwareChipset::AmdSevSnp)
                | (Self::AppleAppAttest, HardwareChipset::AppleSecureEnclave)
                | (Self::Pkcs11SlotAttestation, HardwareChipset::Pkcs11Hsm)
                | (Self::KeystoneReport, HardwareChipset::RiscVKeystone)
                | (Self::Tpm2Quote, HardwareChipset::Tpm20)
                | (Self::OpTeeAttestation, HardwareChipset::ArmTrustZone)
                | (Self::SecureElementCert, HardwareChipset::HardwareWallet)
        )
    }
}

/// Metadata describing kernel driver, HAL interface, or device node for a chipset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChipsetDriverInfo {
    pub chipset: HardwareChipset,
    pub device_node: String,
    pub driver_version: String,
    pub vendor_id: String,
}

impl ChipsetDriverInfo {
    /// Fail-closed validation of driver information.
    pub fn validate(&self) -> Result<(), EnclaveVerificationError> {
        if self.device_node.trim().is_empty() {
            return Err(EnclaveVerificationError::InvalidDriverInfo(
                "device_node path cannot be empty".to_string(),
            ));
        }
        if self.driver_version.trim().is_empty() {
            return Err(EnclaveVerificationError::InvalidDriverInfo(
                "driver_version cannot be empty".to_string(),
            ));
        }
        if self.vendor_id.trim().is_empty() {
            return Err(EnclaveVerificationError::InvalidDriverInfo(
                "vendor_id cannot be empty".to_string(),
            ));
        }
        let dev = self.device_node.trim();
        if !dev.starts_with("/dev/")
            && !dev.starts_with("hal:")
            && !dev.starts_with("sys:")
            && !dev.starts_with("/usr/lib")
        {
            return Err(EnclaveVerificationError::InvalidDriverInfo(format!(
                "device_node '{dev}' does not match expected system path prefix (/dev/, hal:, sys:, /usr/lib)"
            )));
        }
        Ok(())
    }
}

/// Complete hardware attestation evidence container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareAttestationEvidence {
    pub chipset: HardwareChipset,
    pub attestation_type: AttestationType,
    pub raw_payload: Vec<u8>,
    pub challenge_nonce: Vec<u8>,
    pub measurement_digest: Vec<u8>,
    pub driver_info: Option<ChipsetDriverInfo>,
}

impl HardwareAttestationEvidence {
    /// Validates structural integrity, challenge nonces, measurement bounds,
    /// and chipset-attestation pairing. Fail-closed.
    pub fn validate(&self) -> Result<(), EnclaveVerificationError> {
        if !self.attestation_type.is_compatible_with(self.chipset) {
            return Err(EnclaveVerificationError::MismatchedChipsetAttestation {
                expected: format!("{:?}", self.chipset),
                found: format!("{:?}", self.attestation_type),
            });
        }
        if self.raw_payload.is_empty() {
            return Err(EnclaveVerificationError::EmptyEvidence);
        }
        if self.challenge_nonce.len() != 32 || self.challenge_nonce.iter().all(|&b| b == 0) {
            return Err(EnclaveVerificationError::InvalidChipsetEvidence(
                "challenge_nonce must be exactly 32 non-zero bytes".to_string(),
            ));
        }
        if self.measurement_digest.len() < 32 {
            return Err(EnclaveVerificationError::InvalidChipsetEvidence(
                "measurement_digest must be at least 32 bytes".to_string(),
            ));
        }
        if let Some(ref info) = self.driver_info {
            if info.chipset != self.chipset {
                return Err(EnclaveVerificationError::InvalidDriverInfo(format!(
                    "driver_info chipset '{:?}' does not match evidence chipset '{:?}'",
                    info.chipset, self.chipset
                )));
            }
            info.validate()?;
        }
        Ok(())
    }
}

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
    /// Evidence is invalid for the specified hardware chipset or attestation type.
    InvalidChipsetEvidence(String),
    /// Chipset driver info failed validation.
    InvalidDriverInfo(String),
    /// Attestation type is incompatible with the target hardware chipset.
    MismatchedChipsetAttestation { expected: String, found: String },
    /// A provider-backed verifier must run in the enclave SDK/downstream.
    UnsupportedProvider,
}

impl std::fmt::Display for EnclaveVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyEvidence => write!(f, "enclave verification evidence is empty"),
            Self::MalformedDer(reason) => write!(f, "malformed attestation DER: {reason}"),
            Self::InvalidChipsetEvidence(reason) => write!(f, "invalid chipset evidence: {reason}"),
            Self::InvalidDriverInfo(reason) => write!(f, "invalid driver info: {reason}"),
            Self::MismatchedChipsetAttestation { expected, found } => write!(
                f,
                "attestation type '{found}' is incompatible with chipset '{expected}'"
            ),
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

    /// Structurally validates hardware attestation evidence, verifying challenge nonces,
    /// measurement digests, and driver info. Returns `UnsupportedProvider` for
    /// cryptographic root chain verification, which belongs in `conxius-enclave-sdk`.
    pub fn verify_hardware_attestation_evidence(
        evidence: &HardwareAttestationEvidence,
    ) -> Result<bool, EnclaveVerificationError> {
        evidence.validate()?;
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

    #[test]
    fn test_hardware_chipset_default_prefixes() {
        assert_eq!(
            HardwareChipset::AwsNitro.default_device_node_prefix(),
            "/dev/nitro_enclave"
        );
        assert_eq!(
            HardwareChipset::AndroidStrongBox.default_device_node_prefix(),
            "hal:android.hardware.security.keymint"
        );
        assert_eq!(
            HardwareChipset::IntelSgx.default_device_node_prefix(),
            "/dev/sgx_enclave"
        );
        assert_eq!(
            HardwareChipset::IntelTdx.default_device_node_prefix(),
            "/dev/tdx_guest"
        );
        assert_eq!(
            HardwareChipset::AmdSevSnp.default_device_node_prefix(),
            "/dev/sev-guest"
        );
        assert_eq!(
            HardwareChipset::AppleSecureEnclave.default_device_node_prefix(),
            "sys:apple.framework.localauth"
        );
        assert_eq!(
            HardwareChipset::Pkcs11Hsm.default_device_node_prefix(),
            "/usr/lib/libpkcs11"
        );
        assert_eq!(
            HardwareChipset::RiscVKeystone.default_device_node_prefix(),
            "/dev/keystone_enclave"
        );
        assert_eq!(
            HardwareChipset::Tpm20.default_device_node_prefix(),
            "/dev/tpmrm0"
        );
        assert_eq!(
            HardwareChipset::ArmTrustZone.default_device_node_prefix(),
            "/dev/tee0"
        );
        assert_eq!(
            HardwareChipset::HardwareWallet.default_device_node_prefix(),
            "/dev/hidraw"
        );
    }

    #[test]
    fn test_attestation_type_compatibility() {
        assert!(AttestationType::NitroCoseSign1.is_compatible_with(HardwareChipset::AwsNitro));
        assert!(
            AttestationType::KeyMintX509Chain.is_compatible_with(HardwareChipset::AndroidStrongBox)
        );
        assert!(AttestationType::IntelDcapQuote.is_compatible_with(HardwareChipset::IntelSgx));
        assert!(AttestationType::IntelDcapQuote.is_compatible_with(HardwareChipset::IntelTdx));
        assert!(AttestationType::AmdSnpReport.is_compatible_with(HardwareChipset::AmdSevSnp));
        assert!(
            AttestationType::AppleAppAttest.is_compatible_with(HardwareChipset::AppleSecureEnclave)
        );
        assert!(
            AttestationType::Pkcs11SlotAttestation.is_compatible_with(HardwareChipset::Pkcs11Hsm)
        );
        assert!(AttestationType::KeystoneReport.is_compatible_with(HardwareChipset::RiscVKeystone));
        assert!(AttestationType::Tpm2Quote.is_compatible_with(HardwareChipset::Tpm20));
        assert!(AttestationType::OpTeeAttestation.is_compatible_with(HardwareChipset::ArmTrustZone));
        assert!(
            AttestationType::SecureElementCert.is_compatible_with(HardwareChipset::HardwareWallet)
        );

        // Mismatched check
        assert!(!AttestationType::NitroCoseSign1.is_compatible_with(HardwareChipset::IntelSgx));
    }

    #[test]
    fn test_chipset_driver_info_validation() {
        let valid_info = ChipsetDriverInfo {
            chipset: HardwareChipset::AwsNitro,
            device_node: "/dev/nitro_enclave".to_string(),
            driver_version: "1.2.0".to_string(),
            vendor_id: "aws".to_string(),
        };
        assert!(valid_info.validate().is_ok());

        let empty_node = ChipsetDriverInfo {
            device_node: "  ".to_string(),
            ..valid_info.clone()
        };
        assert!(matches!(
            empty_node.validate(),
            Err(EnclaveVerificationError::InvalidDriverInfo(_))
        ));

        let invalid_path = ChipsetDriverInfo {
            device_node: "invalid/path".to_string(),
            ..valid_info.clone()
        };
        assert!(matches!(
            invalid_path.validate(),
            Err(EnclaveVerificationError::InvalidDriverInfo(_))
        ));
    }

    #[test]
    fn test_hardware_attestation_evidence_validation() {
        let valid_nonce = vec![0x42; 32];
        let valid_digest = vec![0xab; 32];
        let driver_info = ChipsetDriverInfo {
            chipset: HardwareChipset::AwsNitro,
            device_node: "/dev/nitro_enclave".to_string(),
            driver_version: "1.2.0".to_string(),
            vendor_id: "aws".to_string(),
        };

        let evidence = HardwareAttestationEvidence {
            chipset: HardwareChipset::AwsNitro,
            attestation_type: AttestationType::NitroCoseSign1,
            raw_payload: vec![0x01, 0x02, 0x03],
            challenge_nonce: valid_nonce.clone(),
            measurement_digest: valid_digest.clone(),
            driver_info: Some(driver_info),
        };

        // Structural validation succeeds, but verify_hardware_attestation_evidence returns UnsupportedProvider (fail closed for crypto verification)
        assert_eq!(
            HeadlessEnclave::verify_hardware_attestation_evidence(&evidence),
            Err(EnclaveVerificationError::UnsupportedProvider)
        );

        // Mismatched attestation type
        let mismatched = HardwareAttestationEvidence {
            attestation_type: AttestationType::IntelDcapQuote,
            ..evidence.clone()
        };
        assert!(matches!(
            HeadlessEnclave::verify_hardware_attestation_evidence(&mismatched),
            Err(EnclaveVerificationError::MismatchedChipsetAttestation { .. })
        ));

        // Zero nonce
        let zero_nonce = HardwareAttestationEvidence {
            challenge_nonce: vec![0; 32],
            ..evidence.clone()
        };
        assert!(matches!(
            HeadlessEnclave::verify_hardware_attestation_evidence(&zero_nonce),
            Err(EnclaveVerificationError::InvalidChipsetEvidence(_))
        ));

        // Short digest
        let short_digest = HardwareAttestationEvidence {
            measurement_digest: vec![0x01; 16],
            ..evidence.clone()
        };
        assert!(matches!(
            HeadlessEnclave::verify_hardware_attestation_evidence(&short_digest),
            Err(EnclaveVerificationError::InvalidChipsetEvidence(_))
        ));

        // Mismatched driver info chipset
        let bad_driver_chipset = HardwareAttestationEvidence {
            driver_info: Some(ChipsetDriverInfo {
                chipset: HardwareChipset::IntelSgx,
                device_node: "/dev/sgx_enclave".to_string(),
                driver_version: "1.0".to_string(),
                vendor_id: "intel".to_string(),
            }),
            ..evidence.clone()
        };
        assert!(matches!(
            HeadlessEnclave::verify_hardware_attestation_evidence(&bad_driver_chipset),
            Err(EnclaveVerificationError::InvalidDriverInfo(_))
        ));
    }
}
