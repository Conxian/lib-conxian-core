use anyhow::Context;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub const ENV_CONXIAN_PRIVATE_KEY_HEX: &str = "CONXIAN_PRIVATE_KEY_HEX";
pub const ENV_NEXUS_PRIVATE_KEY: &str = "NEXUS_PRIVATE_KEY";

#[derive(Clone)]
pub struct Wallet {
    signing_key: SigningKey,
}

impl Wallet {
    pub fn new() -> anyhow::Result<Self> {
        if let Ok(hex_key) = std::env::var(ENV_CONXIAN_PRIVATE_KEY_HEX) {
            return Self::from_private_key_hex(&hex_key);
        }

        if let Ok(hex_key) = std::env::var(ENV_NEXUS_PRIVATE_KEY) {
            return Self::from_private_key_hex(&hex_key);
        }

        Err(anyhow::anyhow!(
            "missing private key env var: set {ENV_CONXIAN_PRIVATE_KEY_HEX} (or legacy {ENV_NEXUS_PRIVATE_KEY})"
        ))
    }

    pub fn from_private_key_hex(hex_key: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_key.trim())
            .with_context(|| "invalid hex in private key env var")?;
        Self::from_private_key_bytes(&bytes)
    }

    pub fn from_private_key_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let signing_key = SigningKey::from_slice(bytes).with_context(|| "invalid private key")?;
        Ok(Self { signing_key })
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }

    pub fn public_key(&self) -> String {
        hex::encode(self.public_key_bytes())
    }

    pub fn stacks_address_hash(&self) -> String {
        let pubkey = self.public_key_bytes();
        let sha2 = Sha256::digest(&pubkey);
        let hash160 = Ripemd160::digest(sha2);
        hex::encode(hash160)
    }

    pub fn sign(&self, message: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let digest = hasher.finalize();
        let signature: Signature = self.signing_key.sign(&digest);
        hex::encode(signature.to_bytes())
    }
}

pub fn sign_transaction(tx_id: &str) -> anyhow::Result<String> {
    let wallet = Wallet::new()?;
    Ok(wallet.sign(tx_id))
}
