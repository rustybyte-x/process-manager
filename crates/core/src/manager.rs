use crate::{command::CommandLine, process::ProcessEntry};

/// Abstraction over process-related system operations.
///
/// This trait defines the minimal interface required by the application
/// to inspect, start, and terminate processes.
pub trait ProcessManager {
    /// Returns the current list of running processes.
    fn list_processes(&mut self) -> Result<Vec<ProcessEntry>, crate::error::Error>;

    /// Terminates the process with the given process ID.
    fn kill_process(&mut self, pid: u32) -> Result<(), crate::error::Error>;

    /// Starts a new process from the given command line.
    ///
    /// The returned PID is the PID of the directly spawned child process.
    /// For terminal-launch modes, this may refer to the terminal launcher
    /// rather than the final application process shown to the user.
    fn start_process(&mut self, command: &CommandLine) -> Result<u32, crate::error::Error>;
}
