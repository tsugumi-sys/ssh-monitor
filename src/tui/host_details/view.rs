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

    let cpu_detail = block_on(fetch_latest_cpu_by_host(&app.db, host_id)).unwrap_or(None);

    let block = Block::default().title("CPU").borders(Borders::ALL);
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if let Some(cpu) = cpu_detail {
        // Total usage line
        let mut lines = vec![
            format!("Model: {}", cpu.model_name),
            format!("Total: {:.1}%", cpu.usage_percent),
        ];

        // Per-core usage
        let core_lines: Vec<String> = cpu
            .per_core
            .iter()
            .enumerate()
            .map(|(i, usage)| render_cpu_bar(&format!("c{}", i), *usage))
            .collect();

        // Split core_lines into multiple columns (e.g., 2 or 3 per row)
        let cols = 3;
        for chunk in core_lines.chunks(cols) {
            lines.push(chunk.join("   "));
        }

        let paragraph = Paragraph::new(lines.join("\n"))
            .style(Style::default())
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, inner_area);
    } else {
        let paragraph = Paragraph::new("No CPU data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, inner_area);
    }
}

fn render_cpu_bar(label: &str, percent: f32) -> String {
    let width = 14;
    let filled = (percent / 100.0 * width as f32).round() as usize;
    let empty = width - filled;
    format!(
        "{:>3} [{}{}] {:>5.1}%",
        label,
        "█".repeat(filled),
        " ".repeat(empty),
        percent
    )
}
