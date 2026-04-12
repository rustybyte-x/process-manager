/// Describes how a parsed command should be launched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandMode {
    /// Start the process in the background without attaching it to the TUI's
    /// standard input, output, or error streams.
    Background,

    /// Start the process in a separate terminal window.
    Terminal,
}

/// A parsed command line consisting of a program, its arguments,
/// and the desired launch mode.
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

    /// The mode used to launch the command.
    pub mode: CommandMode,
}

impl CommandLine {
    /// Parses a raw command string into a [`CommandLine`].
    ///
    /// Supported forms:
    ///
    /// - `sleep 60` → background mode
    /// - `! python3 -m http.server 8000` → terminal mode
    ///
    /// The current implementation uses whitespace splitting and therefore
    /// does not support shell-style quoting or escaping.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::InvalidCommand`] if no executable
    /// could be parsed from the input.
    pub fn parse(input: &str) -> Result<Self, crate::error::Error> {
        let mut parts = input.split_whitespace();

        let first = parts.next().ok_or(crate::error::Error::InvalidCommand)?;

        let (mode, program) = if first == "!" {
            let program = parts.next().ok_or(crate::error::Error::InvalidCommand)?;

            (CommandMode::Terminal, program)
        } else {
            (CommandMode::Background, first)
        };

        let args = parts.map(|part| part.to_string()).collect();

        Ok(Self {
            program: program.to_string(),
            args,
            mode,
        })
    }
}

/// Returns a small set of built-in command suggestions for command mode.
///
/// Suggestions intentionally focus on common demo and testing commands.
/// Completion is limited to the executable part of the command to avoid
/// rewriting user-provided arguments unexpectedly.
pub fn suggest_commands(input: &str) -> Vec<&'static str> {
    const KNOWN_COMMANDS: &[&str] = &["sleep", "yes", "python3", "open", "ping"];

    let trimmed = input.trim_start();

    // Support both:
    // - "py"
    // - "! py"
    let executable_prefix = if let Some(rest) = trimmed.strip_prefix('!') {
        rest.trim_start()
    } else {
        trimmed
    };

    // Only autocomplete the executable name, not argument lists.
    if executable_prefix.contains(' ') {
        return Vec::new();
    }

    KNOWN_COMMANDS
        .iter()
        .copied()
        .filter(|candidate| candidate.starts_with(executable_prefix))
        .collect()
}

/// Returns the best matching command completion for the given input.
///
/// If exactly one executable matches the current prefix, the completed
/// command string is returned. Otherwise `None` is returned to avoid
/// surprising replacements.
pub fn complete_command(input: &str) -> Option<String> {
    let trimmed = input.trim_start();
    let terminal_mode = trimmed.starts_with('!');

    let suggestions = suggest_commands(input);

    if suggestions.len() != 1 {
        return None;
    }

    let completed = suggestions[0];

    if terminal_mode {
        Some(format!("! {completed}"))
    } else {
        Some(completed.to_string())
    }
}
