use color_eyre::Result;
use process_manager_core::{
    action::Action,
    app::{App, StatusLevel, UiMode},
    controller::dispatch,
    system::SystemProcessManager,
};

mod events;
mod ui;

/// Starts the terminal application and installs error reporting.
fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = app(terminal);
    ratatui::restore();

    result
}

/// Runs the main TUI event loop.
///
/// The loop redraws the screen only when the application state has changed.
/// This keeps CPU usage lower than redrawing on every polling cycle.
fn app(mut terminal: ratatui::DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    let mut process_manager = SystemProcessManager::new();

    // Force the first frame to be rendered after the initial refresh.
    let mut dirty = true;

    dispatch(&mut app, &mut process_manager, Action::Refresh)?;

    while !app.should_quit {
        if dirty {
            terminal.draw(|frame| ui::render(frame, &app))?;
            dirty = false;
        }

        if let Some(action) = events::next_action(&app)? {
            match dispatch(&mut app, &mut process_manager, action) {
                Ok(()) => {
                    // Any processed action may have changed state or triggered a refresh.
                    dirty = true;
                }
                Err(err) => {
                    // Surface operational failures in the status bar instead of
                    // aborting the TUI session.
                    app.set_status(StatusLevel::Error, format!("error: {err}"));
                    app.mode = UiMode::Normal;
                    dirty = true;
                }
            }
        }
    }

    Ok(())
}
