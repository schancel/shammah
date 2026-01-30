// Comprehensive routing test - exercises all patterns and features

use anyhow::Result;
use shammah::crisis::CrisisDetector;
use shammah::patterns::{PatternLibrary, PatternMatcher};
use shammah::router::{RouteDecision, Router};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Shammah Phase 1 Comprehensive Routing Test ===\n");

    // Load components
    let patterns_path = std::path::PathBuf::from("data/patterns.json");
    let pattern_library = PatternLibrary::load_from_file(&patterns_path)?;
    let pattern_matcher = PatternMatcher::new(pattern_library.clone(), 0.2);

    let crisis_path = std::path::PathBuf::from("data/crisis_keywords.json");
    let crisis_detector = CrisisDetector::load_from_file(&crisis_path)?;

    let router = Router::new(pattern_matcher, crisis_detector);

    // Test queries organized by expected behavior
    let test_cases = vec![
        ("PATTERN TESTS", vec![
            ("What is the golden rule?", Some("reciprocity")),
            ("How should I treat others?", Some("reciprocity")),
            ("Why do lies require more lies?", Some("deception-compounding")),
            ("What happens when you lie?", Some("deception-compounding")),
            ("Can harsh judgment backfire?", Some("judgment-rebound")),
            ("How does trauma affect people?", Some("trauma-patterns")),
            ("What is PTSD?", Some("trauma-patterns")),
            ("Explain systemic oppression", Some("systemic-oppression")),
            ("What are coordination problems?", Some("coordination-failure")),
            ("Tell me about path dependence", Some("path-dependence")),
        ]),
        ("CRISIS DETECTION TESTS", vec![
            ("I'm thinking about suicide", None),
            ("I want to kill myself", None),
            ("I'm going to hurt people", None),
            ("I'm being abused", None),
        ]),
        ("FORWARD TESTS (No Match)", vec![
            ("How do I implement quicksort in Rust?", None),
            ("What's the weather like today?", None),
            ("Explain quantum computing", None),
            ("Write me a poem about trees", None),
        ]),
    ];

    let mut stats = Stats::default();

    for (category, queries) in test_cases {
        println!("## {}\n", category);

        for (query, expected_pattern) in queries {
            let decision = router.route(query);

            match decision {
                RouteDecision::Local { pattern, confidence } => {
                    let status = if expected_pattern == Some(pattern.id.as_str()) {
                        stats.correct_local += 1;
                        "✓"
                    } else {
                        stats.incorrect_local += 1;
                        "✗"
                    };
                    println!(
                        "{} LOCAL: {} → {} ({:.2})",
                        status, query, pattern.id, confidence
                    );
                }
                RouteDecision::Forward { reason } => {
                    if category == "CRISIS DETECTION TESTS" {
                        stats.correct_forward += 1;
                        println!("✓ CRISIS: {} → {:?}", query, reason.as_str());
                    } else if category == "FORWARD TESTS (No Match)" {
                        stats.correct_forward += 1;
                        println!("✓ FORWARD: {} → {:?}", query, reason.as_str());
                    } else {
                        stats.incorrect_forward += 1;
                        println!("✗ FORWARD: {} → {:?} (expected local)", query, reason.as_str());
                    }
                }
            }
        }
        println!();
    }

    // Print summary
    println!("=== SUMMARY ===\n");
    let total = stats.total();
    let correct = stats.correct();
    let accuracy = if total > 0 {
        (correct as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("Total queries: {}", total);
    println!("Correct routing: {} ({:.1}%)", correct, accuracy);
    println!("  - Correct local: {}", stats.correct_local);
    println!("  - Correct forward: {}", stats.correct_forward);
    println!("Incorrect routing: {}", stats.incorrect_local + stats.incorrect_forward);
    println!("  - Should be local: {}", stats.incorrect_forward);
    println!("  - Should be forward: {}", stats.incorrect_local);

    println!("\n=== PHASE 1 GOALS ===");
    let local_rate = if total > 0 {
        (stats.correct_local as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    println!("Local rate: {:.1}% (target: 20-30%)", local_rate);
    println!("Crisis detection: {}% (target: 100%)",
        if stats.correct_forward >= 4 { 100 } else { 0 });

    Ok(())
}

#[derive(Default)]
struct Stats {
    correct_local: usize,
    incorrect_local: usize,
    correct_forward: usize,
    incorrect_forward: usize,
}

impl Stats {
    fn total(&self) -> usize {
        self.correct_local + self.incorrect_local + self.correct_forward + self.incorrect_forward
    }

    fn correct(&self) -> usize {
        self.correct_local + self.correct_forward
    }
}
