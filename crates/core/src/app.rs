use std::time::{Duration, Instant};

use crate::{
    action::Action,
    command::{complete_command, suggest_commands},
    process::ProcessEntry,
};

/// Describes how user input should currently be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Normal,
    Filter,
    Command,
    ConfirmKill,
}

/// Defines the active sort mode for the process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    NameAsc,
    PidAsc,
    CpuDesc,
    MemoryDesc,
}

/// Semantic status levels used by the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Central application state.
///
/// This type contains UI-facing state and process data used by the TUI.
/// It intentionally remains free of direct side effects such as starting
/// or killing processes. External operations should be coordinated by
/// a controller or dispatch layer.
#[derive(Debug)]
pub struct App {
    /// The full list of processes currently known to the application.
    pub processes: Vec<ProcessEntry>,

    /// Indices into `processes` that are visible after applying the current filter.
    pub filtered_indices: Vec<usize>,

    /// The currently selected row within `filtered_indices`.
    pub selected: usize,

    /// Indicates whether the application should exit.
    pub should_quit: bool,

    /// The current filter input buffer.
    pub filter_input: String,

    /// The current command input buffer.
    pub command_input: String,

    /// The active UI mode.
    pub mode: UiMode,

    /// The active process list sort mode.
    pub sort_mode: SortMode,

    /// The current status message shown by the UI.
    pub status: String,

    /// The semantic level associated with the status message.
    pub status_level: StatusLevel,

    /// Indicates whether periodic auto-refresh is enabled.
    pub auto_refresh: bool,

    /// The configured interval used for automatic refresh.
    pub refresh_interval: Duration,

    /// Timestamp of the most recent successful refresh.
    pub last_refresh: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates a new application state with default UI settings.
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
            filtered_indices: Vec::new(),
            selected: 0,
            should_quit: false,
            filter_input: String::new(),
            command_input: String::new(),
            mode: UiMode::Normal,
            sort_mode: SortMode::NameAsc,
            status: String::from(
                "r refresh | d kill | / filter | : command | Tab autocomplete | q quit",
            ),
            status_level: StatusLevel::Info,
            auto_refresh: true,
            refresh_interval: Duration::from_secs(2),
            last_refresh: Instant::now(),
        }
    }

    /// Applies a pure state transition for the given [`Action`].
    ///
    /// This method updates local application state only and intentionally does
    /// not perform any external side effects such as process management.
    pub fn update(&mut self, action: &Action) {
        match action {
            Action::SelectNext => self.select_next(),
            Action::SelectPrevious => self.select_previous(),

            Action::EnterFilterMode => self.mode = UiMode::Filter,
            Action::EnterCommandMode => self.mode = UiMode::Command,
            Action::LeaveInputMode => self.mode = UiMode::Normal,

            Action::ConfirmKill => self.mode = UiMode::ConfirmKill,
            Action::CancelKill => self.mode = UiMode::Normal,

            Action::CycleSort => {
                self.sort_mode = match self.sort_mode {
                    SortMode::NameAsc => SortMode::PidAsc,
                    SortMode::PidAsc => SortMode::CpuDesc,
                    SortMode::CpuDesc => SortMode::MemoryDesc,
                    SortMode::MemoryDesc => SortMode::NameAsc,
                };
            }

            Action::InputChar(c) => match self.mode {
                UiMode::Normal | UiMode::ConfirmKill => {}
                UiMode::Filter => {
                    self.filter_input.push(*c);
                    self.rebuild_filter();
                }
                UiMode::Command => {
                    self.command_input.push(*c);
                }
            },

            Action::Backspace => match self.mode {
                UiMode::Normal | UiMode::ConfirmKill => {}
                UiMode::Filter => {
                    self.filter_input.pop();
                    self.rebuild_filter();
                }
                UiMode::Command => {
                    self.command_input.pop();
                }
            },

            Action::ClearInput => match self.mode {
                UiMode::Normal | UiMode::ConfirmKill => {}
                UiMode::Filter => {
                    self.filter_input.clear();
                    self.rebuild_filter();
                }
                UiMode::Command => {
                    self.command_input.clear();
                }
            },

            Action::AutocompleteCommand => {
                if self.mode == UiMode::Command {
                    if let Some(completed) = complete_command(&self.command_input) {
                        self.command_input = completed;
                    }
                }
            }

            Action::Quit => self.should_quit = true,

            Action::Tick | Action::Refresh | Action::KillSelected | Action::SubmitInput => {}
        }
    }

    /// Returns whether an automatic refresh should be triggered now.
    pub fn should_auto_refresh_now(&self) -> bool {
        self.auto_refresh && self.last_refresh.elapsed() >= self.refresh_interval
    }

    /// Marks the application as having completed a refresh just now.
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Instant::now();
    }

    /// Sets the current status message and its semantic level.
    pub fn set_status(&mut self, level: StatusLevel, message: impl Into<String>) {
        self.status_level = level;
        self.status = message.into();
    }

    /// Returns the current command suggestions for the command input buffer.
    ///
    /// Suggestions are only shown in command mode to avoid mixing command
    /// concerns with other input contexts.
    pub fn command_suggestions(&self) -> Vec<&'static str> {
        if self.mode != UiMode::Command {
            return Vec::new();
        }

        suggest_commands(&self.command_input)
    }

    pub fn set_processes(&mut self, mut processes: Vec<ProcessEntry>) {
        self.sort_processes(&mut processes);
        self.processes = processes;
        self.rebuild_filter();
        self.clamp_selection();
    }

    pub fn selected_process(&self) -> Option<&ProcessEntry> {
        let process_index = *self.filtered_indices.get(self.selected)?;
        self.processes.get(process_index)
    }

    pub fn selected_pid(&self) -> Option<u32> {
        self.selected_process().map(|p| p.pid)
    }

    pub fn selected_name(&self) -> Option<&str> {
        self.selected_process().map(|p| p.name.as_str())
    }

    pub fn rebuild_filter(&mut self) {
        let needle = self.filter_input.trim().to_lowercase();

        self.filtered_indices = self
            .processes
            .iter()
            .enumerate()
            .filter(|(_, process)| {
                needle.is_empty()
                    || process.name.to_lowercase().contains(&needle)
                    || process.pid.to_string().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect();

        self.clamp_selection();
    }

    pub fn clamp_selection(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len() - 1;
        }
    }

    pub fn apply_sort(&mut self) {
        let mut processes = std::mem::take(&mut self.processes);
        self.sort_processes(&mut processes);
        self.processes = processes;
        self.rebuild_filter();
        self.clamp_selection();
    }

    fn sort_processes(&self, processes: &mut [ProcessEntry]) {
        match self.sort_mode {
            SortMode::NameAsc => {
                processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            SortMode::PidAsc => {
                processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
            SortMode::CpuDesc => {
                processes.sort_by(|a, b| {
                    b.cpu_percent
                        .partial_cmp(&a.cpu_percent)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortMode::MemoryDesc => {
                processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
            }
        }
    }

    fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected = 0;
            return;
        }

        self.selected = (self.selected + 1) % self.filtered_indices.len();
    }

    fn select_previous(&mut self) {
        if self.filtered_indices.is_empty() {
            self.selected = 0;
            return;
        }

        if self.selected == 0 {
            self.selected = self.filtered_indices.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}
