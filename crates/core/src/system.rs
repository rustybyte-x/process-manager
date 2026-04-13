use std::process::{Command, Stdio};

use sysinfo::{Pid, ProcessRefreshKind, Signal, System};

use crate::{
    command::{CommandLine, CommandMode},
    error::Error,
    manager::ProcessManager,
    process::ProcessEntry,
};

/// Default system-backed implementation of [`ProcessManager`].
///
/// This implementation uses the `sysinfo` crate to inspect and manage
/// processes on the current machine.
#[derive(Default)]
pub struct SystemProcessManager {
    system: System,
}

impl SystemProcessManager {
    /// Creates a new system process manager and performs an initial process refresh.
    pub fn new() -> Self {
        let mut system = System::new_all();

        // Perform an initial refresh so that process CPU values have a baseline.
        system.refresh_processes_specifics(ProcessRefreshKind::new().with_cpu().with_memory());

        Self { system }
    }
}

impl ProcessManager for SystemProcessManager {
    /// Retrieves the current process snapshot from the system.
    ///
    /// Only process information relevant to the TUI is refreshed in order to
    /// keep periodic updates lighter than a full `refresh_all()`.
    fn list_processes(&mut self) -> Result<Vec<ProcessEntry>, Error> {
        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new().with_cpu().with_memory());

        // let current_pid = std::process::id();

        let mut processes: Vec<ProcessEntry> = self
            .system
            .processes()
            .iter()
            // Hide the process manager itself from the list so it does not
            // dominate the display with its own transient CPU usage.
            //.filter(|(pid, _)| pid.as_u32() != current_pid)
            .map(|(pid, p)| {
                let name = p.name().to_string();

                ProcessEntry {
                    pid: pid.as_u32(),
                    search_name: name.to_lowercase(),
                    name,
                    cpu_percent: p.cpu_usage(),
                    memory_bytes: p.memory(),
                    status: p.status().into(),
                }
            })
            .collect();

        processes.sort_by(|a, b| a.search_name.cmp(&b.search_name));
        Ok(processes)
    }

    /// Attempts to terminate the process with the given PID.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProcessNotFound`] if the PID does not exist in the
    /// current process snapshot.
    ///
    /// Returns [`Error::OperationFailed`] if the backend reported that the
    /// process could not be terminated.
    fn kill_process(&mut self, pid: u32) -> Result<(), Error> {
        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new());

        let Some(process) = self.system.process(Pid::from_u32(pid)) else {
            return Err(Error::ProcessNotFound(pid));
        };

        match process.kill_with(Signal::Kill) {
            Some(true) => Ok(()),
            Some(false) | None => Err(Error::OperationFailed(format!(
                "failed to kill process with pid {}",
                pid
            ))),
        }
    }

    /// Starts a new process from the provided parsed command line.
    ///
    /// Background commands are detached from the TUI's standard streams so they
    /// cannot corrupt the Ratatui screen. Terminal commands are launched through
    /// a platform-specific terminal wrapper.
    ///
    /// # Returns
    ///
    /// Returns the PID of the directly spawned child process. For terminal mode,
    /// this may be the launcher process rather than the final application shown
    /// in the external terminal window.
    fn start_process(&mut self, command: &CommandLine) -> Result<u32, Error> {
        match command.mode {
            CommandMode::Background => {
                let child = Command::new(&command.program)
                    .args(&command.args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;

                Ok(child.id())
            }

            CommandMode::Terminal => {
                let command_line = join_command_line(command);

                #[cfg(target_os = "windows")]
                {
                    let child = Command::new("cmd")
                        .args(["/C", "start", "", &command_line])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;

                    return Ok(child.id());
                }

                #[cfg(target_os = "macos")]
                {
                    let shell_command = format!("/bin/zsh -lc {}", shell_quote(&command_line));

                    let apple_script = format!(
                        r#"
                        tell application "Terminal"
                            if (count of windows) = 0 then
                                do script "{}"
                            else
                                do script "{}" in front window
                            end if
                            activate
                        end tell
                        "#,
                        escape_applescript_string(&shell_command),
                        escape_applescript_string(&shell_command),
                    );

                    let child = Command::new("osascript")
                        .args(["-e", &apple_script])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;

                    return Ok(child.id());
                }

                #[cfg(target_os = "linux")]
                {
                    let child = Command::new("x-terminal-emulator")
                        .args(["-e", &command_line])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;

                    return Ok(child.id());
                }

                #[allow(unreachable_code)]
                Err(Error::OperationFailed(
                    "terminal launch is not supported on this platform".to_string(),
                ))
            }
        }
    }
}

/// Builds a shell-style command line from program and arguments.
///
/// This is primarily used when launching a command through a terminal wrapper
/// such as `cmd`, AppleScript, or a terminal emulator.
fn join_command_line(command: &CommandLine) -> String {
    std::iter::once(command.program.as_str())
        .chain(command.args.iter().map(|arg| arg.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Escapes a string for safe embedding inside a single-quoted shell string.
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\"'\"'"#))
}

/// Escapes backslashes and double quotes for AppleScript string literals.
fn escape_applescript_string(input: &str) -> String {
    input.replace('\\', r#"\\"#).replace('"', r#"\""#)
}
