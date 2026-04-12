use sysinfo::{Pid, Signal, System};

use crate::{command::CommandLine, error::Error, manager::ProcessManager, process::ProcessEntry};

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
        self.system.refresh_all();

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

    /// Starts a new child process without blocking the TUI.
    ///
    /// The spawned process is detached from the application's control flow
    /// after creation. Errors reported here indicate startup failures only.
    fn start_process(&mut self, command: &CommandLine) -> Result<(), Error> {
        std::process::Command::new(&command.program)
            .args(&command.args)
            .spawn()?;

        Ok(())
    }
}
