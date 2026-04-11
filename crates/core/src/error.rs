/// Errors that can occur while interacting with processes or commands.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An underlying I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The requested process could not be found.
    #[error("process with pid {0} not found")]
    ProcessNotFound(u32),

    /// The provided command is invalid.
    #[error("invalid command")]
    InvalidCommand,

    /// A process-related operation failed for an implementation-specific reason.
    #[error("operation failed: {0}")]
    OperationFailed(String),
}
