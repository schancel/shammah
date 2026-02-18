// MCP client coordinator - manages multiple server connections
//
// NOTE: This is a simplified stub implementation until the rust-mcp-sdk
// API stabilizes or we implement JSON-RPC directly.

use super::config::McpServerConfig;
use super::connection::McpConnection;
use crate::tools::types::{ToolDefinition, ToolInputSchema};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP client that manages multiple server connections (stub implementation)
pub struct McpClient {
    /// Active server connections (name -> connection)
    connections: Arc<RwLock<HashMap<String, Arc<RwLock<McpConnection>>>>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Connect to MCP servers from configuration (stub)
    pub async fn from_config(servers: &HashMap<String, McpServerConfig>) -> Result<Self> {
        let client = Self::new();

        for (name, config) in servers {
            if !config.enabled {
                tracing::debug!("Skipping disabled MCP server '{}'", name);
                continue;
            }

            match McpConnection::connect(name.clone(), config).await {
                Ok(conn) => {
                    client
                        .connections
                        .write()
                        .await
                        .insert(name.clone(), Arc::new(RwLock::new(conn)));
                    tracing::info!("Created MCP connection for server: {} (stub)", name);
                }
                Err(e) => {
                    tracing::warn!("Failed to create MCP connection for '{}': {}", name, e);
                    // Continue with other servers
                }
            }
        }

        Ok(client)
    }

    /// List all available tools from all connected servers (stub)
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let connections = self.connections.read().await;
        let mut tools = Vec::new();

        for (server_name, conn) in connections.iter() {
            let conn = conn.read().await;
            let server_tools = conn.list_tools();

            for tool in server_tools {
                // Convert MCP tool to our ToolDefinition format
                // Prefix tool name with "mcp_<server>_" to avoid conflicts
                let prefixed_name = format!("mcp_{}_{}", server_name, tool.name);

                tools.push(ToolDefinition {
                    name: prefixed_name,
                    description: tool
                        .description
                        .clone()
                        .unwrap_or_else(|| format!("Tool from MCP server '{}'", server_name)),
                    input_schema: ToolInputSchema {
                        schema_type: "object".to_string(),
                        properties: serde_json::json!({}), // TODO: Convert from MCP schema format
                        required: Vec::new(),
                    },
                });
            }
        }

        tools
    }

    /// Execute a tool on the appropriate server (stub)
    pub async fn execute_tool(&self, tool_name: &str, params: Value) -> Result<String> {
        // Parse prefixed name: "mcp_<server>_<tool>"
        let parts: Vec<&str> = tool_name.split('_').collect();
        if parts.len() < 3 || parts[0] != "mcp" {
            anyhow::bail!("Invalid MCP tool name: {}", tool_name);
        }

        let server_name = parts[1];
        let actual_tool_name = parts[2..].join("_");

        tracing::debug!(
            "Executing MCP tool '{}' on server '{}' (stub)",
            actual_tool_name,
            server_name
        );

        // Find the connection
        let connections = self.connections.read().await;
        let conn = connections
            .get(server_name)
            .with_context(|| format!("MCP server '{}' not found", server_name))?;

        // Call the tool
        let conn = conn.read().await;
        conn.call_tool(&actual_tool_name, params)
            .await
            .context("Failed to execute MCP tool")
    }

    /// Refresh tools from all servers (stub)
    pub async fn refresh_all_tools(&self) -> Result<()> {
        let connections = self.connections.read().await;

        for (name, conn) in connections.iter() {
            let mut conn = conn.write().await;
            if let Err(e) = conn.refresh_tools().await {
                tracing::warn!("Failed to refresh tools for MCP server '{}': {}", name, e);
            }
        }

        Ok(())
    }

    /// Get list of connected server names
    pub async fn list_servers(&self) -> Vec<String> {
        self.connections
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Check if a server is connected
    pub async fn is_connected(&self, name: &str) -> bool {
        self.connections.read().await.contains_key(name)
    }

    /// Disconnect from a specific server (stub)
    pub async fn disconnect(&self, name: &str) -> Result<()> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.remove(name) {
            let conn = conn.read().await;
            conn.shutdown()
                .await
                .context("Failed to shutdown server")?;
            tracing::info!("Disconnected from MCP server: {}", name);
        }

        Ok(())
    }

    /// Disconnect from all servers (stub)
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut connections = self.connections.write().await;
        let names: Vec<_> = connections.keys().cloned().collect();

        for name in names {
            if let Some(conn) = connections.remove(&name) {
                let conn = conn.read().await;
                if let Err(e) = conn.shutdown().await {
                    tracing::warn!("Failed to shutdown MCP server '{}': {}", name, e);
                }
            }
        }

        tracing::info!("Disconnected from all MCP servers");
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        tracing::debug!("Dropping MCP client");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::mcp::TransportType;

    #[tokio::test]
    async fn test_client_creation() {
        let client = McpClient::new();
        let servers = client.list_servers().await;
        assert_eq!(servers.len(), 0);
    }

    #[tokio::test]
    async fn test_from_config_empty() {
        let config: HashMap<String, McpServerConfig> = HashMap::new();
        let client = McpClient::from_config(&config).await.unwrap();
        assert_eq!(client.list_servers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_from_config_with_servers() {
        let mut config = HashMap::new();
        config.insert(
            "test1".to_string(),
            McpServerConfig {
                transport: TransportType::Stdio,
                command: Some("test".to_string()),
                args: vec![],
                env: HashMap::new(),
                url: None,
                enabled: true,
            },
        );
        config.insert(
            "test2".to_string(),
            McpServerConfig {
                transport: TransportType::Stdio,
                command: Some("test2".to_string()),
                args: vec![],
                env: HashMap::new(),
                url: None,
                enabled: false, // Disabled
            },
        );

        let client = McpClient::from_config(&config).await.unwrap();
        let servers = client.list_servers().await;

        // Only enabled server should be connected
        assert_eq!(servers.len(), 1);
        assert!(servers.contains(&"test1".to_string()));
    }

    #[tokio::test]
    async fn test_disconnect() {
        let mut config = HashMap::new();
        config.insert(
            "test".to_string(),
            McpServerConfig {
                transport: TransportType::Stdio,
                command: Some("test".to_string()),
                args: vec![],
                env: HashMap::new(),
                url: None,
                enabled: true,
            },
        );

        let client = McpClient::from_config(&config).await.unwrap();
        assert!(client.is_connected("test").await);

        client.disconnect("test").await.unwrap();
        assert!(!client.is_connected("test").await);
    }
}
