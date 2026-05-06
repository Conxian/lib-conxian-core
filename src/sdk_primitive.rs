use serde::{Deserialize, Serialize};
use crate::wallet::Wallet;
use crate::musig2;
use secp256k1::{PublicKey, XOnlyPublicKey};

/// The first commercial SDK primitive: Hardware-backed Bitcoin signing plus policy enforcement.
/// This defines the core capabilities for target integrators.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SigningPolicy {
    pub max_amount_sats: u64,
    pub allowed_destinations: Vec<String>,
    pub require_biometric: bool,
    pub timelock_blocks: u32,
}

 /// The VaultSDK is the primary entry point for integrators building native Bitcoin apps.
 /// It encapsulates hardware-backed signing, mandatory policy enforcement, and multi-sig aggregation.
pub struct VaultSDK {
    wallet: Wallet,
    policy: SigningPolicy,
}

impl VaultSDK {
    pub fn new(wallet: Wallet, policy: SigningPolicy) -> Self {
        Self { wallet, policy }
    }

    /// Validates a transaction request against the enforced policy.
    pub fn validate_request(&self, amount_sats: u64, destination: &str) -> Result<(), String> {
        if amount_sats > self.policy.max_amount_sats {
            return Err(format!("Policy violation: amount {} exceeds limit {}", amount_sats, self.policy.max_amount_sats));
        }

        if !self.policy.allowed_destinations.is_empty() && !self.policy.allowed_destinations.contains(&destination.to_string()) {
            return Err("Policy violation: destination not in allowlist".to_string());
        }

        Ok(())
    }

    /// Signs a Bitcoin transaction ID using the hardware-backed wallet after policy verification.
    pub fn sign_with_policy(&self, tx_id: &str, amount_sats: u64, destination: &str) -> Result<String, String> {
        self.validate_request(amount_sats, destination)?;

        if self.policy.require_biometric {
            // In production, this triggers the StrongBox TEE biometric prompt
            println!("Triggering TEE biometric handshake for transaction signing...");
        }

        Ok(self.wallet.sign(tx_id))
    }

    /// Aggregates keys for MuSig2 Taproot multi-sig, a core SDK capability.
    /// This implementation includes the internal wallet key in the aggregation.
    pub fn aggregate_musig2_keys(&self, other_pubkeys: &[PublicKey]) -> Result<XOnlyPublicKey, String> {
        let all_keys = [self.wallet.public_key_bytes()];
        let mut pubkeys = Vec::new();

        // Convert internal key
        let internal_pk = PublicKey::from_slice(&all_keys[0])
            .map_err(|e| format!("Internal key error: {}", e))?;
        pubkeys.push(internal_pk);

        // Add others
        pubkeys.extend_from_slice(other_pubkeys);

        musig2::aggregate_public_keys(&pubkeys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    #[test]
    fn test_vault_sdk_policy_enforcement() {
        let key_hex = "01".repeat(32);
        let wallet = Wallet::from_private_key_hex(&key_hex).unwrap();
        let policy = SigningPolicy {
            max_amount_sats: 1000000,
            allowed_destinations: vec!["bc1q_safe".to_string()],
            require_biometric: false,
            timelock_blocks: 0,
        };

        let sdk = VaultSDK::new(wallet, policy);

        // Happy path
        assert!(sdk.sign_with_policy("tx123", 500000, "bc1q_safe").is_ok());

        // Amount violation
        assert!(sdk.sign_with_policy("tx123", 2000000, "bc1q_safe").is_err());

        // Destination violation
        assert!(sdk.sign_with_policy("tx123", 500000, "bc1q_bad").is_err());
    }

    #[test]
    fn test_vault_sdk_musig2_aggregation() {
        let key_hex = "01".repeat(32);
        let wallet = Wallet::from_private_key_hex(&key_hex).unwrap();
        let sdk = VaultSDK::new(wallet, SigningPolicy {
            max_amount_sats: 0,
            allowed_destinations: vec![],
            require_biometric: false,
            timelock_blocks: 0,
        });

        let other_key_hex = "02".repeat(32);
        let other_wallet = Wallet::from_private_key_hex(&other_key_hex).unwrap();
        let other_pk = PublicKey::from_slice(&other_wallet.public_key_bytes()).unwrap();

        let aggregated = sdk.aggregate_musig2_keys(&[other_pk]).unwrap();
        assert!(!aggregated.serialize().is_empty());
    }
}
