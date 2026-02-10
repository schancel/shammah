// Unified request/response types for multi-provider LLM support
//
// These types abstract over provider-specific formats (Claude, OpenAI, Gemini, etc.)
// allowing the rest of the codebase to work with a unified interface.

use crate::claude::types::{ContentBlock, Message};
use crate::tools::types::ToolDefinition;
use serde::{Deserialize, Serialize};

/// Unified request format for all LLM providers
///
/// This wraps the existing Message format and adds provider-agnostic options.
/// Each provider implementation will transform this into their specific API format.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderRequest {
    /// Conversation messages (using Claude's Message format as the common denominator)
    pub messages: Vec<Message>,

    /// Model name (provider-specific)
    pub model: String,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Tool definitions (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Temperature (0.0 to 1.0, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Whether to stream the response
    #[serde(skip)]
    pub stream: bool,
}

impl ProviderRequest {
    /// Create a new request from messages
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            model: String::new(), // Will be set by provider
            max_tokens: 4096,
            tools: None,
            temperature: None,
            stream: false,
        }
    }

    /// Set the model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Add tools to the request
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Enable streaming
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// Unified response format from LLM providers
///
/// This wraps the provider-specific response in a common format.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderResponse {
    /// Response ID (provider-specific)
    pub id: String,

    /// Model that generated the response
    pub model: String,

    /// Content blocks (text, tool_use, etc.)
    pub content: Vec<ContentBlock>,

    /// Why the model stopped generating
    pub stop_reason: Option<String>,

    /// Role of the responder (usually "assistant")
    pub role: String,

    /// Provider name (e.g., "claude", "openai", "gemini")
    pub provider: String,
}

impl ProviderResponse {
    /// Extract text from the response
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if response contains tool uses
    pub fn has_tool_uses(&self) -> bool {
        self.content.iter().any(|block| block.is_tool_use())
    }

    /// Extract tool uses from response
    pub fn tool_uses(&self) -> Vec<crate::tools::types::ToolUse> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => Some(crate::tools::types::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    /// Convert to Message for conversation history
    pub fn to_message(&self) -> Message {
        Message {
            role: self.role.clone(),
            content: self.content.clone(),
        }
    }
}

/// Stream chunk types for streaming responses
///
/// Re-export from generators module for convenience
pub use crate::generators::StreamChunk;
