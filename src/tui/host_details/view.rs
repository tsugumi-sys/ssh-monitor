use futures::executor::block_on;
use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let block = Block::default().borders(Borders::ALL).title("Host Details");
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // model
            Constraint::Length(3), // total usage
            Constraint::Min(0),    // per cpu
        ])
        .split(inner);

    if let Some(id) = &app.selected_id {
        let cpu_map = block_on(app.cpu_states.snapshot_map());
        if let Some(cpu) = cpu_map.get(id) {
            let header = Paragraph::new(format!("CPU Usage for {}", id))
                .alignment(Alignment::Center);
            frame.render_widget(header, chunks[0]);

            let model = Paragraph::new(cpu.model_name.clone())
                .alignment(Alignment::Left);
            frame.render_widget(model, chunks[1]);

            let total_gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Green))
                .ratio((cpu.usage_percent as f64) / 100.0)
                .label(format!("{:.1}%", cpu.usage_percent));
            frame.render_widget(total_gauge, chunks[2]);

            let per_core_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1); cpu.per_core.len()])
                .split(chunks[3]);
            for (i, usage) in cpu.per_core.iter().enumerate() {
                if i < per_core_chunks.len() {
                    let gauge = Gauge::default()
                        .gauge_style(Style::default().fg(Color::Cyan))
                        .label(format!("CPU{} {:.1}%", i, usage))
                        .ratio((*usage as f64) / 100.0);
                    frame.render_widget(gauge, per_core_chunks[i]);
                }
            }
        } else {
            frame.render_widget(Paragraph::new("No CPU data"), inner);
        }
    } else {
        frame.render_widget(Paragraph::new("No host selected"), inner);
    }
}
