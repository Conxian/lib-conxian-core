//! Vault SDK — Basic Usage Example
//!
//! Demonstrates wallet creation, policy enforcement, and transaction signing
//! using the primary `VaultSDK` commercial interface.
//!
//! Run with: `cargo run --example vault_sdk_basic`

use lib_conxian_core::{SigningPolicy, VaultSDK, Wallet};

fn main() -> anyhow::Result<()> {
    // 1. Create a wallet from a hex private key (32 bytes)
    let key_hex = "01".repeat(32);
    let wallet = Wallet::from_private_key_hex(&key_hex)?;

    // 2. Define a signing policy
    let policy = SigningPolicy {
        max_amount_sats: 1_000_000, // Max 0.01 BTC per tx
        allowed_destinations: vec!["bc1q_safe_vault_address".to_string()],
        require_biometric: false, // TEE biometric not required for example
        timelock_blocks: 0,
    };

    // 3. Initialize the Vault SDK
    let sdk = VaultSDK::new(wallet, policy);

    // 4. Sign a transaction within policy limits
    let sig = sdk
        .sign_with_policy("tx_001", 500_000, "bc1q_safe_vault_address")
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("Signed transaction tx_001: {}", sig);

    // 5. Policy violation — amount exceeds limit
    match sdk.sign_with_policy("tx_002", 2_000_000, "bc1q_safe_vault_address") {
        Ok(_) => println!("Unexpected: signed over limit"),
        Err(e) => println!("Policy enforced: {}", e),
    }

    // 6. Policy violation — unlisted destination
    match sdk.sign_with_policy("tx_003", 100_000, "bc1q_unknown") {
        Ok(_) => println!("Unexpected: signed to unlisted destination"),
        Err(e) => println!("Policy enforced: {}", e),
    }

    Ok(())
}
