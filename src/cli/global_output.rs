// Global output system for TUI
//
// Provides global access to OutputManager and StatusBar via macros.
// This allows any code (including background tasks and dependencies)
// to write to the output buffer without passing references around.
//
// Non-interactive Mode Behavior:
// - output_claude!() prints to stdout (actual model output)
// - output_status!() is silent unless SHAMMAH_LOG=1
// - Other macros write to buffer (for potential logging)

use once_cell::sync::Lazy;
use std::io::{self, IsTerminal};
use std::sync::{Arc, Mutex};

use super::{OutputManager, StatusBar};
use super::tui::TuiRenderer;

/// Global singleton OutputManager
pub static GLOBAL_OUTPUT: Lazy<Arc<OutputManager>> = Lazy::new(|| Arc::new(OutputManager::new()));

/// Global singleton StatusBar
pub static GLOBAL_STATUS: Lazy<Arc<StatusBar>> = Lazy::new(|| Arc::new(StatusBar::new()));

/// Global singleton TUI renderer (optional, set when TUI mode is enabled)
pub static GLOBAL_TUI_RENDERER: Lazy<Mutex<Option<TuiRenderer>>> = Lazy::new(|| Mutex::new(None));

/// Get reference to global OutputManager
pub fn global_output() -> &'static Arc<OutputManager> {
    &GLOBAL_OUTPUT
}

/// Get reference to global StatusBar
pub fn global_status() -> &'static Arc<StatusBar> {
    &GLOBAL_STATUS
}

/// Set the global TUI renderer (called when TUI mode is enabled)
pub fn set_global_tui_renderer(renderer: TuiRenderer) {
    *GLOBAL_TUI_RENDERER.lock().unwrap() = Some(renderer);
}

/// Get a reference to the global TUI renderer
pub fn get_global_tui_renderer() -> &'static Mutex<Option<TuiRenderer>> {
    &GLOBAL_TUI_RENDERER
}

/// Shutdown the global TUI renderer and restore terminal state
pub fn shutdown_global_tui() -> anyhow::Result<()> {
    use std::time::Duration;

    eprintln!("[DEBUG] shutdown_global_tui: Starting...");

    // Try to acquire lock with timeout to prevent indefinite hang
    // Use a loop with try_lock to implement timeout behavior
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(500);

    loop {
        match GLOBAL_TUI_RENDERER.try_lock() {
            Ok(mut tui_lock) => {
                eprintln!("[DEBUG] shutdown_global_tui: Lock acquired");
                if let Some(tui) = tui_lock.take() {
                    eprintln!("[DEBUG] shutdown_global_tui: Calling tui.shutdown()...");
                    tui.shutdown()?;
                    eprintln!("[DEBUG] shutdown_global_tui: tui.shutdown() completed");
                } else {
                    eprintln!("[DEBUG] shutdown_global_tui: No TUI to shutdown");
                }
                return Ok(());
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                if start.elapsed() > timeout {
                    // Timeout - force cleanup without taking the lock
                    eprintln!("[DEBUG] shutdown_global_tui: Lock timeout - forcing emergency cleanup");

                    // Emergency terminal cleanup
                    use crossterm::{cursor, execute, terminal};
                    let _ = terminal::disable_raw_mode();
                    let _ = execute!(
                        std::io::stdout(),
                        cursor::Show,
                        terminal::Clear(terminal::ClearType::FromCursorDown)
                    );

                    return Ok(());
                }
                // Wait a bit and try again
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(std::sync::TryLockError::Poisoned(_)) => {
                // Mutex was poisoned - do emergency cleanup
                eprintln!("[DEBUG] shutdown_global_tui: Mutex poisoned - forcing emergency cleanup");

                use crossterm::{cursor, execute, terminal};
                let _ = terminal::disable_raw_mode();
                let _ = execute!(
                    std::io::stdout(),
                    cursor::Show,
                    terminal::Clear(terminal::ClearType::FromCursorDown)
                );

                return Ok(());
            }
        }
    }
}

/// Check if we're in non-interactive mode (stdout is not a TTY)
pub fn is_non_interactive() -> bool {
    !io::stdout().is_terminal()
}

/// Check if SHAMMAH_LOG environment variable is set
pub fn logging_enabled() -> bool {
    std::env::var("SHAMMAH_LOG")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Output a user message (query/input)
#[macro_export]
macro_rules! output_user {
    ($($arg:tt)*) => {{
        $crate::cli::global_output::global_output().write_user(format!($($arg)*));
    }};
}

/// Output a Claude response
/// In non-interactive mode (piped output), prints directly to stdout
/// In interactive mode (TUI), writes to buffer
#[macro_export]
macro_rules! output_claude {
    ($($arg:tt)*) => {{
        let content = format!($($arg)*);
        if $crate::cli::global_output::is_non_interactive() {
            // Non-interactive mode: print to stdout
            use std::io::Write;
            let _ = writeln!(std::io::stdout(), "{}", content);
        } else {
            // Interactive mode: write to buffer for TUI
            $crate::cli::global_output::global_output().write_claude(content);
        }
    }};
}

/// Append to the last Claude response (for streaming)
#[macro_export]
macro_rules! output_claude_append {
    ($($arg:tt)*) => {{
        $crate::cli::global_output::global_output().append_claude(format!($($arg)*));
    }};
}

/// Output tool execution result
#[macro_export]
macro_rules! output_tool {
    ($tool:expr, $($arg:tt)*) => {{
        $crate::cli::global_output::global_output().write_tool($tool, format!($($arg)*));
    }};
}

/// Output status information
/// In non-interactive mode, only prints if SHAMMAH_LOG=1
/// In interactive mode, writes to buffer for TUI
#[macro_export]
macro_rules! output_status {
    ($($arg:tt)*) => {{
        let content = format!($($arg)*);
        if $crate::cli::global_output::is_non_interactive() {
            // Non-interactive mode: only print if logging enabled
            if $crate::cli::global_output::logging_enabled() {
                eprintln!("[STATUS] {}", content);
            }
        } else {
            // Interactive mode: write to buffer for TUI
            $crate::cli::global_output::global_output().write_status(content);
        }
    }};
}

/// Output error message
/// In non-interactive mode, prints to stderr if SHAMMAH_LOG=1
/// In interactive mode, writes to buffer for TUI
#[macro_export]
macro_rules! output_error {
    ($($arg:tt)*) => {{
        let content = format!($($arg)*);
        if $crate::cli::global_output::is_non_interactive() {
            // Non-interactive mode: print to stderr if logging enabled
            if $crate::cli::global_output::logging_enabled() {
                eprintln!("[ERROR] {}", content);
            }
        } else {
            // Interactive mode: write to buffer for TUI
            $crate::cli::global_output::global_output().write_error(content);
        }
    }};
}

/// Output progress update
/// In non-interactive mode, only prints if SHAMMAH_LOG=1
/// In interactive mode, writes to buffer for TUI
#[macro_export]
macro_rules! output_progress {
    ($($arg:tt)*) => {{
        let content = format!($($arg)*);
        if $crate::cli::global_output::is_non_interactive() {
            // Non-interactive mode: only print if logging enabled
            if $crate::cli::global_output::logging_enabled() {
                eprintln!("[PROGRESS] {}", content);
            }
        } else {
            // Interactive mode: write to buffer for TUI
            $crate::cli::global_output::global_output().write_progress(content);
        }
    }};
}

// Status bar macros

/// Update training statistics
/// In non-interactive mode, only prints if SHAMMAH_LOG=1
#[macro_export]
macro_rules! status_training {
    ($queries:expr, $local_pct:expr, $quality:expr) => {{
        if $crate::cli::global_output::is_non_interactive() {
            if $crate::cli::global_output::logging_enabled() {
                eprintln!(
                    "[STATUS] Training: {} queries | Local: {:.0}% | Quality: {:.2}",
                    $queries, $local_pct * 100.0, $quality
                );
            }
        } else {
            $crate::cli::global_output::global_status()
                .update_training_stats($queries, $local_pct, $quality);
        }
    }};
}

/// Update download progress
/// In non-interactive mode, only prints if SHAMMAH_LOG=1
#[macro_export]
macro_rules! status_download {
    ($name:expr, $pct:expr, $downloaded:expr, $total:expr) => {{
        if $crate::cli::global_output::is_non_interactive() {
            if $crate::cli::global_output::logging_enabled() {
                eprintln!(
                    "[STATUS] Downloading {}: {:.0}% ({}/{})",
                    $name, $pct * 100.0, $downloaded, $total
                );
            }
        } else {
            $crate::cli::global_output::global_status()
                .update_download_progress($name, $pct, $downloaded, $total);
        }
    }};
}

/// Update operation status
/// In non-interactive mode, only prints if SHAMMAH_LOG=1
#[macro_export]
macro_rules! status_operation {
    ($($arg:tt)*) => {{
        let content = format!($($arg)*);
        if $crate::cli::global_output::is_non_interactive() {
            if $crate::cli::global_output::logging_enabled() {
                eprintln!("[STATUS] {}", content);
            }
        } else {
            $crate::cli::global_output::global_status().update_operation(content);
        }
    }};
}

/// Clear operation status
#[macro_export]
macro_rules! status_clear_operation {
    () => {{
        $crate::cli::global_output::global_status().clear_operation();
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_access() {
        let output = global_output();
        output.write_user("Test");
        assert_eq!(output.len(), 1);
    }

    #[test]
    fn test_macros() {
        // Clear any previous test data
        global_output().clear();

        output_user!("Hello");
        output_claude!("Response");
        output_status!("Status message");

        assert_eq!(global_output().len(), 3);
    }

    #[test]
    fn test_status_macros() {
        status_training!(10, 0.5, 0.8);
        status_operation!("Testing");

        let lines = global_status().get_lines();
        assert_eq!(lines.len(), 2);
    }
}
