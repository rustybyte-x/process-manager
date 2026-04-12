use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use process_manager_core::{
    action::Action,
    app::{App, UiMode},
};

/// Polls terminal input and maps it to a domain-level [`Action`].
///
/// If no key event is available within the polling interval, this function
/// emits a periodic [`Action::Tick`] to support time-based tasks such as
/// auto-refresh.
pub fn next_action(app: &App) -> Result<Option<Action>> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(Some(Action::Tick));
    }

    let Event::Key(key) = event::read()? else {
        return Ok(Some(Action::Tick));
    };

    if key.kind != KeyEventKind::Press {
        return Ok(Some(Action::Tick));
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

    Ok(action)
}
