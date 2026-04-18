import sys

content = open('gateway/src/engine/mod.rs').read()

old_fetch = """    async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        let fallback_height = 841500;
        let client = reqwest::Client::new();
        match client
            .get("https://api.mainnet.hiro.so/v2/info")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                if let Ok(info) = resp.json::<serde_json::Value>().await {
                    if let Some(height) = info["stacks_tip_height"].as_u64() {
                        return Ok(height);
                    }
                }
                Ok(fallback_height)
            }
            Err(_) => Ok(fallback_height),
        }
    }"""

new_fetch = """    async fn fetch_stacks_block_height(&self) -> Result<u64, reqwest::Error> {
        let fallback_height = 841500;
        let client = reqwest::Client::new();
        let res = client
            .get("https://api.mainnet.hiro.so/v2/info")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        let height = match res {
            Ok(resp) => {
                if let Ok(info) = resp.json::<serde_json::Value>().await {
                    info["stacks_tip_height"].as_u64().unwrap_or(fallback_height)
                } else {
                    fallback_height
                }
            }
            Err(_) => fallback_height,
        };

        // Update Stacks metadata
        let mut statuses = self.service_statuses.write().unwrap();
        if let Some(status) = statuses.get_mut("stacks") {
            status.metadata.insert("block_height".to_string(), height.to_string());
            status.metadata.insert("hiro_api_connected".to_string(), "true".to_string());
            status.last_checked = Utc::now();
        }

        Ok(height)
    }"""

content = content.replace(old_fetch, new_fetch)

with open('gateway/src/engine/mod.rs', 'w') as f:
    f.write(content)
