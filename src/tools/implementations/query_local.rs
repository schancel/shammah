// QueryLocalModelTool - Let Claude see Shammah's responses directly

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::tools::registry::Tool;
use crate::tools::types::{ToolContext, ToolInputSchema};

/// Tool that queries the local generator (Shammah) directly
pub struct QueryLocalModelTool;

#[async_trait]
impl Tool for QueryLocalModelTool {
    fn name(&self) -> &str {
        "query_local_model"
    }

    fn description(&self) -> &str {
        "Query Shammah (the local LLM) directly and see its response.

Use this tool to:
- Test Shammah's capabilities on specific queries
- See what mistakes or errors Shammah is making
- Compare Shammah's response quality to Claude's
- Identify areas where Shammah needs more training

Input: {\"query\": \"your test query here\"}

Returns: Shammah's raw response plus quality metrics including:
- Response text
- Quality score (0.0-1.0)
- Uncertainty level
- Coherence check
- On-topic check
- Hallucination risk assessment"
    }

    fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "query": {
                    "type": "string",
                    "description": "The query to send to Shammah"
                }
            }),
            required: vec!["query".to_string()],
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext<'_>) -> Result<String> {
        let query = input["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' field"))?;

        // TODO: Implement actual local generation
        // For now, return a placeholder response

        // In the real implementation, this would:
        // 1. Tokenize the query
        // 2. Run through generator model
        // 3. Run validator to get quality metrics
        // 4. Return formatted results

        let response = format!(
            "=== Shammah's Response ===\n\
             [Local generator not yet fully implemented]\n\
             Query: {}\n\n\
             === Quality Metrics ===\n\
             - Quality Score: 0.00/1.0 (not yet trained)\n\
             - Uncertainty: 1.00 (very uncertain)\n\
             - Status: Shammah needs training data to produce responses\n\n\
             To train Shammah, use the GenerateTrainingDataTool to create \
             targeted training examples, then use the TrainTool to train \
             on those examples.",
            query
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_local_tool() {
        let tool = QueryLocalModelTool;
        let input = serde_json::json!({"query": "What is 2+2?"});

        // Create minimal context for testing
        let ctx = ToolContext {
            cwd: std::path::PathBuf::from("."),
            allow_all: true,
        };

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Shammah's Response"));
        assert!(response.contains("Quality Metrics"));
    }
}
