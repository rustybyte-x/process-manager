/// Actions that can be triggered by the UI or event loop.
///
/// This enum represents user intent and application-level commands.
/// It is used to decouple input handling from state transitions and
/// side-effect execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Periodic tick used for background tasks such as auto-refresh.
    Tick,

    /// Refresh the process list from the underlying process manager.
    Refresh,

    /// Move the current selection to the next visible process.
    SelectNext,

    /// Move the current selection to the previous visible process.
    SelectPrevious,

    /// Terminate the currently selected process.
    KillSelected,

    /// Enter filter input mode.
    EnterFilterMode,

    /// Enter command input mode.
    EnterCommandMode,

    /// Leave the current input mode and return to normal mode.
    LeaveInputMode,

    /// Open the kill confirmation dialog.
    ConfirmKill,

    /// Cancel the kill confirmation dialog.
    CancelKill,

    /// Cycle through the available sort modes.
    CycleSort,

    /// Append a character to the active input buffer.
    InputChar(char),

    /// Remove the last character from the active input buffer.
    Backspace,

    /// Submit the currently active input buffer.
    SubmitInput,

    /// Clear the currently active input buffer.
    ClearInput,

    /// Request graceful application shutdown.
    Quit,
}
