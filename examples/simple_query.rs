// Example: Simple query routing demonstration

use anyhow::Result;

use shammah::models::ThresholdRouter;
use shammah::router::{RouteDecision, Router};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Shammah - Simple Query Example\n");

    // Create threshold router (learns which queries to route locally)
    let threshold_router = ThresholdRouter::new();
    println!("Created threshold router\n");

    // Create router
    let router = Router::new(threshold_router);

    // Test queries
    let test_queries = vec![
        "What is the golden rule?",
        "Why do lies require more lies?",
        "How does trauma affect people?",
        "How do I learn Rust?",
        "Explain async/await in Rust",
    ];

    for query in test_queries {
        println!("Query: {}", query);

        let decision = router.route(query);

        match decision {
            RouteDecision::Local {
                pattern_id,
                confidence,
            } => {
                println!(
                    "  → LOCAL (pattern: {}, confidence: {:.2}) [UNUSED - patterns removed]\n",
                    pattern_id, confidence
                );
            }
            RouteDecision::Forward { reason } => {
                println!("  → FORWARD (reason: {})\n", reason.as_str());
            }
        }
    }

    Ok(())
}
