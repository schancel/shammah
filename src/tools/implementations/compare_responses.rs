// CompareResponsesTool - Side-by-side comparison of Shammah vs Claude

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::tools::registry::Tool;
use crate::tools::types::{ToolContext, ToolInputSchema};

/// Tool that compares Shammah's response to Claude's response
pub struct CompareResponsesTool;

#[async_trait]
impl Tool for CompareResponsesTool {
    fn name(&self) -> &str {
        "compare_responses"
    }

    fn description(&self) -> &str {
        "Compare Shammah's response to Claude's response for the same query.

Shows both responses side-by-side with:
- Full text of each response
- Quality scores
- Similarity/divergence metrics
- Analysis of differences

Use this to:
- Understand where Shammah differs from Claude
- Identify if Shammah's response is acceptable
- Find patterns in Shammah's mistakes
- Decide if more training is needed

Input: {\"query\": \"your test query\"}

Returns: Side-by-side comparison with similarity score and verdict"
    }

    fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "query": {
                    "type": "string",
                    "description": "The query to test both models on"
                }
            }),
            required: vec!["query".to_string()],
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext<'_>) -> Result<String> {
        let query = input["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' field"))?;

        // TODO: Implement actual comparison
        // For now, return a placeholder response

        // In the real implementation, this would:
        // 1. Generate Shammah's response
        // 2. Forward same query to Claude
        // 3. Compute semantic similarity
        // 4. Analyze differences
        // 5. Return formatted comparison

        let response = format!(
            "=== Query ===\n\
             {}\n\n\
             === Shammah's Response (Local) ===\n\
             [Local generator not yet fully implemented]\n\
             Quality: 0.00/1.0\n\n\
             === Claude's Response (API) ===\n\
             [Would forward to Claude API here]\n\n\
             === Comparison ===\n\
             - Similarity: N/A (Shammah not trained)\n\
             - Divergence: N/A\n\
             - Verdict: ✗ Shammah needs training before comparisons are meaningful\n\n\
             Recommendation: Use GenerateTrainingDataTool to create training examples,\n\
             then train Shammah before comparing responses.",
            query
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compare_responses_tool() {
        let tool = CompareResponsesTool;
        let input = serde_json::json!({"query": "Explain photosynthesis"});

        let ctx = ToolContext {
            cwd: std::path::PathBuf::from("."),
            allow_all: true,
        };

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Query"));
        assert!(response.contains("Shammah's Response"));
        assert!(response.contains("Claude's Response"));
        assert!(response.contains("Comparison"));
    }
}
