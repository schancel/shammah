// GenerateTrainingDataTool - Claude creates targeted training examples for Shammah

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

use crate::tools::registry::Tool;
use crate::tools::types::{ToolContext, ToolInputSchema};

/// Tool that generates synthetic training data for Shammah
pub struct GenerateTrainingDataTool;

#[async_trait]
impl Tool for GenerateTrainingDataTool {
    fn name(&self) -> &str {
        "generate_training_data"
    }

    fn description(&self) -> &str {
        "Generate synthetic training examples to improve Shammah's capabilities.

Claude (you) can create targeted training data to teach Shammah specific skills:
- Generate diverse examples for a category (math, code, science, etc.)
- Cover different difficulty levels
- Include edge cases and variations
- Provide high-quality responses for each example

Input: {
  \"category\": \"math\" | \"code\" | \"science\" | \"general\" | etc.,
  \"count\": 10-100,
  \"difficulty\": \"easy\" | \"medium\" | \"hard\",
  \"focus\": \"optional specific focus area\"
}

Process:
1. Claude generates N diverse queries in the category
2. Claude provides high-quality responses for each
3. Examples are added to training queue
4. User can trigger training with TrainTool

This enables:
- Rapid skill acquisition (hours vs months)
- Targeted weakness improvement
- Curriculum learning (easy -> hard)
- Active learning (Claude identifies gaps and fills them)"
    }

    fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchema {
            schema_type: "object".to_string(),
            properties: serde_json::json!({
                "category": {
                    "type": "string",
                    "description": "Category of training examples (math, code, science, general, etc.)",
                    "enum": ["math", "code", "science", "history", "general", "reasoning", "creative"]
                },
                "count": {
                    "type": "integer",
                    "description": "Number of examples to generate (10-100)",
                    "minimum": 10,
                    "maximum": 100
                },
                "difficulty": {
                    "type": "string",
                    "description": "Difficulty level",
                    "enum": ["easy", "medium", "hard"]
                },
                "focus": {
                    "type": "string",
                    "description": "Optional specific focus area within category"
                }
            }),
            required: vec!["category".to_string(), "count".to_string(), "difficulty".to_string()],
        }
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext<'_>) -> Result<String> {
        let category = input["category"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'category' field"))?;

        let count = input["count"]
            .as_i64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'count' field"))?;

        let difficulty = input["difficulty"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'difficulty' field"))?;

        let focus = input["focus"].as_str();

        // TODO: Implement actual training data generation
        // For now, return instructions

        // In the real implementation, this would:
        // 1. Claude generates N diverse queries
        // 2. Claude provides high-quality responses
        // 3. Add to training queue
        // 4. Return summary

        let response = format!(
            "=== Training Data Generation Request ===\n\
             Category: {}\n\
             Count: {} examples\n\
             Difficulty: {}\n\
             {}\n\n\
             === Instructions ===\n\
             To generate training data:\n\n\
             1. Create {} diverse {} queries at {} difficulty\n\
             2. For each query, provide your (Claude's) high-quality response\n\
             3. Format as: Query | Response pairs\n\
             4. I will add these to Shammah's training queue\n\n\
             Example format:\n\
             Q: [Your generated question]\n\
             A: [Your high-quality answer]\n\n\
             Please generate the {} {} examples now, and I'll process them \
             into training data.",
            category,
            count,
            difficulty,
            focus.map(|f| format!("Focus: {}", f)).unwrap_or_default(),
            count,
            category,
            difficulty,
            count,
            category
        );

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_training_tool() {
        let tool = GenerateTrainingDataTool;
        let input = serde_json::json!({
            "category": "math",
            "count": 20,
            "difficulty": "medium",
            "focus": "algebra"
        });

        let ctx = ToolContext {
            cwd: std::path::PathBuf::from("."),
            allow_all: true,
        };

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("Training Data Generation"));
        assert!(response.contains("math"));
        assert!(response.contains("20"));
    }
}
