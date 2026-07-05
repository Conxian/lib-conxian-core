//! OP_CAT Recursive Covenants (BIP-347)
//! Script templates for advanced Bitcoin vaults

use sha2::{Digest, Sha256};

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

    /// Verifies if a given preimage satisfies the recursive invariant of a CAT script.
    /// BIP-347: ensures sha256(prefix || suffix) == committed_hash
    pub fn verify_recursive_invariant(
        prefix: &[u8],
        suffix: &[u8],
        committed_hash: &[u8],
    ) -> bool {
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
    fn test_recursive_invariant_verification() {
        let prefix = b"part1";
        let suffix = b"part2";

        let mut hasher = Sha256::new();
        hasher.update(prefix);
        hasher.update(suffix);
        let hash = hasher.finalize();

        assert!(CovenantManager::verify_recursive_invariant(prefix, suffix, hash.as_slice()));
        assert!(!CovenantManager::verify_recursive_invariant(prefix, b"wrong", hash.as_slice()));
    }
}
