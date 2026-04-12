use std::process::{Command, Stdio};

use sysinfo::{Pid, Signal, System};

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
    /// Creates a new system process manager and performs an initial refresh.
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }
}

impl ProcessManager for SystemProcessManager {
    fn list_processes(&mut self) -> Result<Vec<ProcessEntry>, Error> {
        // Prefer a process-specific refresh over refresh_all() to keep
        // the TUI lighter during periodic updates.
        self.system.refresh_processes();

        let mut processes: Vec<ProcessEntry> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| ProcessEntry {
                pid: pid.as_u32(),
                name: p.name().to_string(),
                cpu_percent: p.cpu_usage(),
                memory_bytes: p.memory(),
                status: p.status().into(),
            })
            .collect();

        processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(processes)
    }

    fn kill_process(&mut self, pid: u32) -> Result<(), Error> {
        self.system.refresh_all();

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
                #[cfg(target_os = "windows")]
                {
                    let command_line = join_command_line(command);

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
                    let command_line = join_command_line(command);
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
                    let command_line = join_command_line(command);

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

/// Builds a single shell-style command line from program and arguments.
///
/// This is primarily used when launching commands through a terminal wrapper
/// such as `cmd`, AppleScript, or a terminal emulator.
fn join_command_line(command: &CommandLine) -> String {
    std::iter::once(command.program.as_str())
        .chain(command.args.iter().map(|arg| arg.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Escapes a string for safe embedding inside a single-quoted shell string.
///
/// Example:
/// `hello'world` becomes `'hello'"'"'world'`
fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\"'\"'"#))
}

/// Escapes double quotes and backslashes for AppleScript string literals.
fn escape_applescript_string(input: &str) -> String {
    input.replace('\\', r#"\\"#).replace('"', r#"\""#)
}
