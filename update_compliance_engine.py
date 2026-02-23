with open('gateway/src/engine/mod.rs', 'r') as f:
    content = f.read()

compliance_struct = """
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComplianceStatus {
    pub status: String,
    pub last_audit: DateTime<Utc>,
    pub rules_active: Vec<String>,
    pub risk_score: u32,
}
"""

content = content.replace(
    'pub struct PriceInfo {',
    'pub struct PriceInfo {'
).replace(
    '    pub source: String,\n}',
    '    pub source: String,\n}\n' + compliance_struct
)

content = content.replace(
    'pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,',
    'pub prices: Arc<RwLock<HashMap<String, PriceInfo>>>,\n    pub compliance: Arc<RwLock<ComplianceStatus>>,'
)

content = content.replace(
    '        Self {',
    '        let compliance = ComplianceStatus {\n            status: "compliant".to_string(),\n            last_audit: Utc::now(),\n            rules_active: vec!["KYC".to_string(), "AML".to_string(), "NetworkIntegrity".to_string()],\n            risk_score: 15,\n        };\n\n        Self {'
).replace(
    'prices: Arc::new(RwLock::new(prices)),',
    'prices: Arc::new(RwLock::new(prices)),\n            compliance: Arc::new(RwLock::new(compliance)),'
)

content = content.replace(
    'pub fn get_prices(&self) -> HashMap<String, PriceInfo> {\n        self.prices.read().unwrap().clone()\n    }',
    'pub fn get_prices(&self) -> HashMap<String, PriceInfo> {\n        self.prices.read().unwrap().clone()\n    }\n\n    pub fn get_compliance_status(&self) -> ComplianceStatus {\n        self.compliance.read().unwrap().clone()\n    }'
)

# Add background monitoring for compliance risk score
monitoring_compliance = """
                {
                    let mut compliance = engine_clone.compliance.write().unwrap();
                    let current_requests = engine_clone.request_count.load(Ordering::SeqCst);
                    // Simulate dynamic risk score based on activity
                    compliance.risk_score = (10 + (current_requests % 20) as u32).min(100);
                    if compliance.risk_score > 80 {
                        compliance.status = "warning".to_string();
                    } else {
                        compliance.status = "compliant".to_string();
                    }
                }
"""

content = content.replace(
    '                {\n                    let mut prices = engine_clone.prices.write().unwrap();',
    monitoring_compliance + '\n                {\n                    let mut prices = engine_clone.prices.write().unwrap();'
)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
