//! Advanced Cryptography for Trust-Minimized Execution
//! Aligned with CXIP 20 Section 3.0

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoStubError {
    NotImplemented(&'static str),
}

impl fmt::Display for CryptoStubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(api) => write!(f, "{api} is not implemented"),
        }
    }
}

impl std::error::Error for CryptoStubError {}

pub struct PVDE; // Practical Verifiable Delay Encryption

impl PVDE {
    pub fn generate_puzzle(_delay: u64, _data: &[u8]) -> Result<String, CryptoStubError> {
        Err(CryptoStubError::NotImplemented("PVDE::generate_puzzle"))
    }
}

pub struct WitnessEncryption;

impl WitnessEncryption {
    pub fn encrypt_to_bitcoin_finality(
        _depth: u32,
        _data: &[u8],
    ) -> Result<String, CryptoStubError> {
        Err(CryptoStubError::NotImplemented(
            "WitnessEncryption::encrypt_to_bitcoin_finality",
        ))
    }
}

pub struct AdaptorSignature;

impl AdaptorSignature {
    pub fn create_adaptor_signature(
        _secret: &str,
        _message: &str,
    ) -> Result<String, CryptoStubError> {
        Err(CryptoStubError::NotImplemented(
            "AdaptorSignature::create_adaptor_signature",
        ))
    }
}
