// Shammah - Local-first Constitutional AI Proxy
// Library exports

// Core modules
pub mod claude;
pub mod cli;
pub mod client; // HTTP client for daemon communication (Phase 8)
pub mod config;
pub mod crisis;
pub mod daemon; // Daemon lifecycle and auto-spawn (Phase 8)
pub mod generators; // Unified generator interface
pub mod local; // Local generation system
pub mod metrics;
pub mod models; // Phase 2: Neural network models
pub mod providers; // Multi-provider LLM support
pub mod router;
pub mod server; // HTTP daemon mode (Phase 1)
pub mod tools; // Tool execution system
pub mod training; // Batch training and checkpoints (Phase 2)
