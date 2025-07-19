use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let block = Block::default().title("Host Details").borders(Borders::ALL);
    let content = if let Some(id) = &app.selected_id {
        format!("details view! (host: {})", id)
    } else {
        "details view!".to_string()
    };
    let paragraph = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
