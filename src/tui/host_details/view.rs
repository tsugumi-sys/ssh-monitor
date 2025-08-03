use ratatui::prelude::*;
use ratatui::widgets::*;

use futures::executor::block_on;

use crate::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let Some(host_id) = &app.selected_id else {
        let paragraph = Paragraph::new("No host selected")
            .block(Block::default().title("Host Details").borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    };

    let cpu_detail = block_on(app.details_states.cpu.get(host_id));
    let mem_detail = block_on(app.details_states.mem.get(host_id));
    let host_info = {
        let hosts = block_on(app.ssh_hosts.lock());
        hosts.get(host_id).cloned()
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    let info_block = Block::default().title("Host Info").borders(Borders::ALL);
    let info_inner = info_block.inner(chunks[0]);
    frame.render_widget(info_block, chunks[0]);

    if let Some(info) = host_info {
        let lines = [format!("Name: {}", info.name),
            format!("User: {}", info.user),
            format!("Host: {}:{}", info.ip, info.port)];
        let paragraph = Paragraph::new(lines.join("\n"))
            .style(Style::default())
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, info_inner);
    } else {
        let paragraph = Paragraph::new("No host info")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, info_inner);
    }

    let cpu_block = Block::default().title("CPU").borders(Borders::ALL);
    let cpu_inner = cpu_block.inner(chunks[1]);
    frame.render_widget(cpu_block, chunks[1]);

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
            .map(|(i, usage)| render_bar(&format!("c{}", i), *usage))
            .collect();

        // Split core_lines into multiple columns (e.g., 2 or 3 per row)
        let cols = 3;
        for chunk in core_lines.chunks(cols) {
            lines.push(chunk.join("   "));
        }

        let paragraph = Paragraph::new(lines.join("\n"))
            .style(Style::default())
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, cpu_inner);
    } else {
        let paragraph = Paragraph::new("No CPU data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, cpu_inner);
    }

    let mem_block = Block::default().title("Memory").borders(Borders::ALL);
    let mem_inner = mem_block.inner(chunks[2]);
    frame.render_widget(mem_block, chunks[2]);

    if let Some(mem) = mem_detail {
        let lines = [format!("Total: {:.1}GB", mem.total_mb as f64 / 1024.0),
            render_bar("Use", mem.used_percent),
            format!("Used: {:.1}GB", mem.used_mb as f64 / 1024.0),
            format!("Free: {:.1}GB", mem.free_mb as f64 / 1024.0)];

        let paragraph = Paragraph::new(lines.join("\n"))
            .style(Style::default())
            .alignment(Alignment::Left);
        frame.render_widget(paragraph, mem_inner);
    } else {
        let paragraph = Paragraph::new("No Mem data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, mem_inner);
    }
}

fn render_bar(label: &str, percent: f32) -> String {
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
