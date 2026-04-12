use crate::{
    action::Action,
    app::{App, SortMode, StatusLevel, UiMode},
    command::CommandLine,
    error::Error,
    manager::ProcessManager,
};

/// Applies an [`Action`] by coordinating pure state transitions with
/// side effects from the active [`ProcessManager`].
///
/// This function acts as the boundary between UI intent and system-level
/// process operations.
pub fn dispatch(
    app: &mut App,
    process_manager: &mut impl ProcessManager,
    action: Action,
) -> Result<(), Error> {
    match action {
        Action::Tick => {
            if app.should_auto_refresh_now() && matches!(app.mode, UiMode::Normal) {
                let processes = process_manager.list_processes()?;
                app.set_processes(processes);
                app.mark_refreshed();
            }
        }

        Action::Refresh => {
            let processes = process_manager.list_processes()?;
            let count = processes.len();
            app.set_processes(processes);
            app.mark_refreshed();
            app.set_status(StatusLevel::Info, format!("{count} processes loaded"));
        }

        Action::CycleSort => {
            app.update(&Action::CycleSort);
            app.apply_sort();

            let label = match app.sort_mode {
                SortMode::NameAsc => "name",
                SortMode::PidAsc => "pid",
                SortMode::CpuDesc => "cpu",
                SortMode::MemoryDesc => "memory",
            };

            app.set_status(StatusLevel::Info, format!("sort: {label}"));
        }

        Action::ConfirmKill => {
            if app.selected_process().is_none() {
                app.set_status(StatusLevel::Warning, "no process selected");
            } else {
                app.mode = UiMode::ConfirmKill;
                let pid = app.selected_pid().unwrap_or(0);
                let name = app.selected_name().unwrap_or("unknown");
                app.set_status(
                    StatusLevel::Warning,
                    format!("confirm kill: {name} ({pid}) - press y to confirm, n to cancel"),
                );
            }
        }

        Action::CancelKill => {
            app.mode = UiMode::Normal;
            app.set_status(StatusLevel::Info, "kill cancelled");
        }

        Action::KillSelected => {
            let Some(pid) = app.selected_pid() else {
                app.set_status(StatusLevel::Warning, "no process selected");
                app.mode = UiMode::Normal;
                return Ok(());
            };

            process_manager.kill_process(pid)?;

            let processes = process_manager.list_processes()?;
            app.set_processes(processes);
            app.mark_refreshed();
            app.mode = UiMode::Normal;
            app.set_status(
                StatusLevel::Success,
                format!("killed process with pid {pid}"),
            );
        }

        Action::SubmitInput => match app.mode {
            UiMode::Normal | UiMode::ConfirmKill => {}

            UiMode::Filter => {
                app.mode = UiMode::Normal;
                let count = app.filtered_indices.len();
                app.set_status(
                    StatusLevel::Info,
                    format!("filter applied ({count} matches)"),
                );
            }

            UiMode::Command => {
                let raw_input = app.command_input.trim().to_string();

                if raw_input.is_empty() {
                    app.mode = UiMode::Normal;
                    app.set_status(StatusLevel::Warning, "command is empty");
                    return Ok(());
                }

                let command = CommandLine::parse(&raw_input)?;

                match process_manager.start_process(&command) {
                    Ok(()) => {
                        app.command_input.clear();
                        app.mode = UiMode::Normal;

                        let processes = process_manager.list_processes()?;
                        app.set_processes(processes);
                        app.mark_refreshed();

                        app.set_status(StatusLevel::Success, format!("started: {raw_input}"));
                    }
                    Err(err) => {
                        app.set_status(StatusLevel::Error, format!("failed to start: {err}"));
                    }
                }
            }
        },

        other => {
            app.update(&other);
        }
    }

    Ok(())
}
