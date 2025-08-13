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
    let disk_detail = block_on(app.details_states.disk.get(host_id));
    let host_info = {
        let hosts = block_on(app.ssh_hosts.lock());
        hosts.get(host_id).cloned()
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
        ])
        .split(area);

    let info_block = Block::default().title("Host Info").borders(Borders::ALL);
    let info_inner = info_block.inner(chunks[0]);
    frame.render_widget(info_block, chunks[0]);

    if let Some(info) = host_info {
        let lines = [
            format!("Name: {}", info.name),
            format!("User: {}", info.user),
            format!("Host: {}:{}", info.ip, info.port),
        ];
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
        let lines = [
            format!("Total: {:.1}GB", mem.total_mb as f64 / 1024.0),
            render_bar("Use", mem.used_percent),
            format!("Used: {:.1}GB", mem.used_mb as f64 / 1024.0),
            format!("Free: {:.1}GB", mem.free_mb as f64 / 1024.0),
        ];

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

    let disk_block = Block::default().title("Disk").borders(Borders::ALL);
    let disk_inner = disk_block.inner(chunks[3]);
    frame.render_widget(disk_block, chunks[3]);

    if let Some(disk) = disk_detail {
        // Calculate comprehensive overview metrics
        let free_mb = disk.total_mb.saturating_sub(disk.used_mb);
        let total_gb = disk.total_mb as f64 / 1024.0;
        let used_gb = disk.used_mb as f64 / 1024.0;
        let free_gb = free_mb as f64 / 1024.0;

        // Try to get detailed volume data for enhanced overview
        let (volume_count, overview_content) =
            if let Ok(volumes) = block_on(app.details_states.disk.get_volumes(host_id, &app.db)) {
                if !volumes.is_empty() {
                    // Enhanced overview with volume count and detailed metrics
                    let volume_count = volumes.len();
                    let content = vec![
                        format!("Storage Overview ({} volumes)", volume_count),
                        format!("Total Capacity: {:.1} GB", total_gb),
                        render_bar("Usage", disk.used_percent),
                        format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                    ];
                    (volume_count, content)
                } else {
                    // Simple overview if no volumes
                    let content = vec![
                        format!("Total: {:.1} GB", total_gb),
                        render_bar("Usage", disk.used_percent),
                        format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                    ];
                    (0, content)
                }
            } else {
                // Fallback overview
                let content = vec![
                    format!("Total: {:.1} GB", total_gb),
                    render_bar("Usage", disk.used_percent),
                    format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                ];
                (0, content)
            };

        if volume_count > 0 {
            // Split the disk area into overview, spacer, and table sections
            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Overview
                    Constraint::Length(1), // Spacer
                    Constraint::Min(0),    // Table takes remaining space
                ])
                .split(disk_inner);

            // Render enhanced overview
            let overview_paragraph = Paragraph::new(overview_content.join(
                "
",
            ))
            .style(Style::default())
            .alignment(Alignment::Left);
            frame.render_widget(overview_paragraph, disk_chunks[0]);

            // Render spacer (empty line)
            let spacer = Paragraph::new("").style(Style::default());
            frame.render_widget(spacer, disk_chunks[1]);

            // Get volumes again for table rendering
            if let Ok(volumes) = block_on(app.details_states.disk.get_volumes(host_id, &app.db)) {
                // Sort volumes by usage percentage (descending) and take top 5
                let mut sorted_volumes = volumes.clone();
                sorted_volumes.sort_by(|a, b| {
                    b.used_percent
                        .partial_cmp(&a.used_percent)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_volumes: Vec<_> = sorted_volumes.into_iter().take(5).collect();

                // Create table with border and title
                let table_block = Block::default()
                    .title("Volume Details (Top 5 by Usage)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray));
                let table_inner = table_block.inner(disk_chunks[2]);
                frame.render_widget(table_block, disk_chunks[2]);

                // Create table headers
                let header = Row::new(vec!["Mount Point", "Size", "Used", "Avail", "Use%"])
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .height(1);

                // Create table rows from top volumes data with improved formatting
                let rows: Vec<Row> = top_volumes
                    .iter()
                    .map(|vol| {
                        let row_style = if vol.used_percent > 90.0 {
                            Style::default().fg(Color::Red)
                        } else if vol.used_percent > 75.0 {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        };

                        Row::new(vec![
                            vol.mount_point.clone(),
                            format!("{:.1}G", vol.total_mb as f64 / 1024.0),
                            format!("{:.1}G", vol.used_mb as f64 / 1024.0),
                            format!("{:.1}G", vol.available_mb as f64 / 1024.0),
                            format!("{:.1}%", vol.used_percent),
                        ])
                        .style(row_style)
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Percentage(35), // Mount Point
                        Constraint::Percentage(16), // Size
                        Constraint::Percentage(16), // Used
                        Constraint::Percentage(16), // Available
                        Constraint::Percentage(17), // Use%
                    ],
                )
                .header(header)
                .block(Block::default());

                frame.render_widget(table, table_inner);
            }
        } else {
            // Show only overview if no volume data
            let paragraph = Paragraph::new(overview_content.join(
                "
",
            ))
            .style(Style::default())
            .alignment(Alignment::Left);
            frame.render_widget(paragraph, disk_inner);
        }
    } else {
        let paragraph = Paragraph::new("No Disk data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, disk_inner);
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
