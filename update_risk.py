with open('gateway/src/engine/mod.rs', 'r') as f:
    content = f.read()

new_logic = """    fn calculate_risk_level(latency: u32, trust_model: &str) -> String {
        if latency > 250 || trust_model == "Centralized" {
            "High".to_string()
        } else if latency > 150 || trust_model == "Federated" || trust_model == "Optimistic" || trust_model == "Optimistic Rollup" || trust_model == "Powpeg" || trust_model == "Spiderchain" {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }"""

import re
content = re.sub(r'fn calculate_risk_level\(.*?\)\s*->\s*String\s*\{.*?\}', new_logic, content, flags=re.DOTALL)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
