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
    ratatui::run(app)?;
    Ok(())
}

/// Runs the main TUI event loop.
///
/// The loop repeatedly renders the current state, collects the next action,
/// and dispatches it through the controller.
fn app(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    let mut process_manager = SystemProcessManager::new();

    // Load the initial process snapshot before the first frame is rendered.
    dispatch(&mut app, &mut process_manager, Action::Refresh)?;

    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if let Some(action) = events::next_action(&app)? {
            if let Err(err) = dispatch(&mut app, &mut process_manager, action) {
                app.set_status(StatusLevel::Error, format!("error: {err}"));
                app.mode = UiMode::Normal;
            }
        }
    }

    Ok(())
}
