// TUI Input Handler - Coordinates rustyline with Ratatui
//
// This module provides an async wrapper around rustyline that works
// with the TUI renderer by suspending/resuming the TUI during input.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::TuiRenderer;
use crate::cli::InputHandler;

/// Async input handler that coordinates with TUI
pub struct TuiInputHandler {
    /// Underlying rustyline handler (wrapped in Arc<RwLock> for async access)
    input_handler: Arc<RwLock<InputHandler>>,
    /// Optional TUI renderer (for suspend/resume)
    tui_renderer: Arc<RwLock<Option<TuiRenderer>>>,
}

impl TuiInputHandler {
    /// Create a new TUI input handler
    pub fn new(
        input_handler: InputHandler,
        tui_renderer: Arc<RwLock<Option<TuiRenderer>>>,
    ) -> Self {
        Self {
            input_handler: Arc::new(RwLock::new(input_handler)),
            tui_renderer,
        }
    }

    /// Read a line of input asynchronously
    ///
    /// This will:
    /// 1. Suspend the TUI (leave raw mode)
    /// 2. Show the rustyline prompt
    /// 3. Read user input
    /// 4. Resume the TUI (re-enter raw mode)
    pub async fn read_line(&mut self, prompt: &str) -> Result<Option<String>> {
        // Suspend TUI before reading input
        self.suspend_tui().await?;

        // Read input using rustyline
        // Note: This is blocking, but acceptable in CLI context
        let result = {
            let mut handler = self.input_handler.write().await;
            handler.read_line(prompt)?
        };

        // Resume TUI after reading input
        self.resume_tui().await?;

        Ok(result)
    }

    /// Save history to disk
    pub async fn save_history(&self) -> Result<()> {
        let mut handler = self.input_handler.write().await;
        handler.save_history()
    }

    /// Suspend the TUI (internal)
    async fn suspend_tui(&self) -> Result<()> {
        let mut tui_guard = self.tui_renderer.write().await;
        if let Some(tui) = tui_guard.as_mut() {
            tui.suspend().context("Failed to suspend TUI")?;
        }
        Ok(())
    }

    /// Resume the TUI (internal)
    async fn resume_tui(&self) -> Result<()> {
        let mut tui_guard = self.tui_renderer.write().await;
        if let Some(tui) = tui_guard.as_mut() {
            tui.resume().context("Failed to resume TUI")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handler_creation() {
        // Create a mock input handler
        let input_handler = InputHandler::new().unwrap();
        let tui_renderer = Arc::new(RwLock::new(None));

        let handler = TuiInputHandler::new(input_handler, tui_renderer);
        // Just verify it creates without panic
        assert!(true);
    }

    #[tokio::test]
    async fn test_suspend_resume_without_tui() {
        let input_handler = InputHandler::new().unwrap();
        let tui_renderer = Arc::new(RwLock::new(None));

        let handler = TuiInputHandler::new(input_handler, tui_renderer);

        // Should not fail even without TUI
        assert!(handler.suspend_tui().await.is_ok());
        assert!(handler.resume_tui().await.is_ok());
    }
}
