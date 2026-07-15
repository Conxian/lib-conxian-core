//! Clarity contract bridge for Stacks integration.
//!
//! Provides types for creating signed Clarity contract calls.

use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A Clarity contract call payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClarityCall {
    pub contract_address: String,
    pub contract_name: String,
    pub function_name: String,
    pub arguments: Vec<String>,
    pub sender_address: String,
}

/// A signed Clarity contract call ready for broadcast.
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedContractCall {
    pub payload: ClarityCall,
    pub signature: String,
    pub public_key: String,
}

/// Bridge for creating signed Clarity contract calls.
///
/// # Example
///
/// ```rust,ignore
/// use k256::ecdsa::SigningKey;
/// use lib_conxian_core::contract_bridge::ContractBridge;
///
/// let signing_key = SigningKey::from_slice(&hex::decode("...")?)?;
/// let bridge = ContractBridge::new();
/// let signed_call = bridge.create_signed_call(
///     &signing_key,
///     "ST1...",
///     "contract-name.function-name",
///     vec![],
/// )?;
/// ```
pub struct ContractBridge;

impl ContractBridge {
    /// Creates a signed Clarity contract call.
    ///
    /// # Arguments
    ///
    /// * `signing_key` - The ECDSA signing key for the sender
    /// * `contract` - The contract principal in format "address.name"
    /// * `function` - The function name to call
    /// * `args` - The function arguments
    pub fn create_signed_call(
        signing_key: &SigningKey,
        contract: &str,
        function: &str,
        args: Vec<String>,
    ) -> anyhow::Result<SignedContractCall> {
        let (addr, name) = parse_contract_principal(contract)?;

        let sender_address = compute_stacks_address(signing_key);

        let call = ClarityCall {
            contract_address: addr,
            contract_name: name,
            function_name: function.to_string(),
            arguments: args,
            sender_address,
        };

        let serialized = serde_json::to_string(&call)
            .map_err(|e| anyhow::anyhow!("serialization failed: {e}"))?;

        let signature: Signature = signing_key.sign(serialized.as_bytes());
        let verifying_key = signing_key.verifying_key();
        let pubkey_point = verifying_key.to_sec1_point(true);
        let public_key = hex::encode(pubkey_point.as_bytes());

        Ok(SignedContractCall {
            payload: call,
            signature: hex::encode(signature.to_bytes()),
            public_key,
        })
    }
}

/// Computes a Stacks address hash from a public key.
fn compute_stacks_address(signing_key: &SigningKey) -> String {
    let verifying_key = signing_key.verifying_key();
    let pubkey_point = verifying_key.to_sec1_point(true);
    let pubkey_bytes = pubkey_point.as_bytes();
    let sha2_hash = Sha256::digest(pubkey_bytes);
    let hash160 = Ripemd160::digest(sha2_hash);
    hex::encode(hash160)
}

fn parse_contract_principal(contract: &str) -> anyhow::Result<(String, String)> {
    let contract = contract.trim();
    if contract.is_empty() {
        return Err(anyhow::anyhow!("contract principal is empty"));
    }

    let (addr, name) = contract
        .split_once('.')
        .ok_or_else(|| anyhow::anyhow!("invalid contract principal: {contract}"))?;

    let addr = addr.trim();
    let name = name.trim();
    if addr.is_empty() || name.is_empty() {
        return Err(anyhow::anyhow!("invalid contract principal: {contract}"));
    }

    Ok((addr.to_string(), name.to_string()))
}
