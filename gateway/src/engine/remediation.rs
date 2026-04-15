use std::env;

/// Production check for mainnet-only enforcement.
pub fn is_production_mainnet() -> bool {
    env::var("CONXIAN_NETWORK").unwrap_or_else(|_| "mainnet".to_string()) == "mainnet"
}

/// Validates whether a request is appropriate for the current environment.
/// Production mainnet MUST NOT use testnet flags.
/// Non-production MUST use testnet flags to acknowledge validation state.
pub fn validate_request(is_testnet_request: bool) -> Result<(), String> {
    if is_production_mainnet() {
        if is_testnet_request {
            return Err("Testnet bypass is strictly prohibited on production mainnet.".to_string());
        }
    } else if !is_testnet_request {
        return Err(
            "Non-production environment requires explicit testnet flag for validation.".to_string(),
        );
    }
    Ok(())
}
