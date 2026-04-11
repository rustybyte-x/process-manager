use crate::{command::CommandLine, process::ProcessEntry};

/// Abstraction over process-related system operations.
///
/// This trait defines the minimal interface required by the application
/// to inspect, start, and terminate processes.
///
/// Implementations may use platform-specific APIs or third-party crates
/// behind the scenes, while the rest of the application remains independent
/// from those details.
pub trait ProcessManager {
    /// Returns the current list of running processes.
    ///
    /// # Errors
    ///
    /// Returns an error if the process list could not be retrieved.
    fn list_processes(&mut self) -> Result<Vec<ProcessEntry>, crate::error::Error>;

    /// Terminates the process with the given process ID.
    ///
    /// # Parameters
    ///
    /// - `pid`: The operating system process ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the process does not exist or could not be terminated.
    fn kill_process(&mut self, pid: u32) -> Result<(), crate::error::Error>;

    /// Starts a new process from the given command line.
    ///
    /// # Parameters
    ///
    /// - `command`: The parsed command line to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if the command is invalid or the process could not be started.
    fn start_process(&mut self, command: &CommandLine) -> Result<(), crate::error::Error>;
}
