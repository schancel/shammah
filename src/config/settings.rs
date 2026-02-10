// Configuration structs

use super::backend::BackendConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    /// Claude API key
    pub api_key: String,

    /// Path to crisis_keywords.json
    pub crisis_keywords_path: PathBuf,

    /// Directory for metrics storage
    pub metrics_dir: PathBuf,

    /// Enable streaming responses (default: true)
    pub streaming_enabled: bool,

    /// Enable TUI (Ratatui-based interface) (default: false for Phase 2)
    pub tui_enabled: bool,

    /// Path to constitutional guidelines for local LLM (optional)
    /// Only used for local inference, NOT sent to Claude API
    pub constitution_path: Option<PathBuf>,

    /// Backend configuration (device selection, model paths)
    pub backend: BackendConfig,

    /// Server configuration (daemon mode)
    pub server: ServerConfig,
}

/// Server configuration for daemon mode
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Enable daemon mode
    pub enabled: bool,
    /// Bind address (e.g., "127.0.0.1:8000")
    pub bind_address: String,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Session timeout in minutes
    pub session_timeout_minutes: u64,
    /// Enable API key authentication
    pub auth_enabled: bool,
    /// Valid API keys for authentication
    pub api_keys: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: "127.0.0.1:8000".to_string(),
            max_sessions: 100,
            session_timeout_minutes: 30,
            auth_enabled: false,
            api_keys: vec![],
        }
    }
}

impl Config {
    pub fn new(api_key: String) -> Self {
        let home = dirs::home_dir().expect("Could not determine home directory");
        let project_dir = std::env::current_dir().expect("Could not determine current directory");

        // Look for constitution in ~/.shammah/constitution.md
        let constitution_path = home.join(".shammah/constitution.md");
        let constitution_path = if constitution_path.exists() {
            Some(constitution_path)
        } else {
            None
        };

        Self {
            api_key,
            crisis_keywords_path: project_dir.join("data/crisis_keywords.json"),
            metrics_dir: home.join(".shammah/metrics"),
            streaming_enabled: true, // Enable by default
            tui_enabled: true,       // TUI is the default for interactive terminals
            constitution_path,
            backend: BackendConfig::default(),
            server: ServerConfig::default(),
        }
    }

    /// Save configuration to TOML file at ~/.shammah/config.toml
    pub fn save(&self) -> anyhow::Result<()> {
        use std::fs;

        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let config_dir = home.join(".shammah");
        let config_path = config_dir.join("config.toml");

        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        // Create serializable config
        let toml_config = TomlConfig {
            api_key: self.api_key.clone(),
            streaming_enabled: self.streaming_enabled,
            backend: self.backend.clone(),
        };

        let toml_string = toml::to_string_pretty(&toml_config)?;
        fs::write(&config_path, toml_string)?;

        tracing::info!("Configuration saved to {:?}", config_path);
        Ok(())
    }
}

/// TOML-serializable config (subset of Config)
#[derive(serde::Serialize, serde::Deserialize)]
struct TomlConfig {
    api_key: String,
    streaming_enabled: bool,
    backend: BackendConfig,
}
