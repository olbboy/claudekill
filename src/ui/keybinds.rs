// Keybinds module - keyboard input handling

use crate::app::{App, AppState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

/// Actions that can be triggered by user input
pub enum Action {
    None,
    Quit,
    Delete,
}

/// Handle keyboard events with timeout
pub fn handle_events(app: &mut App, timeout: Duration) -> anyhow::Result<Action> {
    if event::poll(timeout)?
        && let Event::Key(key) = event::read()?
        && key.kind == KeyEventKind::Press
    {
        return handle_key(app, key.code, key.modifiers);
    }
    Ok(Action::None)
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> anyhow::Result<Action> {
    // Handle help overlay first - any key closes it
    if app.show_help {
        app.show_help = false;
        return Ok(Action::None);
    }

    // Handle confirm dialog
    if app.state == AppState::Confirming {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.state = AppState::Deleting;
                return Ok(Action::Delete);
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.state = AppState::Browsing;
                app.message = None;
            }
            _ => {}
        }
        return Ok(Action::None);
    }

    // Normal keybinds
    match code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            return Ok(Action::Quit);
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return Ok(Action::Quit);
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),

        // Selection
        KeyCode::Char(' ') => app.toggle_selection(),
        KeyCode::Char('a') => app.select_all(),
        KeyCode::Char('n') => app.select_none(),

        // Delete
        KeyCode::Char('d') => {
            if app.selected_count() > 0 && app.state == AppState::Browsing {
                app.state = AppState::Confirming;
            }
        }

        // Help
        KeyCode::Char('?') => app.show_help = !app.show_help,

        _ => {}
    }

    Ok(Action::None)
}
