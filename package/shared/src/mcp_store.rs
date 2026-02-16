use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::McpConfig;

fn mcp_config_path() -> Result<PathBuf> {
    Ok(crate::paths::config_dir()?.join("mcp.json"))
}

pub fn load() -> McpConfig {
    match load_from_file() {
        Ok(config) => config,
        Err(_) => McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        },
    }
}

fn load_from_file() -> Result<McpConfig> {
    let path = mcp_config_path()?;
    if !path.exists() {
        anyhow::bail!("mcp.json not found");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let config: McpConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    info!(
        count = config.mcp_servers.len(),
        path = %path.display(),
        "Loaded MCP config from file"
    );
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::McpServerConfig;

    #[test]
    fn test_load_returns_empty_when_no_file() {
        let config = load();
        assert!(config.mcp_servers.is_empty());
    }

    #[test]
    fn test_parse_stdio_server() {
        let json = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
                }
            }
        }"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        let server = &config.mcp_servers["filesystem"];
        assert_eq!(server.command.as_deref(), Some("npx"));
        assert_eq!(
            server.args.as_ref().unwrap(),
            &vec!["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
        );
        assert!(server.url.is_none());
    }

    #[test]
    fn test_parse_http_server() {
        let json = r#"{
            "mcpServers": {
                "remote": {
                    "url": "http://localhost:8080/mcp"
                }
            }
        }"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        let server = &config.mcp_servers["remote"];
        assert!(server.command.is_none());
        assert_eq!(server.url.as_deref(), Some("http://localhost:8080/mcp"));
    }

    #[test]
    fn test_parse_mixed_servers() {
        let json = r#"{
            "mcpServers": {
                "local": {
                    "command": "my-tool",
                    "args": ["serve"],
                    "env": {"FOO": "bar"}
                },
                "remote": {
                    "url": "https://example.com/mcp"
                }
            }
        }"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 2);

        let local = &config.mcp_servers["local"];
        assert_eq!(local.command.as_deref(), Some("my-tool"));
        assert_eq!(local.env.as_ref().unwrap().get("FOO").unwrap(), "bar");

        let remote = &config.mcp_servers["remote"];
        assert_eq!(remote.url.as_deref(), Some("https://example.com/mcp"));
    }

    #[test]
    fn test_parse_empty_servers() {
        let json = r#"{"mcpServers": {}}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert!(config.mcp_servers.is_empty());
    }

    #[test]
    fn test_server_config_is_stdio() {
        let server = McpServerConfig {
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "server".to_string()]),
            env: None,
            url: None,
        };
        assert!(server.command.is_some());
        assert!(server.url.is_none());
    }

    #[test]
    fn test_server_config_is_http() {
        let server = McpServerConfig {
            command: None,
            args: None,
            env: None,
            url: Some("http://localhost:8080".to_string()),
        };
        assert!(server.command.is_none());
        assert!(server.url.is_some());
    }
}
