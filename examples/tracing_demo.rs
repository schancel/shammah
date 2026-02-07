// Example demonstrating tracing integration (Phase 3.5 Part 2)
//
// This example shows how tracing logs from dependencies and our code
// are captured and routed through the OutputManager instead of printing
// directly to stdout/stderr.
//
// Run with: cargo run --example tracing_demo
// Run with debug: SHAMMAH_DEBUG=1 cargo run --example tracing_demo
// Run with custom log level: RUST_LOG=debug cargo run --example tracing_demo

use shammah::cli::output_layer::OutputManagerLayer;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::prelude::*;

fn init_tracing_demo() {
    // Check if debug logging should be enabled
    let show_debug = std::env::var("SHAMMAH_DEBUG")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    println!("Debug logging: {}", show_debug);

    // Create our custom output layer
    let output_layer = if show_debug {
        OutputManagerLayer::with_debug()
    } else {
        OutputManagerLayer::new()
    };

    // Create environment filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Build the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(output_layer)
        .init();

    // Bridge log crate → tracing (do this after subscriber is set up)
    tracing_log::LogTracer::init().ok();
}

fn main() {
    println!("=== Tracing Integration Demo ===\n");

    // Initialize tracing with our custom layer
    init_tracing_demo();

    // Simulate various log levels from our code
    info!("Application started");
    info!("Loading configuration...");

    // Simulate warning
    warn!("Configuration file not found, using defaults");

    // Simulate progress messages
    info!("Downloading model from HuggingFace...");
    info!("Loading model weights...");

    // Debug messages (only shown with SHAMMAH_DEBUG=1 or RUST_LOG=debug)
    debug!("This is a debug message");
    trace!("This is a trace message");

    // Simulate error
    error!("Failed to connect to service: connection refused");

    // Simulate logs from different modules
    info!(target: "shammah::models::loader", "Model loaded successfully");
    info!(target: "tokio::runtime", "Starting worker thread");
    info!(target: "reqwest::client", "HTTP request sent");
    info!(target: "hf_hub::download", "Downloading file chunk");

    println!("\n=== Checking Buffer ===");

    // In interactive mode, check the global buffer
    if !shammah::cli::global_output::is_non_interactive() {
        let output = shammah::cli::global_output::global_output();
        let messages = output.get_messages();
        println!("Total messages captured: {}", messages.len());

        if messages.is_empty() {
            println!("Note: Messages might be in status buffer instead");
            // Messages go to status in most cases
        }
    } else {
        println!("Running in non-interactive mode");
        println!("Logs are captured but silent unless SHAMMAH_LOG=1");
    }

    println!("\n=== Demo Complete ===");
    println!("\nTry running:");
    println!("  cargo run --example tracing_demo");
    println!("  SHAMMAH_DEBUG=1 cargo run --example tracing_demo");
    println!("  RUST_LOG=debug cargo run --example tracing_demo");
    println!("  SHAMMAH_LOG=1 cargo run --example tracing_demo 2>&1 | grep STATUS");
}
