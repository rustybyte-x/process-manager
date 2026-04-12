use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use process_manager_core::{
    action::Action,
    app::{App, UiMode},
};

/// Polls terminal input and maps it to a domain-level [`Action`].
///
/// This function only emits [`Action::Tick`] when auto-refresh is actually due.
/// That avoids unnecessary redraws and reduces CPU usage while the UI is idle.
pub fn next_action(app: &App) -> Result<Option<Action>> {
    // Poll for keyboard input first.
    if event::poll(Duration::from_millis(100))? {
        let Event::Key(key) = event::read()? else {
            return Ok(None);
        };

        if key.kind != KeyEventKind::Press {
            return Ok(None);
        }

        let action = match app.mode {
            UiMode::Normal => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('r') => Some(Action::Refresh),
                KeyCode::Char('d') => Some(Action::ConfirmKill),
                KeyCode::Char('s') => Some(Action::CycleSort),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::SelectNext),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::SelectPrevious),
                KeyCode::Char('/') => Some(Action::EnterFilterMode),
                KeyCode::Char(':') => Some(Action::EnterCommandMode),
                _ => None,
            },

            UiMode::Filter => match key.code {
                KeyCode::Esc => Some(Action::LeaveInputMode),
                KeyCode::Enter => Some(Action::SubmitInput),
                KeyCode::Backspace => Some(Action::Backspace),
                KeyCode::Char(c) => Some(Action::InputChar(c)),
                _ => None,
            },

            UiMode::Command => match key.code {
                KeyCode::Esc => Some(Action::LeaveInputMode),
                KeyCode::Enter => Some(Action::SubmitInput),
                KeyCode::Tab => Some(Action::AutocompleteCommand),
                KeyCode::Backspace => Some(Action::Backspace),
                KeyCode::Char(c) => Some(Action::InputChar(c)),
                _ => None,
            },

            UiMode::ConfirmKill => match key.code {
                KeyCode::Char('y') => Some(Action::KillSelected),
                KeyCode::Char('n') | KeyCode::Esc => Some(Action::CancelKill),
                _ => None,
            },
        };

        return Ok(action);
    }

    // Emit a tick only when auto-refresh is actually due.
    if app.should_auto_refresh_now() {
        return Ok(Some(Action::Tick));
    }

    Ok(None)
}
