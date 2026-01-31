// Shammah - Local-first Constitutional AI Proxy
// Main entry point

use anyhow::Result;
use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

use shammah::claude::ClaudeClient;
use shammah::cli::{ConversationHistory, Repl};
use shammah::config::load_config;
use shammah::crisis::CrisisDetector;
use shammah::metrics::MetricsLogger;
use shammah::models::ThresholdRouter;
use shammah::router::Router;

#[derive(Parser, Debug)]
#[command(name = "shammah")]
#[command(about = "Local-first Constitutional AI Proxy", version)]
struct Args {
    /// Initial prompt to send after startup
    #[arg(long = "initial-prompt")]
    initial_prompt: Option<String>,

    /// Path to session state file to restore
    #[arg(long = "restore-session")]
    restore_session: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Check for piped input BEFORE initializing anything else
    if !io::stdin().is_terminal() {
        // Piped input mode: read query from stdin and process as single query
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        // Skip processing if input is empty
        if input.trim().is_empty() {
            return Ok(());
        }

        // Initialize minimal components for piped mode
        let config = load_config()?;
        let crisis_detector = CrisisDetector::load_from_file(&config.crisis_keywords_path)?;

        let models_dir = dirs::home_dir()
            .map(|home| home.join(".shammah").join("models"))
            .expect("Failed to determine home directory");
        std::fs::create_dir_all(&models_dir)?;

        let threshold_router_path = models_dir.join("threshold_router.json");
        let threshold_router = if threshold_router_path.exists() {
            ThresholdRouter::load(&threshold_router_path).unwrap_or_else(|_| ThresholdRouter::new())
        } else {
            ThresholdRouter::new()
        };

        let router = Router::new(crisis_detector, threshold_router);
        let claude_client = ClaudeClient::new(config.api_key.clone())?;
        let metrics_logger = MetricsLogger::new(config.metrics_dir.clone())?;

        // Create REPL (will detect non-interactive mode automatically)
        let mut repl = Repl::new(config, claude_client, router, metrics_logger);

        // Process the piped query and exit
        let response = repl.process_query(input.trim()).await?;
        println!("{}", response);

        return Ok(());
    }

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = load_config()?;

    // Load crisis detector
    let crisis_detector = CrisisDetector::load_from_file(&config.crisis_keywords_path)?;

    // Load or create threshold router
    let models_dir = dirs::home_dir()
        .map(|home| home.join(".shammah").join("models"))
        .expect("Failed to determine home directory");
    std::fs::create_dir_all(&models_dir)?;

    let threshold_router_path = models_dir.join("threshold_router.json");
    let threshold_router = if threshold_router_path.exists() {
        match ThresholdRouter::load(&threshold_router_path) {
            Ok(router) => {
                eprintln!(
                    "✓ Loaded threshold router with {} queries",
                    router.stats().total_queries
                );
                router
            }
            Err(e) => {
                eprintln!("Warning: Failed to load threshold router: {}", e);
                eprintln!("  Creating new threshold router");
                ThresholdRouter::new()
            }
        }
    } else {
        eprintln!("Creating new threshold router");
        ThresholdRouter::new()
    };

    // Create router with threshold router
    let router = Router::new(crisis_detector, threshold_router);

    // Create Claude client
    let claude_client = ClaudeClient::new(config.api_key.clone())?;

    // Create metrics logger
    let metrics_logger = MetricsLogger::new(config.metrics_dir.clone())?;

    // Create and run REPL
    let mut repl = Repl::new(config, claude_client, router, metrics_logger);

    // Restore session if requested
    if let Some(session_path) = args.restore_session {
        if session_path.exists() {
            match ConversationHistory::load(&session_path) {
                Ok(history) => {
                    repl.restore_conversation(history);
                    eprintln!("✓ Restored conversation from session");
                    std::fs::remove_file(&session_path)?;
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to restore session: {}", e);
                }
            }
        }
    }

    // Run REPL (potentially with initial prompt)
    repl.run_with_initial_prompt(args.initial_prompt).await?;

    Ok(())
}
