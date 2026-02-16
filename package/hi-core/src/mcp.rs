use anyhow::{Context, Result};
use rig::tool::ToolDyn;
use rig::tool::rmcp::McpTool;
use rmcp::ServiceExt;
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::TokioChildProcess;
use shared::config::{McpConfig, McpServerConfig};
use tracing::{info, warn};

/// Holds live MCP server connections and the tools discovered from them.
///
/// Must be kept alive for the duration of the session — dropping it
/// closes child-process transports and HTTP sessions.
pub struct McpManager {
    _services: Vec<Box<dyn Send + Sync>>,
}

/// Wrapper to erase heterogeneous `RunningService` transport types.
/// The inner service must be kept alive (RAII) to maintain the MCP connection.
struct StdioService(#[allow(dead_code)] RunningService<RoleClient, ()>);
// SAFETY: RunningService is Send+Sync because it holds Arc<()> + Peer + JoinHandle.
unsafe impl Send for StdioService {}
unsafe impl Sync for StdioService {}

struct HttpService(#[allow(dead_code)] RunningService<RoleClient, ()>);
unsafe impl Send for HttpService {}
unsafe impl Sync for HttpService {}

impl McpManager {
    /// Connect to all MCP servers described in `config`, discover their
    /// tools, and return `(McpManager, Vec<Box<dyn ToolDyn>>)`.
    ///
    /// Servers that fail to connect are logged and skipped — a partial
    /// failure does not prevent the session from starting.
    pub async fn connect(config: &McpConfig) -> (Self, Vec<Box<dyn ToolDyn>>) {
        let mut all_tools: Vec<Box<dyn ToolDyn>> = Vec::new();
        let mut services: Vec<Box<dyn Send + Sync>> = Vec::new();

        for (name, server_config) in &config.mcp_servers {
            match connect_server(name, server_config).await {
                Ok((service, tools)) => {
                    let tool_count = tools.len();
                    for t in tools {
                        all_tools.push(Box::new(t) as Box<dyn ToolDyn>);
                    }
                    services.push(service);
                    info!(server = %name, tool_count, "Connected MCP server");
                }
                Err(e) => {
                    warn!(server = %name, error = %e, "Failed to connect MCP server, skipping");
                }
            }
        }

        (Self { _services: services }, all_tools)
    }

    pub fn empty() -> Self {
        Self { _services: Vec::new() }
    }
}

async fn connect_server(
    name: &str,
    config: &McpServerConfig,
) -> Result<(Box<dyn Send + Sync>, Vec<McpTool>)> {
    if let Some(ref command) = config.command {
        connect_stdio(name, command, config).await
    } else if let Some(ref url) = config.url {
        connect_http(name, url).await
    } else {
        anyhow::bail!("MCP server '{name}' has neither 'command' nor 'url'");
    }
}

async fn connect_stdio(
    name: &str,
    command: &str,
    config: &McpServerConfig,
) -> Result<(Box<dyn Send + Sync>, Vec<McpTool>)> {
    let mut cmd = tokio::process::Command::new(command);
    if let Some(ref args) = config.args {
        cmd.args(args);
    }
    if let Some(ref env) = config.env {
        for (k, v) in env {
            cmd.env(k, v);
        }
    }

    let transport = TokioChildProcess::new(cmd)
        .with_context(|| format!("Failed to spawn stdio MCP server '{name}'"))?;

    let service = ().serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize MCP server '{name}': {e}"))?;

    let tools = discover_tools(&service, name).await?;
    Ok((Box::new(StdioService(service)), tools))
}

async fn connect_http(
    name: &str,
    url: &str,
) -> Result<(Box<dyn Send + Sync>, Vec<McpTool>)> {
    let transport = StreamableHttpClientTransport::from_uri(url);

    let service = ().serve(transport)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize HTTP MCP server '{name}': {e}"))?;

    let tools = discover_tools(&service, name).await?;
    Ok((Box::new(HttpService(service)), tools))
}

async fn discover_tools(
    service: &RunningService<RoleClient, ()>,
    name: &str,
) -> Result<Vec<McpTool>> {
    let raw_tools = service.peer().list_all_tools()
        .await
        .with_context(|| format!("Failed to list tools from MCP server '{name}'"))?;

    let sink = service.peer().clone();
    let tools: Vec<McpTool> = raw_tools
        .into_iter()
        .map(|def| McpTool::from_mcp_server(def, sink.clone()))
        .collect();

    Ok(tools)
}

pub async fn load_and_connect() -> (McpManager, Vec<Box<dyn ToolDyn>>) {
    let config = shared::mcp_store::load();
    if config.mcp_servers.is_empty() {
        return (McpManager::empty(), Vec::new());
    }
    McpManager::connect(&config).await
}

pub fn mcp_tool_descriptions(tools: &[Box<dyn ToolDyn>], builtin_count: usize) -> Vec<String> {
    tools.iter().skip(builtin_count).map(|t| {
        let name = t.name();
        format!("{name}: MCP tool")
    }).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_empty_manager() {
        let manager = McpManager::empty();
        assert_eq!(manager._services.len(), 0);
    }

    #[tokio::test]
    async fn test_connect_empty_config() {
        let config = McpConfig {
            mcp_servers: HashMap::new(),
        };
        let (manager, tools) = McpManager::connect(&config).await;
        assert_eq!(manager._services.len(), 0);
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_connect_invalid_command_skips() {
        let mut servers = HashMap::new();
        servers.insert(
            "bad".to_string(),
            McpServerConfig {
                command: Some("__nonexistent_mcp_binary_12345__".to_string()),
                args: None,
                env: None,
                url: None,
            },
        );
        let config = McpConfig { mcp_servers: servers };
        let (manager, tools) = McpManager::connect(&config).await;
        assert_eq!(manager._services.len(), 0);
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_load_and_connect_no_file() {
        let (manager, tools) = load_and_connect().await;
        assert_eq!(manager._services.len(), 0);
        assert!(tools.is_empty());
    }

    #[test]
    fn test_mcp_tool_descriptions_empty() {
        let tools: Vec<Box<dyn ToolDyn>> = Vec::new();
        let descs = mcp_tool_descriptions(&tools, 0);
        assert!(descs.is_empty());
    }
}
