use std::process::{Command, Stdio};

use sysinfo::{Pid, ProcessRefreshKind, Signal, System};

use crate::{
    command::{CommandLine, CommandMode},
    error::Error,
    manager::ProcessManager,
    process::ProcessEntry,
};

/// Default system-backed implementation of [`ProcessManager`].
#[derive(Default)]
pub struct SystemProcessManager {
    system: System,
}

impl SystemProcessManager {
    /// Creates a new system process manager and performs an initial refresh.
    pub fn new() -> Self {
        let mut system = System::new_all();

        // Initial refresh so that the first CPU readings have a baseline.
        system.refresh_processes_specifics(ProcessRefreshKind::new().with_cpu().with_memory());

        Self { system }
    }
}

impl ProcessManager for SystemProcessManager {
    fn list_processes(&mut self) -> Result<Vec<ProcessEntry>, Error> {
        // Refresh only the process information that the TUI actually displays.
        self.system
            .refresh_processes_specifics(ProcessRefreshKind::new().with_cpu().with_memory());

        let mut processes: Vec<ProcessEntry> = self
            .system
            .processes()
            .iter()
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

        processes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(processes)
    }

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
                let command_line = std::iter::once(command.program.as_str())
                    .chain(command.args.iter().map(|arg| arg.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ");

                #[cfg(target_os = "windows")]
                let child = Command::new("cmd")
                    .args(["/C", "start", "", &command_line])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;

                #[cfg(target_os = "macos")]
                let child = Command::new("osascript")
                    .args([
                        "-e",
                        &format!(
                            r#"tell application "Terminal"
                                if (count of windows) = 0 then
                                    do script "{}"
                                else
                                    do script "{}" in front window
                                end if
                                activate
                            end tell"#,
                            escape_applescript_string(&format!(
                                "/bin/zsh -lc {}",
                                shell_quote(&command_line)
                            )),
                            escape_applescript_string(&format!(
                                "/bin/zsh -lc {}",
                                shell_quote(&command_line)
                            )),
                        ),
                    ])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;

                #[cfg(target_os = "linux")]
                let child = Command::new("x-terminal-emulator")
                    .args(["-e", &command_line])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;

                Ok(child.id())
            }
        }
    }
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\"'\"'"#))
}

fn escape_applescript_string(input: &str) -> String {
    input.replace('\\', r#"\\"#).replace('"', r#"\""#)
}
