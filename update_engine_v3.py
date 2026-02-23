with open('gateway/src/engine/mod.rs', 'r') as f:
    content = f.read()

# Add get_exchange_rate method
exchange_rate_method = """
    pub fn get_exchange_rate(&self, from: &str, to: &str) -> serde_json::Value {
        self.increment_requests();
        let rate = match (from, to) {
            ("BTC", "USD") => 65000.0,
            ("USD", "BTC") => 1.0 / 65000.0,
            ("STX", "BTC") => 0.000038,
            _ => 1.0,
        };
        serde_json::json!({
            "from": from,
            "to": to,
            "rate": rate,
            "timestamp": Utc::now()
        })
    }

    pub fn is_healthy(&self) -> bool {
        let statuses = self.service_statuses.read().unwrap();
        // Check if at least some services are active and checked recently
        if statuses.is_empty() { return false; }
        let now = Utc::now();
        statuses.values().any(|s| (now - s.last_checked).num_seconds() < 60)
    }
"""

# Insert before the last closing brace of the Engine impl (which is at the very end of the file)
# Wait, I need to find the right place. The last block in the file is an Engine impl block.
# Let's find the last '}' and insert before it.

last_brace_idx = content.rfind('}')
content = content[:last_brace_idx] + exchange_rate_method + content[last_brace_idx:]

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
