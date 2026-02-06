// Progressive Bootstrap Demo
// Demonstrates instant startup with background model loading

use shammah::models::{BootstrapLoader, DevicePreference, GeneratorState, QwenSize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Progressive Bootstrap Demo ===\n");

    // Step 1: Instant startup (simulating REPL initialization)
    let startup_time = Instant::now();
    println!("Step 1: REPL Startup");
    println!("--------------------");
    println!("Initializing REPL components...");

    // Create shared generator state
    let generator_state = Arc::new(RwLock::new(GeneratorState::Initializing));

    // Simulate other REPL initialization (config, Claude client, router, etc.)
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let startup_elapsed = startup_time.elapsed();
    println!(
        "✓ REPL ready in {:.0}ms\n",
        startup_elapsed.as_secs_f64() * 1000.0
    );

    // Step 2: Start background model loading
    println!("Step 2: Background Model Loading");
    println!("----------------------------------");

    let state_clone = Arc::clone(&generator_state);
    let loader = BootstrapLoader::new(state_clone);

    // Spawn background task for model loading
    let load_task = {
        let loader_clone = loader;
        tokio::spawn(async move {
            println!("Background task: Starting model load...");

            // For demo, use smallest model with manual override
            let result = loader_clone
                .load_generator_async(
                    Some(QwenSize::Qwen1_5B), // Override to smallest model
                    DevicePreference::Auto,
                )
                .await;

            match result {
                Ok(_) => println!("\nBackground task: ✓ Model loaded successfully"),
                Err(e) => {
                    println!("\nBackground task: ✗ Load failed: {}", e);
                    loader_clone.handle_error(e).await;
                }
            }
        })
    };

    // Step 3: Simulate user queries while model loads
    println!("\nStep 3: User Queries (Model Still Loading)");
    println!("-------------------------------------------");

    for i in 1..=5 {
        // Check generator state
        let state = generator_state.read().await;

        println!("\nQuery #{}: \"How do I use Rust lifetimes?\"", i);
        println!("  Generator state: {}", state.status_message());

        if state.is_ready() {
            println!("  ✓ Using local generator");
            break;
        } else {
            println!("  → Forwarding to Claude API (graceful degradation)");
        }

        drop(state); // Release lock

        // Simulate query processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Step 4: Wait for model to finish loading (if not already)
    println!("\nStep 4: Waiting for Model Load to Complete");
    println!("-------------------------------------------");

    // Wait for background task
    load_task.await?;

    // Check final state
    let final_state = generator_state.read().await;
    println!("\nFinal state: {}", final_state.status_message());

    if final_state.is_ready() {
        println!("✓ All future queries will use local generation");
    } else {
        println!("⚠ Model not available - will continue forwarding to Claude");
    }

    // Step 5: Summary
    println!("\n=== Summary ===");
    println!("\nProgressive Bootstrap Benefits:");
    println!("✓ REPL available in <100ms (vs 2-5 seconds with synchronous loading)");
    println!("✓ User can start querying immediately");
    println!("✓ Graceful degradation (forwards to Claude while model loads)");
    println!("✓ Background download with progress tracking (first run)");
    println!("✓ Seamless transition to local generation when ready");

    println!("\nUser Experience:");
    println!("  $ shammah");
    println!("  > How do I use lifetimes in Rust?");
    println!("  ⏳ Downloading Qwen-2.5-3B (first time only)...");
    println!("  [=====>    ] 45% (2.1GB / 4.7GB)");
    println!("  ");
    println!("  [Response from Claude while downloading...]");
    println!("  ");
    println!("  ✓ Model ready - future queries will use local generation");

    println!("\n=== Demo Complete ===");

    Ok(())
}
