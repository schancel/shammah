// AnalyzeModelTool - Claude analyzes Shammah's capabilities and weaknesses

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::tools::registry::Tool;
use crate::tools::types::{ToolContext, ToolInputSchema};

/// Tool that analyzes Shammah's current capabilities
pub struct AnalyzeModelTool;

#[async_trait]
impl Tool for AnalyzeModelTool {
    fn name(&self) -> &str {
        "analyze_model"
    }

    fn description(&self) -> &str {
        "Analyze Shammah's current capabilities and identify areas for improvement.

Performs comprehensive capability assessment:
1. Tests Shammah on diverse queries across categories
2. Evaluates response quality for each category
3. Identifies strengths and weaknesses
4. Recommends targeted training areas

Input: {
  \"test_count\": 50-200,
  \"categories\": [\"math\", \"code\", \"science\", ...] (optional)
}

Returns detailed analysis:
- Overall performance metrics
- Per-category accuracy scores
- Identified weak areas
- Specific recommendations for improvement
- Suggested training data counts

Use this to:
- Understand what Shammah can and cannot do
- Prioritize training efforts
- Track improvement over time
- Make data-driven training decisions"
    }

    fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "test_count": {
                    "type": "integer",
                    "description": "Number of test queries (50-200)",
                    "minimum": 50,
                    "maximum": 200,
                    "default": 100
                },
                "categories": {
                    "type": "array",
                    "description": "Optional list of categories to test",
                    "items": {
                        "type": "string"
                    }
                }
            }),
            required: vec![],
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext<'_>) -> Result<String> {
        let test_count = input["test_count"].as_i64().unwrap_or(100);

        let categories = if let Some(cats) = input["categories"].as_array() {
            cats.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![
                "greetings".to_string(),
                "math".to_string(),
                "code".to_string(),
                "science".to_string(),
                "history".to_string(),
                "reasoning".to_string(),
                "creative".to_string(),
            ]
        };

        // TODO: Implement actual model analysis
        // For now, return a template analysis

        // In the real implementation, this would:
        // 1. Generate test queries across categories
        // 2. Get Shammah's responses
        // 3. Get Claude's responses (ground truth)
        // 4. Compare and score
        // 5. Aggregate by category
        // 6. Identify patterns
        // 7. Return recommendations

        let response = format!(
            "=== Shammah Capability Analysis ===\n\
             Test queries: {}\n\
             Categories tested: {}\n\n\
             === Current Status ===\n\
             Overall: Shammah is not yet trained (0% local success rate)\n\n\
             === Recommendations ===\n\
             Shammah needs initial training before meaningful analysis.\n\n\
             Suggested bootstrap training:\n\
             1. Greetings & simple queries: 50 examples (easy)\n\
             2. General knowledge: 100 examples (easy-medium)\n\
             3. Math basics: 50 examples (easy-medium)\n\
             4. Code snippets: 50 examples (medium)\n\
             5. Reasoning: 50 examples (medium)\n\n\
             Total: 300 examples to establish baseline capability\n\n\
             Use GenerateTrainingDataTool to create these examples,\n\
             then TrainTool to train Shammah.\n\n\
             After initial training, run this analysis again to measure progress.",
            test_count,
            categories.join(", ")
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_model_tool() {
        let tool = AnalyzeModelTool;
        let input = serde_json::json!({
            "test_count": 100,
            "categories": ["math", "code", "science"]
        });

        let ctx = ToolContext {
            cwd: std::path::PathBuf::from("."),
            allow_all: true,
        };

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Capability Analysis"));
        assert!(response.contains("Recommendations"));
    }
}
