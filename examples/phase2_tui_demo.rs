// Phase 2 Demo: Ratatui TUI Rendering
//
// Run with: cargo run --example phase2_tui_demo

use shammah::cli::tui::TuiRenderer;
use shammah::cli::{OutputManager, StatusBar};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phase 2: Ratatui TUI Demo ===");
    println!("This will launch the TUI in 2 seconds...");
    println!("Press Ctrl+C to exit\n");

    thread::sleep(Duration::from_secs(2));

    // Create output manager and status bar
    let output_mgr = OutputManager::new();
    let status_bar = StatusBar::new();

    // Add some initial messages
    output_mgr.write_user("Hello, Shammah!");
    output_mgr.write_claude("Hi! How can I help you today?");
    output_mgr.write_tool("read", "Reading file: src/main.rs");
    output_mgr.write_tool("read", "✓ File read successfully (234 lines)");
    output_mgr
        .write_claude("Based on the code, I can see you're building a Rust-based AI assistant...");
    output_mgr.write_status("Processing query...");

    // Add status lines
    status_bar.update_training_stats(42, 0.38, 0.82);
    status_bar.update_download_progress("Qwen-2.5-3B", 0.65, 1_690_000_000, 2_600_000_000);
    status_bar.update_operation("Tool execution: read");

    // Initialize TUI
    let mut tui = TuiRenderer::new(output_mgr.clone(), status_bar.clone())?;

    // Render loop
    for i in 0..50 {
        // Render the TUI
        tui.render()?;

        // Simulate streaming response
        if i % 5 == 0 {
            output_mgr.append_claude(" Adding");
            output_mgr.append_claude(" more");
            output_mgr.append_claude(" content...");
        }

        // Update download progress
        if i % 3 == 0 {
            let progress = 0.65 + (i as f64 * 0.007);
            let downloaded = (progress * 2_600_000_000.0) as u64;
            status_bar.update_download_progress("Qwen-2.5-3B", progress, downloaded, 2_600_000_000);
        }

        // Add new messages occasionally
        if i == 10 {
            output_mgr.write_user("What's the best way to handle errors in Rust?");
        }
        if i == 15 {
            output_mgr.write_claude(
                "In Rust, the best practice for error handling is to use Result<T, E>...",
            );
        }
        if i == 20 {
            output_mgr.write_tool("grep", "Searching for 'Result' in codebase...");
            status_bar.update_operation("Tool execution: grep");
        }
        if i == 25 {
            output_mgr.write_tool("grep", "✓ Found 247 matches");
            status_bar.clear_operation();
        }
        if i == 30 {
            output_mgr.write_progress("Training model: epoch 3/10 (loss: 0.245)");
        }
        if i == 40 {
            output_mgr.write_error("Warning: Low confidence (0.45)");
        }

        thread::sleep(Duration::from_millis(200));
    }

    // Final render
    tui.render()?;
    thread::sleep(Duration::from_secs(2));

    // Shutdown cleanly
    tui.shutdown()?;

    println!("\n=== Phase 2 Demo Complete ===");
    println!("\nWhat was demonstrated:");
    println!("  ✓ TUI layout with 3 sections (output, input, status)");
    println!("  ✓ Colored message types (user, claude, tool, status, error, progress)");
    println!("  ✓ Multi-line status bar at bottom");
    println!("  ✓ Dynamic status updates (download progress, operation status)");
    println!("  ✓ Streaming text append");
    println!("  ✓ Clean shutdown and terminal restoration");
    println!("\nNext: Phase 3 - Integrate input handling with rustyline");

    Ok(())
}
