use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use crate::session::SessionManager;

#[derive(Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub struct McpServer {
    manager: Arc<SessionManager>,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(SessionManager::new()),
        }
    }

    pub fn run(&self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            if let Ok(req) = serde_json::from_str::<RpcRequest>(&line) {
                let mut response = RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: None,
                    error: None,
                };

                match self.handle_request(&req) {
                    Ok(Value::Null) => continue, // Notifications don't get responses
                    Ok(res) => response.result = Some(res),
                    Err(err) => {
                        response.error = Some(serde_json::json!({
                            "code": -32603,
                            "message": err.to_string()
                        }));
                    }
                }

                if let Ok(json) = serde_json::to_string(&response) {
                    writeln!(stdout, "{}", json).unwrap();
                    stdout.flush().unwrap();
                }
            }
        }
    }

    fn handle_request(&self, req: &RpcRequest) -> anyhow::Result<Value> {
        // Very basic MCP Tool mapping
        // In a real implementation this would use a robust MCP crate like `mcp-rs`
        match req.method.as_str() {
            "initialize" => {
                Ok(serde_json::json!({
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "async-shell",
                        "version": "0.1.0"
                    },
                    "protocolVersion": "2024-11-05"
                }))
            }
            "notifications/initialized" => {
                // Return null to signify successful handling of the notification without sending a response object
                Ok(Value::Null)
            }
            "tools/list" => {
                Ok(serde_json::json!({
                    "tools": [
                        {
                            "name": "register_agent",
                            "description": "Initializes a new agent session and returns a unique Ed25519 public key (hex) to be used as your `agent_id`. You must call this once to get an ID before reading from or spawning PTYs.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "spawn",
                            "description": "Spawn a background process in a PTY",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "agent_id": { "type": "string" },
                                    "session_id": { "type": "string" },
                                    "command": { "type": "string" }
                                },
                                "required": ["agent_id", "session_id", "command"]
                            }
                        },
                        {
                            "name": "read_history",
                            "description": "Read the scrollback history of a background PTY. If start_line is omitted, it acts as a cursor and resumes reading exactly where this agent last left off.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "agent_id": { "type": "string", "description": "A unique identifier for the agent requesting the history (to track cursor positions independently)." },
                                    "session_id": { "type": "string" },
                                    "start_line": { "type": "integer", "description": "Optional 0-indexed start line. If omitted, resumes from the agent's last read position." },
                                    "max_lines": { "type": "integer", "description": "Maximum number of lines to return. Defaults to 100." }
                                },
                                "required": ["agent_id", "session_id"]
                            }
                        },
                        {
                            "name": "write_stdin",
                            "description": "Send keystrokes to a background PTY",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "agent_id": { "type": "string" },
                                    "session_id": { "type": "string" },
                                    "input": { "type": "string" }
                                },
                                "required": ["agent_id", "session_id", "input"]
                            }
                        }
                    ]
                }))
            }
            "tools/call" => {
                let params = req.params.as_ref().ok_or_else(|| anyhow::anyhow!("Missing params"))?;
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments").ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;

                match name {
                    "register_agent" => {
                        let agent_id = self.manager.register_agent();
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": agent_id }] }))
                    }
                    "spawn" => {
                        let agent_id = args.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
                        let id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
                        let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                        self.manager.spawn(id, agent_id, cmd)?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Spawned session '{}'", id) }] }))
                    }
                    "read_history" => {
                        let agent_id = args.get("agent_id").and_then(|v| v.as_str()).unwrap_or("default_agent");
                        let id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
                        let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
                        let max_lines = args.get("max_lines").and_then(|v| v.as_u64()).map(|v| v as usize);
                        
                        let output = self.manager.read_history(id, agent_id, start_line, max_lines)?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": output }] }))
                    }
                    "write_stdin" => {
                        let agent_id = args.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
                        let id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
                        let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("");
                        self.manager.write_stdin(id, agent_id, input)?;
                        Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Wrote to session '{}'", id) }] }))
                    }
                    _ => anyhow::bail!("Unknown tool {}", name),
                }
            }
            _ => anyhow::bail!("Method not supported"),
        }
    }
}
