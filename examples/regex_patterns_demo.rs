/// Demonstration of regex pattern matching in the tool approval system
///
/// This example shows how to use regular expressions for more powerful
/// pattern matching compared to simple wildcards.
///
/// Usage:
///     cargo run --example regex_patterns_demo
use shammah::tools::{
    executor::ToolSignature,
    patterns::{PatternType, PersistentPatternStore, ToolPattern},
};

fn main() {
    println!("=== Regex Pattern Matching Demo ===\n");

    let mut store = PersistentPatternStore::default();

    // Example 1: Match specific cargo commands with regex
    println!("Example 1: Match cargo test or cargo build");
    let regex_pattern = ToolPattern::new_with_type(
        r"^cargo (test|build) in /project$".to_string(),
        "bash".to_string(),
        "Allow cargo test and build in project directory".to_string(),
        PatternType::Regex,
    );

    // Validate the pattern
    match regex_pattern.validate() {
        Ok(_) => println!("✓ Pattern is valid"),
        Err(e) => println!("✗ Pattern is invalid: {}", e),
    }

    let pattern_id = regex_pattern.id.clone();
    store.add_pattern(regex_pattern);

    // Test matching
    let sig1 = ToolSignature {
        tool_name: "bash".to_string(),
        context_key: "cargo test in /project".to_string(),
    };

    let sig2 = ToolSignature {
        tool_name: "bash".to_string(),
        context_key: "cargo build in /project".to_string(),
    };

    let sig3 = ToolSignature {
        tool_name: "bash".to_string(),
        context_key: "cargo run in /project".to_string(),
    };

    println!("\nTest signatures:");
    println!(
        "  'cargo test in /project' -> {}",
        if store.matches(&sig1).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );
    println!(
        "  'cargo build in /project' -> {}",
        if store.matches(&sig2).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );
    println!(
        "  'cargo run in /project' -> {}",
        if store.matches(&sig3).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );

    // Example 2: Match Rust source files with regex
    println!("\n\nExample 2: Match reading Rust source files");
    let file_pattern = ToolPattern::new_with_type(
        r"^reading /project/src/.*\.rs$".to_string(),
        "read".to_string(),
        "Allow reading Rust source files in src directory".to_string(),
        PatternType::Regex,
    );

    store.add_pattern(file_pattern);

    let sig4 = ToolSignature {
        tool_name: "read".to_string(),
        context_key: "reading /project/src/main.rs".to_string(),
    };

    let sig5 = ToolSignature {
        tool_name: "read".to_string(),
        context_key: "reading /project/src/lib.rs".to_string(),
    };

    let sig6 = ToolSignature {
        tool_name: "read".to_string(),
        context_key: "reading /project/src/test.txt".to_string(),
    };

    println!("\nTest signatures:");
    println!(
        "  'reading /project/src/main.rs' -> {}",
        if store.matches(&sig4).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );
    println!(
        "  'reading /project/src/lib.rs' -> {}",
        if store.matches(&sig5).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );
    println!(
        "  'reading /project/src/test.txt' -> {}",
        if store.matches(&sig6).is_some() {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );

    // Example 3: Invalid regex pattern
    println!("\n\nExample 3: Invalid regex pattern");
    let invalid_pattern = ToolPattern::new_with_type(
        r"^test[".to_string(), // Unclosed bracket
        "bash".to_string(),
        "Invalid pattern".to_string(),
        PatternType::Regex,
    );

    match invalid_pattern.validate() {
        Ok(_) => println!("✓ Pattern is valid"),
        Err(e) => println!("✗ Pattern is invalid: {}", e),
    }

    // Example 4: Compare wildcard vs regex
    println!("\n\nExample 4: Wildcard vs Regex comparison");

    let wildcard = ToolPattern::new(
        "cargo * in *".to_string(),
        "bash".to_string(),
        "Wildcard pattern".to_string(),
    );

    let regex = ToolPattern::new_with_type(
        r"^cargo \w+ in /.*$".to_string(),
        "bash".to_string(),
        "Regex pattern".to_string(),
        PatternType::Regex,
    );

    let sig7 = ToolSignature {
        tool_name: "bash".to_string(),
        context_key: "cargo test in /project".to_string(),
    };

    println!("Testing: 'cargo test in /project'");
    println!(
        "  Wildcard pattern 'cargo * in *' -> {}",
        if wildcard.matches(&sig7) {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );
    println!(
        "  Regex pattern '^cargo \\w+ in /.*$' -> {}",
        if regex.matches(&sig7) {
            "✓ Match"
        } else {
            "✗ No match"
        }
    );

    // Show pattern info
    println!("\n\nPattern Statistics:");
    if let Some(pattern) = store.find_by_id_mut(&pattern_id) {
        println!("  Pattern: {}", pattern.pattern);
        println!("  Type: {:?}", pattern.pattern_type);
        println!("  Match count: {}", pattern.match_count);
        println!("  Last used: {:?}", pattern.last_used);
    }
}
