use crate::models::McpServer;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::collections::HashMap;

pub fn discover_mcp_servers() -> Result<Vec<McpServer>> {
    let home = std::env::var("HOME")?;
    let settings_path = Path::new(&home).join(".gemini").join("settings.json");
    if !settings_path.exists() { return Ok(vec![]); }

    let data = fs::read_to_string(settings_path)?;
    let v: serde_json::Value = serde_json::from_str(&data)?;
    let mut servers = Vec::new();

    if let Some(mcp) = v.get("mcpServers").and_then(|m| m.as_object()) {
        for (name, config) in mcp {
            let mut server = McpServer {
                name: name.clone(),
                url: config.get("httpUrl").and_then(|u| u.as_str()).map(std::string::ToString::to_string),
                command: config.get("command").and_then(|u| u.as_str()).map(std::string::ToString::to_string),
                args: vec![],
                env: HashMap::new(),
            };
            if let Some(args) = config.get("args").and_then(|a| a.as_array()) {
                server.args = args.iter().filter_map(|a| a.as_str().map(std::string::ToString::to_string)).collect();
            }
            if let Some(env) = config.get("env").and_then(|e| e.as_object()) {
                for (k, val) in env {
                    if let Some(s) = val.as_str() {
                        server.env.insert(k.clone(), s.to_string());
                    }
                }
            }
            servers.push(server);
        }
    }
    Ok(servers)
}
