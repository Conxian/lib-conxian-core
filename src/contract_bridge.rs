use serde::{Deserialize, Serialize};

use crate::Wallet;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClarityCall {
    pub contract_address: String,
    pub contract_name: String,
    pub function_name: String,
    pub arguments: Vec<String>,
    pub sender_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedContractCall {
    pub payload: ClarityCall,
    pub signature: String,
    pub public_key: String,
}

pub struct ContractBridge;

impl ContractBridge {
    pub fn create_signed_call(
        wallet: &Wallet,
        contract: &str,
        function: &str,
        args: Vec<String>,
    ) -> anyhow::Result<SignedContractCall> {
        let (addr, name) = parse_contract_principal(contract)?;

        let call = ClarityCall {
            contract_address: addr,
            contract_name: name,
            function_name: function.to_string(),
            arguments: args,
            sender_address: wallet.stacks_address_hash(),
        };

        let serialized = serde_json::to_string(&call)
            .map_err(|e| anyhow::anyhow!("serialization failed: {e}"))?;
        let signature = wallet.sign(&serialized);

        Ok(SignedContractCall {
            payload: call,
            signature,
            public_key: wallet.public_key(),
        })
    }
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
