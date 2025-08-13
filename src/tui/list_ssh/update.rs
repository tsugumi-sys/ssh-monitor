use crate::App;
use crossterm::event::KeyCode;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let total = app.visible_hosts.len();
    if total == 0 {
        return;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            let new_index = match app.table_state.selected() {
                Some(i) if i + 1 < app.visible_hosts.len() => i + 1,
                _ => 0,
            };
            app.table_state.select(Some(new_index));
            app.selected_id = Some(app.visible_hosts[new_index].0.clone());
            scroll_if_needed(app, new_index);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let new_index = match app.table_state.selected() {
                Some(0) | None => app.visible_hosts.len().saturating_sub(1),
                Some(i) => i - 1,
            };
            app.table_state.select(Some(new_index));
            app.selected_id = Some(app.visible_hosts[new_index].0.clone());
            scroll_if_needed(app, new_index);
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        _ => {}
    }
}

fn scroll_if_needed(app: &mut App, new_index: usize) {
    let visible_rows = app.table_height.max(1);
    if new_index < app.vertical_scroll {
        app.vertical_scroll = new_index;
    } else if new_index >= app.vertical_scroll + visible_rows {
        app.vertical_scroll = new_index + 1 - visible_rows;
    }
}
