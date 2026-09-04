//! OP_CAT Recursive Covenants (BIP-347)
//! Script templates for advanced Bitcoin vaults

use sha2::{Digest, Sha256};

/// Typed errors for covenant script construction and verification (BIP-347).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CovenantError {
    /// Pubkey length must be either 32 bytes (x-only Taproot) or 33 bytes (compressed SEC1).
    InvalidPubkeyLength(usize),
    /// Target hash must be exactly 32 bytes (SHA-256 digest).
    InvalidTargetHashLength(usize),
    /// Preimage component is empty or malformed.
    MalformedPreimage,
}

impl std::fmt::Display for CovenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPubkeyLength(len) => {
                write!(
                    f,
                    "invalid covenant pubkey length: {len} (expected 32 or 33 bytes)"
                )
            }
            Self::InvalidTargetHashLength(len) => {
                write!(
                    f,
                    "invalid covenant target hash length: {len} (expected 32 bytes)"
                )
            }
            Self::MalformedPreimage => write!(f, "malformed covenant preimage component"),
        }
    }
}

impl std::error::Error for CovenantError {}

/// Manager for constructing and verifying OP_CAT based covenants.
pub struct CovenantManager;

impl CovenantManager {
    /// Generates a basic OP_CAT recursive vault script.
    /// Logic: <pubkey> OP_CHECKSIGVERIFY <preimage_prefix> OP_CAT <preimage_suffix> OP_SHA256 <hash> OP_EQUAL
    ///
    /// This template follows the pattern where a transaction must reveal a preimage
    /// that, when concatenated (OP_CAT), matches a committed hash, enforcing spending constraints.
    pub fn generate_cat_vault_script(pubkey: &[u8], target_hash: &[u8]) -> Vec<u8> {
        let mut script = Vec::new();

        // ASN.1 style byte encoding for script representation
        script.push(pubkey.len() as u8);
        script.extend_from_slice(pubkey);

        script.push(0xad); // OP_CHECKSIGVERIFY

        // Hardened logic for preimage concatenation (BIP-347)
        script.push(0x7e); // OP_CAT

        script.push(0xa8); // OP_SHA256

        // Push Target Hash
        script.push(target_hash.len() as u8);
        script.extend_from_slice(target_hash);

        script.push(0x87); // OP_EQUAL

        script
    }

    /// Generates an OP_CAT recursive vault script with strict input validation.
    /// Returns `CovenantError` if pubkey or target_hash dimensions are invalid.
    pub fn generate_cat_vault_script_checked(
        pubkey: &[u8],
        target_hash: &[u8],
    ) -> Result<Vec<u8>, CovenantError> {
        if pubkey.len() != 32 && pubkey.len() != 33 {
            return Err(CovenantError::InvalidPubkeyLength(pubkey.len()));
        }
        if target_hash.len() != 32 {
            return Err(CovenantError::InvalidTargetHashLength(target_hash.len()));
        }

        Ok(Self::generate_cat_vault_script(pubkey, target_hash))
    }

    /// Verifies if a given preimage satisfies the recursive invariant of a CAT script.
    /// BIP-347: ensures sha256(prefix || suffix) == committed_hash
    pub fn verify_recursive_invariant(prefix: &[u8], suffix: &[u8], committed_hash: &[u8]) -> bool {
        if prefix.is_empty() || suffix.is_empty() || committed_hash.len() != 32 {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(prefix);
        hasher.update(suffix);
        let result = hasher.finalize();

        result.as_slice() == committed_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_vault_script_generation() {
        let pubkey = [0u8; 33];
        let target_hash = [0u8; 32];
        let script = CovenantManager::generate_cat_vault_script(&pubkey, &target_hash);

        assert!(script.contains(&0xad)); // OP_CHECKSIGVERIFY
        assert!(script.contains(&0x7e)); // OP_CAT
        assert!(script.contains(&0xa8)); // OP_SHA256
        assert!(script.contains(&0x87)); // OP_EQUAL
    }

    #[test]
    fn test_cat_vault_script_checked_validation() {
        let pub33 = [0x02; 33];
        let pub32 = [0x01; 32];
        let hash32 = [0xaa; 32];

        // Valid compressed 33-byte pubkey
        assert!(CovenantManager::generate_cat_vault_script_checked(&pub33, &hash32).is_ok());

        // Valid x-only 32-byte pubkey
        assert!(CovenantManager::generate_cat_vault_script_checked(&pub32, &hash32).is_ok());

        // Invalid pubkey lengths
        assert_eq!(
            CovenantManager::generate_cat_vault_script_checked(&[0x01; 31], &hash32),
            Err(CovenantError::InvalidPubkeyLength(31))
        );
        assert_eq!(
            CovenantManager::generate_cat_vault_script_checked(&[0x01; 34], &hash32),
            Err(CovenantError::InvalidPubkeyLength(34))
        );

        // Invalid target hash lengths
        assert_eq!(
            CovenantManager::generate_cat_vault_script_checked(&pub33, &[0xaa; 31]),
            Err(CovenantError::InvalidTargetHashLength(31))
        );
    }

    #[test]
    fn test_recursive_invariant_verification() {
        let prefix = b"part1";
        let suffix = b"part2";

        let mut hasher = Sha256::new();
        hasher.update(prefix);
        hasher.update(suffix);
        let hash = hasher.finalize();

        assert!(CovenantManager::verify_recursive_invariant(
            prefix,
            suffix,
            hash.as_slice()
        ));
        assert!(!CovenantManager::verify_recursive_invariant(
            prefix,
            b"wrong",
            hash.as_slice()
        ));
        assert!(!CovenantManager::verify_recursive_invariant(
            b"",
            suffix,
            hash.as_slice()
        ));
        assert!(!CovenantManager::verify_recursive_invariant(
            prefix,
            b"",
            hash.as_slice()
        ));
        assert!(!CovenantManager::verify_recursive_invariant(
            prefix,
            suffix,
            &[0x00; 31]
        ));
    }

    #[test]
    fn test_covenant_error_display() {
        assert_eq!(
            CovenantError::InvalidPubkeyLength(10).to_string(),
            "invalid covenant pubkey length: 10 (expected 32 or 33 bytes)"
        );
        assert_eq!(
            CovenantError::InvalidTargetHashLength(16).to_string(),
            "invalid covenant target hash length: 16 (expected 32 bytes)"
        );
        assert_eq!(
            CovenantError::MalformedPreimage.to_string(),
            "malformed covenant preimage component"
        );
    }
}
