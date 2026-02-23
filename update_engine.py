import sys

with open('gateway/src/engine/mod.rs', 'r') as f:
    content = f.read()

# Add PriceInfo struct and update Engine struct
price_info_struct = """
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceInfo {
    pub asset: String,
    pub price_usd: f64,
    pub last_updated: DateTime<Utc>,
    pub source: String,
}
"""

content = content.replace(
    'pub struct ReserveAsset {',
    'pub struct ReserveAsset {'
).replace(
    '    pub status: String,\n}',
    '    pub status: String,\n}\n' + price_info_struct
)

content = content.replace(
    'pub reserves: Arc<RwLock<Vec<ReserveAsset>>>,',
    'pub reserves: Arc<RwLock<Vec<ReserveAsset>>>,\n    pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,'
)

# Add new services
content = content.replace(
    '("b2network", 45, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),',
    '("b2network", 45, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),\n            ("citrea", 52, "ZK Rollup", "Medium", "On-chain (ZK)", "Bitcoin", "ZK Bridge"),\n            ("bitlayer", 60, "Optimistic", "Medium", "On-chain", "Bitcoin", "BitVM Bridge"),'
)

# Add new service metadata
content = content.replace(
    '                "b2network" => {\n                    metadata.insert("block_height".to_string(), "12543".to_string());\n                },',
    '                "b2network" => {\n                    metadata.insert("block_height".to_string(), "12543".to_string());\n                },\n                "citrea" => {\n                    metadata.insert("tvl_usd".to_string(), "12500000".to_string());\n                },\n                "bitlayer" => {\n                    metadata.insert("tvl_usd".to_string(), "8500000".to_string());\n                },'
)

# Initialize prices and update Engine::new return
new_prices_init = """
        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), PriceInfo { asset: "BTC".to_string(), price_usd: 65000.0, last_updated: Utc::now(), source: "Conxian Oracle".to_string() });
        prices.insert("STX".to_string(), PriceInfo { asset: "STX".to_string(), price_usd: 2.5, last_updated: Utc::now(), source: "Conxian Oracle".to_string() });
"""

content = content.replace(
    '        Self {',
    new_prices_init + '\n        Self {'
).replace(
    'service_statuses: Arc::new(RwLock::new(statuses)),\n            reserves: Arc::new(RwLock::new(reserves)),',
    'service_statuses: Arc::new(RwLock::new(statuses)),\n            reserves: Arc::new(RwLock::new(reserves)),\n            prices: Arc::new(RwLock::new(prices)),'
)

# Add get_prices method
content = content.replace(
    'pub fn get_reserves(&self) -> Vec<ReserveAsset> {\n        self.reserves.read().unwrap().clone()\n    }',
    'pub fn get_reserves(&self) -> Vec<ReserveAsset> {\n        self.reserves.read().unwrap().clone()\n    }\n\n    pub fn get_prices(&self) -> HashMap<String, PriceInfo> {\n        self.prices.read().unwrap().clone()\n    }'
)

# Update background monitoring for new services
content = content.replace(
    '                            "b2network" => {\n                                if let Some(v) = status.metadata.get_mut("block_height") {\n                                    let height: u64 = v.parse().unwrap_or(12543);\n                                    *v = (height + 1).to_string();\n                                }\n                            },',
    '                            "b2network" => {\n                                if let Some(v) = status.metadata.get_mut("block_height") {\n                                    let height: u64 = v.parse().unwrap_or(12543);\n                                    *v = (height + 1).to_string();\n                                }\n                            },\n                            "citrea" => {\n                                if let Some(v) = status.metadata.get_mut("tvl_usd") {\n                                    let tvl: f64 = v.parse().unwrap_or(12500000.0);\n                                    *v = format!("{:.0}", tvl + 2500.0);\n                                }\n                            },\n                            "bitlayer" => {\n                                if let Some(v) = status.metadata.get_mut("tvl_usd") {\n                                    let tvl: f64 = v.parse().unwrap_or(8500000.0);\n                                    *v = format!("{:.0}", tvl + 1800.0);\n                                }\n                            },'
)

# Update background monitoring for prices
monitoring_prices = """
                {
                    let mut prices = engine_clone.prices.write().unwrap();
                    for price in prices.values_mut() {
                        let fluctuation = (Utc::now().timestamp() % 101) as f64 / 10000.0 - 0.005;
                        price.price_usd *= 1.0 + fluctuation;
                        price.last_updated = Utc::now();
                    }
                }
"""

content = content.replace(
    '                {\n                    let mut reserves = engine_clone.reserves.write().unwrap();',
    monitoring_prices + '\n                {\n                    let mut reserves = engine_clone.reserves.write().unwrap();'
)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
