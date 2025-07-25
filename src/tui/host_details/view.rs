use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::backend::db::cpu::queries::fetch_latest_cpu_by_host;
use futures::executor::block_on;

use crate::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();
    let Some(host_id) = &app.selected_id else {
        let paragraph = Paragraph::new("No host selected")
            .block(Block::default().title("Host Details").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    };

    let cpu_detail = block_on(fetch_latest_cpu_by_host(&app.db, host_id))
        .unwrap_or(None);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(format!("Host: {}", host_id))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Host Details"));
    frame.render_widget(title, chunks[0]);

    if let Some(cpu) = cpu_detail {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Total CPU Usage"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(cpu.usage_percent as f64 / 100.0)
            .label(format!("{:.1}%", cpu.usage_percent));
        frame.render_widget(gauge, chunks[1]);

        let names: Vec<String> = (0..cpu.per_core.len())
            .map(|i| format!("c{}", i))
            .collect();
        let data_owned: Vec<(String, u64)> = cpu
            .per_core
            .iter()
            .enumerate()
            .map(|(i, v)| (names[i].clone(), *v as u64))
            .collect();
        let data: Vec<(&str, u64)> = data_owned
            .iter()
            .map(|(n, v)| (n.as_str(), *v))
            .collect();

        let chart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("Per Core Usage"))
            .bar_width(3)
            .bar_gap(1)
            .value_style(Style::default().fg(Color::Black).bg(Color::Green))
            .bar_style(Style::default().fg(Color::Green))
            .max(100)
            .data(&data);
        frame.render_widget(chart, chunks[2]);
    } else {
        let paragraph = Paragraph::new("No CPU data")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, chunks[1]);
    }
}
