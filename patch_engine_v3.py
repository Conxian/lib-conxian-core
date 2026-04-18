import sys

content = open('gateway/src/engine/mod.rs').read()

# Already patched the start of initialize_services, now patch the rest

old_metadata_part = """            if name == "stacks" {
                metadata.insert("block_height".to_string(), "841500".to_string());
                metadata.insert("hiro_api_connected".to_string(), "true".to_string());
            }
            if name == "lorenzo" {
                metadata.insert("staked_btc".to_string(), "1250.5".to_string());
            }
            if name == "b2network" {
                metadata.insert("block_height".to_string(), "12600".to_string());
            }"""

new_metadata_part = """            if !is_mainnet {
                if name == "stacks" {
                    metadata.insert("block_height".to_string(), "841500".to_string());
                    metadata.insert("hiro_api_connected".to_string(), "true".to_string());
                }
                if name == "lorenzo" {
                    metadata.insert("staked_btc".to_string(), "1250.5".to_string());
                }
                if name == "b2network" {
                    metadata.insert("block_height".to_string(), "12600".to_string());
                }
            }"""

old_reserves_prices = """        let mut reserves = self.reserves.write().unwrap();
        reserves.push(ReserveAsset {
            asset: "L-BTC".to_string(),
            total_supplied: 1500.0,
            total_reserves: 1500.0,
            collateral_ratio: 1.0,
            status: "Verified (On-chain)".to_string(),
        });
        reserves.push(ReserveAsset {
            asset: "RBTC".to_string(),
            total_supplied: 2500.0,
            total_reserves: 2500.0,
            collateral_ratio: 1.0,
            status: "Verified (On-chain)".to_string(),
        });

        let mut prices = self.prices.write().unwrap();
        prices.insert(
            "BTC".to_string(),
            PriceInfo {
                asset: "BTC".to_string(),
                price_usd: 65000.0,
                last_updated: Utc::now(),
                source: "CoinGecko".to_string(),
            },
        );
        prices.insert(
            "STX".to_string(),
            PriceInfo {
                asset: "STX".to_string(),
                price_usd: 2.50,
                last_updated: Utc::now(),
                source: "CoinGecko".to_string(),
            },
        );"""

new_reserves_prices = """        if !is_mainnet {
            let mut reserves = self.reserves.write().unwrap();
            reserves.push(ReserveAsset {
                asset: "L-BTC".to_string(),
                total_supplied: 1500.0,
                total_reserves: 1500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });
            reserves.push(ReserveAsset {
                asset: "RBTC".to_string(),
                total_supplied: 2500.0,
                total_reserves: 2500.0,
                collateral_ratio: 1.0,
                status: "Verified (On-chain)".to_string(),
            });

            let mut prices = self.prices.write().unwrap();
            prices.insert(
                "BTC".to_string(),
                PriceInfo {
                    asset: "BTC".to_string(),
                    price_usd: 65000.0,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
            prices.insert(
                "STX".to_string(),
                PriceInfo {
                    asset: "STX".to_string(),
                    price_usd: 2.50,
                    last_updated: Utc::now(),
                    source: "CoinGecko (Simulated)".to_string(),
                },
            );
        }"""

content = content.replace(old_metadata_part, new_metadata_part)
content = content.replace(old_reserves_prices, new_reserves_prices)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
