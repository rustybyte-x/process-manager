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
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Err(crate::error::Error::InvalidCommand);
        }

        Ok(Self {
            program: parts[0].to_string(),
            args: parts[1..].iter().map(|part| part.to_string()).collect(),
        })
    }
}

/// Returns a small set of built-in command suggestions for the command mode.
///
/// These suggestions are intentionally conservative and are meant to improve
/// usability for common demo and testing workflows.
///
/// The function matches only on the first token because command completion
/// should complete the executable name, not arbitrary arguments.
pub fn suggest_commands(input: &str) -> Vec<&'static str> {
    const KNOWN_COMMANDS: &[&str] = &["sleep", "yes", "python3", "open", "ping"];

    let trimmed = input.trim_start();

    // If the user has already entered arguments, we no longer try to
    // autocomplete the executable name.
    if trimmed.contains(' ') {
        return Vec::new();
    }

    KNOWN_COMMANDS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(trimmed))
        .collect()
}

/// Returns the best matching command completion for the given input.
///
/// If there is exactly one matching executable prefix, that suggestion
/// is returned. Otherwise `None` is returned to avoid surprising input
/// replacements.
pub fn complete_command(input: &str) -> Option<String> {
    let suggestions = suggest_commands(input);

    if suggestions.len() == 1 {
        Some(suggestions[0].to_string())
    } else {
        None
    }
}
