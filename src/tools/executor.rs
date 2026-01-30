// Tool execution engine
//
// Executes tools with permission checks and multi-turn support

use crate::cli::ConversationHistory;
use crate::tools::permissions::{PermissionCheck, PermissionManager};
use crate::tools::registry::ToolRegistry;
use crate::tools::types::{ToolContext, ToolResult, ToolUse};
use anyhow::{Context, Result};
use std::collections::HashSet;
use tracing::{debug, error, info, instrument};

/// Signature for a tool execution, used for caching approval decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolSignature {
    pub tool_name: String,
    pub context_key: String,
}

/// Session-level cache for tool execution approvals
pub struct ToolConfirmationCache {
    approved: HashSet<ToolSignature>,
}

impl ToolConfirmationCache {
    pub fn new() -> Self {
        Self {
            approved: HashSet::new(),
        }
    }

    pub fn is_approved(&self, sig: &ToolSignature) -> bool {
        self.approved.contains(sig)
    }

    pub fn approve(&mut self, sig: ToolSignature) {
        self.approved.insert(sig);
    }

    pub fn clear(&mut self) {
        self.approved.clear();
    }
}

/// Tool executor - manages tool execution lifecycle
pub struct ToolExecutor {
    registry: ToolRegistry,
    permissions: PermissionManager,
    confirmation_cache: ToolConfirmationCache,
}

impl ToolExecutor {
    /// Create new tool executor
    pub fn new(registry: ToolRegistry, permissions: PermissionManager) -> Self {
        Self {
            registry,
            permissions,
            confirmation_cache: ToolConfirmationCache::new(),
        }
    }

    /// Check if a tool signature is pre-approved
    pub fn is_pre_approved(&self, sig: &ToolSignature) -> bool {
        self.confirmation_cache.is_approved(sig)
    }

    /// Add a tool signature to the approval cache
    pub fn add_approval(&mut self, sig: ToolSignature) {
        self.confirmation_cache.approve(sig);
    }

    /// Clear the approval cache (called on session end)
    pub fn clear_approvals(&mut self) {
        self.confirmation_cache.clear();
    }

    /// Execute a single tool use
    #[instrument(skip(self, tool_use, conversation, save_models_fn), fields(tool = %tool_use.name, id = %tool_use.id))]
    pub async fn execute_tool<F>(
        &self,
        tool_use: &ToolUse,
        conversation: Option<&ConversationHistory>,
        save_models_fn: Option<F>,
    ) -> Result<ToolResult>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        info!("Executing tool: {}", tool_use.name);

        // 1. Check if tool exists
        let tool = self
            .registry
            .get(&tool_use.name)
            .context(format!("Tool '{}' not found", tool_use.name))?;

        // 2. Check permissions
        let permission_check = self
            .permissions
            .check_tool_use(&tool_use.name, &tool_use.input);

        match permission_check {
            PermissionCheck::Allow => {
                debug!("Tool execution allowed");
            }
            PermissionCheck::AskUser(reason) => {
                // For now, deny if ask required (will implement user prompts in Phase 2)
                error!("Tool execution requires user confirmation: {}", reason);
                return Ok(ToolResult::error(
                    tool_use.id.clone(),
                    format!("Permission required: {}", reason),
                ));
            }
            PermissionCheck::Deny(reason) => {
                error!("Tool execution denied: {}", reason);
                return Ok(ToolResult::error(tool_use.id.clone(), reason));
            }
        }

        // 3. Create context
        let context = ToolContext {
            conversation,
            save_models: save_models_fn
                .map(|f| Box::new(f) as Box<dyn Fn() -> Result<()> + Send + Sync>),
        };

        // 4. Execute tool
        match tool.execute(tool_use.input.clone(), &context).await {
            Ok(output) => {
                info!("Tool executed successfully");
                Ok(ToolResult::success(tool_use.id.clone(), output))
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Ok(ToolResult::error(
                    tool_use.id.clone(),
                    format!("Execution error: {}", e),
                ))
            }
        }
    }

    /// Execute multiple tool uses in sequence
    #[instrument(skip(self, tool_uses, conversation, save_models_fn))]
    pub async fn execute_tool_loop<F>(
        &self,
        tool_uses: Vec<ToolUse>,
        conversation: Option<&ConversationHistory>,
        save_models_fn: Option<F>,
    ) -> Result<Vec<ToolResult>>
    where
        F: Fn() -> Result<()> + Send + Sync + Clone,
    {
        info!("Executing {} tool(s)", tool_uses.len());

        let mut results = Vec::new();

        for tool_use in tool_uses {
            let result = self
                .execute_tool(&tool_use, conversation, save_models_fn.clone())
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get reference to registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Get reference to permissions manager
    pub fn permissions(&self) -> &PermissionManager {
        &self.permissions
    }
}

/// Generate a context-specific signature for a tool use
pub fn generate_tool_signature(tool_use: &ToolUse, working_dir: &std::path::Path) -> ToolSignature {
    match tool_use.name.as_str() {
        "bash" => {
            let command = tool_use.input["command"].as_str().unwrap_or("");
            ToolSignature {
                tool_name: "bash".to_string(),
                context_key: format!("{} in {}", command, working_dir.display()),
            }
        }
        "read" => {
            let file_path = tool_use.input["file_path"].as_str().unwrap_or("");
            ToolSignature {
                tool_name: "read".to_string(),
                context_key: format!("reading {}", file_path),
            }
        }
        "glob" => {
            let pattern = tool_use.input["pattern"].as_str().unwrap_or("");
            ToolSignature {
                tool_name: "glob".to_string(),
                context_key: format!("pattern {}", pattern),
            }
        }
        "grep" => {
            let pattern = tool_use.input["pattern"].as_str().unwrap_or("");
            let path = tool_use
                .input
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            ToolSignature {
                tool_name: "grep".to_string(),
                context_key: format!("pattern '{}' in {}", pattern, path),
            }
        }
        "web_fetch" => {
            let url = tool_use.input["url"].as_str().unwrap_or("");
            ToolSignature {
                tool_name: "web_fetch".to_string(),
                context_key: format!("fetching {}", url),
            }
        }
        "save_and_exec" => {
            let command = tool_use.input["command"].as_str().unwrap_or("");
            ToolSignature {
                tool_name: "save_and_exec".to_string(),
                context_key: format!("{} in {}", command, working_dir.display()),
            }
        }
        _ => {
            // Generic signature for unknown tools
            ToolSignature {
                tool_name: tool_use.name.clone(),
                context_key: format!("in {}", working_dir.display()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::registry::Tool;
    use crate::tools::types::ToolInputSchema;
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::path::Path;

    // Mock tool for testing
    struct MockTool {
        should_fail: bool,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock"
        }

        fn description(&self) -> &str {
            "A mock tool"
        }

        fn input_schema(&self) -> ToolInputSchema {
            ToolInputSchema::simple(vec![("param", "Test parameter")])
        }

        async fn execute(&self, input: Value, _context: &ToolContext<'_>) -> Result<String> {
            if self.should_fail {
                anyhow::bail!("Mock failure");
            }
            Ok(format!("Mock result: {}", input))
        }
    }

    fn create_test_executor(allow_tool: bool, tool_should_fail: bool) -> ToolExecutor {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(MockTool {
            should_fail: tool_should_fail,
        }));

        let permissions = if allow_tool {
            PermissionManager::new()
                .with_default_rule(crate::tools::permissions::PermissionRule::Allow)
        } else {
            PermissionManager::new()
                .with_default_rule(crate::tools::permissions::PermissionRule::Deny)
        };

        ToolExecutor::new(registry, permissions)
    }

    #[tokio::test]
    async fn test_execute_tool_success() {
        let executor = create_test_executor(true, false);
        let tool_use = ToolUse::new("mock".to_string(), serde_json::json!({"param": "value"}));

        let result = executor
            .execute_tool(&tool_use, None, None::<fn() -> Result<()>>)
            .await
            .unwrap();

        assert_eq!(result.tool_use_id, tool_use.id);
        assert!(!result.is_error);
        assert!(result.content.contains("Mock result"));
    }

    #[tokio::test]
    async fn test_execute_tool_not_found() {
        let executor = create_test_executor(true, false);
        let tool_use = ToolUse::new(
            "nonexistent".to_string(),
            serde_json::json!({"param": "value"}),
        );

        let result = executor
            .execute_tool(&tool_use, None, None::<fn() -> Result<()>>)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_execute_tool_permission_denied() {
        let executor = create_test_executor(false, false);
        let tool_use = ToolUse::new("mock".to_string(), serde_json::json!({"param": "value"}));

        let result = executor
            .execute_tool(&tool_use, None, None::<fn() -> Result<()>>)
            .await
            .unwrap();

        assert_eq!(result.tool_use_id, tool_use.id);
        assert!(result.is_error);
        assert!(result.content.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_execute_tool_execution_failure() {
        let executor = create_test_executor(true, true);
        let tool_use = ToolUse::new("mock".to_string(), serde_json::json!({"param": "value"}));

        let result = executor
            .execute_tool(&tool_use, None, None::<fn() -> Result<()>>)
            .await
            .unwrap();

        assert_eq!(result.tool_use_id, tool_use.id);
        assert!(result.is_error);
        assert!(result.content.contains("Execution error"));
    }

    #[tokio::test]
    async fn test_execute_tool_loop() {
        let executor = create_test_executor(true, false);
        let tool_uses = vec![
            ToolUse::new("mock".to_string(), serde_json::json!({"param": "1"})),
            ToolUse::new("mock".to_string(), serde_json::json!({"param": "2"})),
        ];

        let results = executor
            .execute_tool_loop(tool_uses, None, None::<fn() -> Result<()>>)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(!results[0].is_error);
        assert!(!results[1].is_error);
    }

    #[test]
    fn test_confirmation_cache() {
        let mut cache = ToolConfirmationCache::new();

        let sig1 = ToolSignature {
            tool_name: "bash".to_string(),
            context_key: "cargo test".to_string(),
        };

        let sig2 = ToolSignature {
            tool_name: "bash".to_string(),
            context_key: "cargo build".to_string(),
        };

        // Initially, nothing is approved
        assert!(!cache.is_approved(&sig1));
        assert!(!cache.is_approved(&sig2));

        // Approve sig1
        cache.approve(sig1.clone());
        assert!(cache.is_approved(&sig1));
        assert!(!cache.is_approved(&sig2));

        // Approve sig2
        cache.approve(sig2.clone());
        assert!(cache.is_approved(&sig1));
        assert!(cache.is_approved(&sig2));

        // Clear cache
        cache.clear();
        assert!(!cache.is_approved(&sig1));
        assert!(!cache.is_approved(&sig2));
    }

    #[test]
    fn test_tool_executor_approval_cache() {
        let mut executor = create_test_executor(true, false);

        let sig = ToolSignature {
            tool_name: "bash".to_string(),
            context_key: "cargo fmt".to_string(),
        };

        // Initially not approved
        assert!(!executor.is_pre_approved(&sig));

        // Add approval
        executor.add_approval(sig.clone());
        assert!(executor.is_pre_approved(&sig));

        // Clear approvals
        executor.clear_approvals();
        assert!(!executor.is_pre_approved(&sig));
    }

    #[test]
    fn test_generate_tool_signature_bash() {
        let working_dir = Path::new("/test/dir");
        let tool_use = ToolUse::new(
            "bash".to_string(),
            json!({
                "command": "cargo test",
                "description": "Run tests"
            }),
        );

        let sig = generate_tool_signature(&tool_use, working_dir);

        assert_eq!(sig.tool_name, "bash");
        assert_eq!(sig.context_key, "cargo test in /test/dir");
    }

    #[test]
    fn test_generate_tool_signature_read() {
        let working_dir = Path::new("/test/dir");
        let tool_use = ToolUse::new(
            "read".to_string(),
            json!({"file_path": "/path/to/file.txt"}),
        );

        let sig = generate_tool_signature(&tool_use, working_dir);

        assert_eq!(sig.tool_name, "read");
        assert_eq!(sig.context_key, "reading /path/to/file.txt");
    }

    #[test]
    fn test_generate_tool_signature_grep() {
        let working_dir = Path::new("/test/dir");
        let tool_use = ToolUse::new(
            "grep".to_string(),
            json!({
                "pattern": "fn main",
                "path": "src/"
            }),
        );

        let sig = generate_tool_signature(&tool_use, working_dir);

        assert_eq!(sig.tool_name, "grep");
        assert_eq!(sig.context_key, "pattern 'fn main' in src/");
    }

    #[test]
    fn test_tool_signature_uniqueness() {
        let working_dir = Path::new("/test/dir");

        let cmd1 = ToolUse::new("bash".to_string(), json!({"command": "cargo test"}));
        let cmd2 = ToolUse::new("bash".to_string(), json!({"command": "cargo build"}));
        let cmd3 = ToolUse::new("bash".to_string(), json!({"command": "cargo test"}));

        let sig1 = generate_tool_signature(&cmd1, working_dir);
        let sig2 = generate_tool_signature(&cmd2, working_dir);
        let sig3 = generate_tool_signature(&cmd3, working_dir);

        // Different commands should have different signatures
        assert_ne!(sig1, sig2);

        // Same command should produce same signature
        assert_eq!(sig1, sig3);
    }
}
