use conxian_gateway::engine::mcp::McpManager;
use conxian_gateway::engine::Engine;
use std::sync::Arc;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize engine for the standalone MCP server
    let engine = Arc::new(Engine::new());
    engine.initialize();

    let manager = McpManager::new(engine);

    eprintln!("Conxian MCP Server v0.2.2 is starting (stdio)...");

    let mut reader = BufReader::new(stdin()).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        let req: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = req.get("id").cloned();

        match method {
            "initialize" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "conxian-gateway-mcp",
                            "version": "0.2.2"
                        }
                    }
                });
                println!("{}", response);
            }
            "tools/list" => {
                let tools = vec![
                    manager.get_telemetry_tool(),
                    manager.get_proof_tool(),
                    manager.get_yield_oracle_tool(),
                    manager.get_industrial_intents_tool(),
                    manager.get_draft_intent_tool(),
                ];
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools
                    }
                });
                println!("{}", response);
            }
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(serde_json::json!({}));
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let result = manager.handle_call(tool_name, arguments).await;

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                });
                println!("{}", response);
            }
            "notifications/initialized" => {
                // No response needed
            }
            "exit" => break,
            _ => {
                if let Some(msg_id) = id {
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": msg_id,
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        }
                    });
                    println!("{}", response);
                }
            }
        }
    }

    Ok(())
}
