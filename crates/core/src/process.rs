/// A process entry shown by the application.
///
/// This type contains the subset of process information needed by the UI.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessEntry {
    /// The operating system process ID.
    pub pid: u32,

    /// A lowercase copy of the process name used for case-insensitive filtering
    /// and sorting without repeated string allocations.
    pub search_name: String,

    /// The display name of the process.
    pub name: String,

    /// The current CPU usage in percent.
    pub cpu_percent: f32,

    /// The current memory usage.
    ///
    /// The exact unit should match the underlying process backend.
    /// Ensure the field name reflects the real unit returned by the backend.
    pub memory_bytes: u64,

    /// The high-level process status used by the application.
    pub status: ProcessStatus,
}

/// High-level process states used by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// The process is currently running.
    Running,

    /// The process is sleeping or idle.
    Sleeping,

    /// The process has been stopped.
    Stopped,

    /// The process is a zombie process.
    Zombie,

    /// The process state could not be determined.
    Unknown,
}

/// Converts an external `sysinfo` process status into the application's
/// internal [`ProcessStatus`] representation.
///
/// Any source variants not explicitly handled are mapped to
/// [`ProcessStatus::Unknown`] to keep the conversion forward-compatible.
impl From<sysinfo::ProcessStatus> for ProcessStatus {
    fn from(status: sysinfo::ProcessStatus) -> Self {
        match status {
            sysinfo::ProcessStatus::Run => ProcessStatus::Running,
            sysinfo::ProcessStatus::Stop => ProcessStatus::Stopped,
            sysinfo::ProcessStatus::Sleep => ProcessStatus::Sleeping,
            sysinfo::ProcessStatus::Zombie => ProcessStatus::Zombie,
            _ => ProcessStatus::Unknown,
        }
    }
}
