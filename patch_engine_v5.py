import sys

content = open('gateway/src/engine/mod.rs').read()

old_liquid_peg = """    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "asset": "L-BTC",
            "peg_status": "Active",
            "collateral_ratio": 1.0,
            "verified_on_chain": true
        })
    }"""

new_liquid_peg = """    pub fn get_liquid_peg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "asset": "L-BTC",
                "peg_status": "ConnectionRequired",
                "error": "Mainnet node connection required for Liquid peg verification.",
                "remediation": "Configure LIQUID_RPC_URL"
            });
        }
        serde_json::json!({
            "asset": "L-BTC",
            "peg_status": "Active",
            "collateral_ratio": 1.0,
            "verified_on_chain": true
        })
    }"""

old_rootstock_powpeg = """    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        serde_json::json!({
            "asset": "RBTC",
            "powpeg_status": "Active",
            "signatories_active": 12,
            "btc_locked": 2500.0
        })
    }"""

new_rootstock_powpeg = """    pub fn get_rootstock_powpeg(&self) -> serde_json::Value {
        self.increment_requests();
        if remediation::is_production_mainnet() {
             return serde_json::json!({
                "asset": "RBTC",
                "powpeg_status": "ConnectionRequired",
                "error": "Mainnet node connection required for Rootstock powpeg verification.",
                "remediation": "Configure ROOTSTOCK_RPC_URL"
            });
        }
        serde_json::json!({
            "asset": "RBTC",
            "powpeg_status": "Active",
            "signatories_active": 12,
            "btc_locked": 2500.0
        })
    }"""

content = content.replace(old_liquid_peg, new_liquid_peg)
content = content.replace(old_rootstock_powpeg, new_rootstock_powpeg)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
