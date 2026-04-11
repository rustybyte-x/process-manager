/// A parsed command line consisting of a program and its arguments.
///
/// This type provides a structured representation of a command that can
/// be passed to a [`ProcessManager`](crate::manager::ProcessManager)
/// implementation for execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine {
    /// The executable or program name.
    pub program: String,

    /// The argument list passed to the executable.
    pub args: Vec<String>,
}

impl CommandLine {
    /// Parses a raw command string into a [`CommandLine`].
    ///
    /// The current implementation uses whitespace splitting and therefore
    /// does not support shell-style quoting or escaping.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::InvalidCommand`] if the input does not
    /// contain a program name.
    pub fn parse(input: &str) -> Result<Self, crate::error::Error> {
        let mut parts = input.split_whitespace();

        let Some(program) = parts.next() else {
            return Err(crate::error::Error::InvalidCommand);
        };

        Ok(Self {
            program: program.to_string(),
            args: parts.map(str::to_string).collect(),
        })
    }
}
