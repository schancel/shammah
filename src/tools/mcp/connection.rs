// MCP connection wrapper for a single server
//
// NOTE: This is a simplified stub implementation until the rust-mcp-sdk
// API stabilizes or we implement JSON-RPC directly.
//
// Current limitation: Can't store ClientRuntime because it's a private type
// in rust-mcp-sdk. Need to either:
// 1. Implement JSON-RPC 2.0 directly (recommended)
// 2. Use type erasure with Box<dyn Any>
// 3. Wait for SDK improvements

use super::config::{McpServerConfig, TransportType};
use anyhow::{Context, Result};
use rust_mcp_sdk::schema::{Implementation, Tool};
use serde_json::Value;

/// A single MCP server connection (stub implementation)
pub struct McpConnection {
    /// Server name
    name: String,

    /// Server configuration
    config: McpServerConfig,

    /// Available tools (cached from discovery)
    tools: Vec<Tool>,

    /// Server info (if connected)
    server_info: Option<Implementation>,

    /// Connection status
    is_connected: bool,
}

impl McpConnection {
    /// Connect to an MCP server (stub - returns unconnected instance)
    pub async fn connect(name: String, config: &McpServerConfig) -> Result<Self> {
        // Validate config
        config
            .validate(&name)
            .context("Invalid MCP server configuration")?;

        // TODO: Implement actual connection when SDK stabilizes or JSON-RPC is implemented
        tracing::warn!(
            "MCP connection for '{}' is a stub - full implementation pending",
            name
        );

        Ok(Self {
            name: name.clone(),
            config: config.clone(),
            tools: Vec::new(),
            server_info: None,
            is_connected: false,
        })
    }

    /// Get the list of available tools
    pub fn list_tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Refresh the list of available tools (stub)
    pub async fn refresh_tools(&mut self) -> Result<()> {
        // TODO: Implement tool discovery
        tracing::debug!(
            "Refresh tools for MCP server '{}' (stub)",
            self.name
        );
        Ok(())
    }

    /// Call a tool on this server (stub)
    pub async fn call_tool(&self, tool_name: &str, _arguments: Value) -> Result<String> {
        // TODO: Implement tool execution
        anyhow::bail!(
            "MCP tool execution not yet implemented (tried to call '{}' on server '{}')",
            tool_name,
            self.name
        )
    }

    /// Get the server name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get server info
    pub fn server_info(&self) -> Option<&Implementation> {
        self.server_info.as_ref()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Shutdown the connection (stub)
    pub async fn shutdown(&self) -> Result<()> {
        tracing::debug!("Shutdown MCP connection '{}' (stub)", self.name);
        Ok(())
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        tracing::debug!("Dropping MCP connection '{}'", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_connection_creation_stub() {
        let config = McpServerConfig {
            transport: TransportType::Stdio,
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "test-server".to_string()],
            env: HashMap::new(),
            url: None,
            enabled: true,
        };

        let conn = McpConnection::connect("test".to_string(), &config)
            .await
            .unwrap();

        assert_eq!(conn.name(), "test");
        assert!(!conn.is_connected());
        assert_eq!(conn.list_tools().len(), 0);
    }

    #[tokio::test]
    async fn test_connection_invalid_config() {
        let config = McpServerConfig {
            transport: TransportType::Stdio,
            command: None, // Invalid - STDIO needs command
            args: vec![],
            env: HashMap::new(),
            url: None,
            enabled: true,
        };

        let result = McpConnection::connect("test".to_string(), &config).await;
        assert!(result.is_err());
    }
}
