use crate::{App, AppMode};
use crossterm::event::KeyCode;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::List;
        }
        _ => {}
    }
}
