// TUI Renderer - Ratatui-based terminal user interface
//
// This module provides a Claude Code-like TUI with:
// - Scrollable output area (top)
// - Fixed input line (middle)
// - Multi-line status area (bottom)

use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;

use super::{OutputManager, StatusBar};

mod output_widget;
mod status_widget;

pub use output_widget::OutputWidget;
pub use status_widget::StatusWidget;

/// TUI renderer for Ratatui-based interface
pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    output_manager: OutputManager,
    status_bar: StatusBar,
    /// Whether TUI is currently active (for suspend/resume)
    is_active: bool,
}

impl TuiRenderer {
    /// Create a new TUI renderer
    pub fn new(output_manager: OutputManager, status_bar: StatusBar) -> Result<Self> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        Ok(Self {
            terminal,
            output_manager,
            status_bar,
            is_active: true,
        })
    }

    /// Render the TUI
    pub fn render(&mut self) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        self.terminal
            .draw(|frame| {
                // Define layout: output area, input line, status area
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),   // Output area (expandable)
                        Constraint::Length(1), // Input line (fixed)
                        Constraint::Length(3), // Status area (3 lines max)
                    ])
                    .split(frame.size());

                // Render output area
                let output_widget = OutputWidget::new(&self.output_manager);
                frame.render_widget(output_widget, chunks[0]);

                // TODO: Input line will be rendered here in Phase 3
                // For now, just leave it blank

                // Render status area
                let status_widget = StatusWidget::new(&self.status_bar);
                frame.render_widget(status_widget, chunks[2]);
            })
            .context("Failed to draw frame")?;

        Ok(())
    }

    /// Suspend the TUI (for showing inquire menus or other non-TUI content)
    pub fn suspend(&mut self) -> Result<()> {
        if !self.is_active {
            return Ok(());
        }

        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .context("Failed to leave alternate screen")?;

        self.is_active = false;
        Ok(())
    }

    /// Resume the TUI (after showing non-TUI content)
    pub fn resume(&mut self) -> Result<()> {
        if self.is_active {
            return Ok(());
        }

        enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )
        .context("Failed to enter alternate screen")?;

        self.is_active = true;

        // Redraw immediately after resuming
        self.render()?;

        Ok(())
    }

    /// Check if the TUI is currently active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Shutdown the TUI and restore terminal state
    pub fn shutdown(mut self) -> Result<()> {
        if self.is_active {
            disable_raw_mode().context("Failed to disable raw mode")?;
            execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .context("Failed to leave alternate screen")?;
        }
        Ok(())
    }
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        // Ensure terminal is restored on drop
        if self.is_active {
            let _ = disable_raw_mode();
            let _ = execute!(
                self.terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            );
        }
    }
}
