// Daemon lifecycle management
//
// Handles PID file creation/removal, process existence checks,
// and graceful shutdown coordination.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Manages daemon lifecycle (PID file, shutdown)
pub struct DaemonLifecycle {
    pid_file: PathBuf,
}

impl DaemonLifecycle {
    /// Create a new daemon lifecycle manager
    pub fn new() -> Result<Self> {
        let pid_file = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?
            .join(".shammah")
            .join("daemon.pid");

        // Ensure parent directory exists
        if let Some(parent) = pid_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        Ok(Self { pid_file })
    }

    /// Write current process PID to file
    pub fn write_pid(&self) -> Result<()> {
        let pid = std::process::id();
        fs::write(&self.pid_file, pid.to_string())
            .with_context(|| format!("Failed to write PID file: {}", self.pid_file.display()))?;
        info!(pid = pid, path = %self.pid_file.display(), "Daemon PID file written");
        Ok(())
    }

    /// Remove PID file (called on shutdown)
    pub fn cleanup(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file)
                .with_context(|| format!("Failed to remove PID file: {}", self.pid_file.display()))?;
            info!("Daemon PID file removed");
        }
        Ok(())
    }

    /// Check if daemon is currently running
    ///
    /// Returns true if:
    /// - PID file exists
    /// - PID can be parsed
    /// - Process with that PID exists
    pub fn is_running(&self) -> bool {
        if !self.pid_file.exists() {
            return false;
        }

        match self.read_pid() {
            Ok(pid) => process_exists(pid),
            Err(_) => false,
        }
    }

    /// Read PID from file
    pub fn read_pid(&self) -> Result<u32> {
        let pid_str = fs::read_to_string(&self.pid_file)
            .with_context(|| format!("Failed to read PID file: {}", self.pid_file.display()))?;
        pid_str
            .trim()
            .parse()
            .with_context(|| format!("Invalid PID in file: {}", pid_str))
    }

    /// Get PID file path
    pub fn pid_file(&self) -> &PathBuf {
        &self.pid_file
    }
}

impl Default for DaemonLifecycle {
    fn default() -> Self {
        Self::new().expect("Failed to initialize DaemonLifecycle")
    }
}

/// Check if a process with the given PID exists
///
/// Uses platform-specific methods:
/// - Unix: kill(pid, 0) to check existence without sending signal
/// - Windows: sysinfo crate to enumerate processes
#[cfg(target_family = "unix")]
fn process_exists(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    // kill with NULL signal checks existence without affecting process
    kill(Pid::from_raw(pid as i32), None).is_ok()
}

#[cfg(target_family = "windows")]
fn process_exists(pid: u32) -> bool {
    use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        ProcessRefreshKind::nothing(),
    );
    system.process(Pid::from(pid as usize)).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pid_file_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let pid_file = temp_dir.path().join("daemon.pid");

        let lifecycle = DaemonLifecycle {
            pid_file: pid_file.clone(),
        };

        // Write PID
        lifecycle.write_pid().unwrap();
        assert!(pid_file.exists());

        // Read PID
        let pid = lifecycle.read_pid().unwrap();
        assert_eq!(pid, std::process::id());

        // Check running
        assert!(lifecycle.is_running());

        // Cleanup
        lifecycle.cleanup().unwrap();
        assert!(!pid_file.exists());
        assert!(!lifecycle.is_running());
    }

    #[test]
    fn test_process_exists() {
        // Current process should exist
        assert!(process_exists(std::process::id()));

        // PID 1 should exist on Unix (init/systemd), may not exist on Windows
        #[cfg(target_family = "unix")]
        assert!(process_exists(1));

        // Very high PID should not exist
        assert!(!process_exists(999999999));
    }
}
