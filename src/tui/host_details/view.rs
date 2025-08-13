use ratatui::prelude::*;
use ratatui::widgets::*;

use futures::executor::block_on;

use super::timeline_chart::TimelineChart;
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
    let gpu_detail = block_on(app.details_states.gpu.get(host_id));
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

    let info_block = Block::default()
        .title("Host Info")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let info_inner = info_block.inner(chunks[0]);
    frame.render_widget(info_block, chunks[0]);

    if let Some(info) = host_info {
        let lines = [
            format!("Name: {}", info.name),
            format!("User: {}@{}:{}", info.user, info.ip, info.port),
            format!("Identity: {}", info.identity_file),
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

    let cpu_mem_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let cpu_block = Block::default()
        .title("CPU")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let cpu_inner = cpu_block.inner(cpu_mem_chunks[0]);
    frame.render_widget(cpu_block, cpu_mem_chunks[0]);

    if let Some(cpu) = cpu_detail {
        let cpu_timeline = block_on(app.details_states.cpu_timeline.get());

        let cpu_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Gauge (model + usage)
                Constraint::Length(4), // Per-core usage
                Constraint::Length(2), // Additional info
                Constraint::Min(8),    // Timeline chart (takes remaining space, min 8 lines)
            ])
            .split(cpu_inner);

        let gauge_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Model name
                Constraint::Length(2), // Usage gauge
            ])
            .split(cpu_sections[0]);

        let model_paragraph = Paragraph::new(format!("Model: {}", cpu.model_name))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        frame.render_widget(model_paragraph, gauge_sections[0]);

        let cpu_bar = render_wide_bar("CPU", cpu.usage_percent);
        let cpu_paragraph = Paragraph::new(format!(
            "Total CPU Usage
{}",
            cpu_bar
        ))
        .block(Block::default())
        .style(Style::default())
        .alignment(Alignment::Left);
        frame.render_widget(cpu_paragraph, gauge_sections[1]);

        let core_lines: Vec<String> = cpu
            .per_core
            .iter()
            .enumerate()
            .map(|(i, usage)| render_bar(&format!("c{}", i), *usage))
            .collect();

        let cols = 3;
        let mut core_display = Vec::new();
        for chunk in core_lines.chunks(cols) {
            core_display.push(chunk.join("   "));
        }

        let cores_paragraph = Paragraph::new(core_display.join(
            "
",
        ))
        .block(Block::default().title("Per-Core Usage"))
        .style(Style::default())
        .alignment(Alignment::Left);
        frame.render_widget(cores_paragraph, cpu_sections[1]);

        let cpu_info = Paragraph::new(format!("Cores: {}", cpu.per_core.len()))
            .style(Style::default())
            .alignment(Alignment::Left);
        frame.render_widget(cpu_info, cpu_sections[2]);

        let timeline_chart = TimelineChart::new("CPU Usage", host_id)
            .data(cpu_timeline.timeline_data.clone())
            .y_bounds((0.0, 100.0))
            .y_unit("%")
            .color(Color::Cyan);

        timeline_chart.render(frame, cpu_sections[3]);
    } else {
        let paragraph = Paragraph::new("No CPU data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, cpu_inner);
    }

    let mem_block = Block::default()
        .title("Memory")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let mem_inner = mem_block.inner(cpu_mem_chunks[1]);
    frame.render_widget(mem_block, cpu_mem_chunks[1]);

    if let Some(mem) = mem_detail {
        let mem_timeline = block_on(app.details_states.mem_timeline.get());

        let mem_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Gauge
                Constraint::Length(6), // Details (combined to match CPU's per-core + info sections)
                Constraint::Min(8),    // Timeline chart (takes remaining space, min 8 lines)
            ])
            .split(mem_inner);

        let mem_bar = render_wide_bar("Mem", mem.used_percent);
        let mem_paragraph = Paragraph::new(format!(
            "Memory: {:.1}GB ({:.1}%)
{}",
            mem.total_mb as f64 / 1024.0,
            mem.used_percent,
            mem_bar
        ))
        .block(Block::default())
        .style(Style::default())
        .alignment(Alignment::Left);
        frame.render_widget(mem_paragraph, mem_sections[0]);

        let lines = [
            format!("Used: {:.1}GB", mem.used_mb as f64 / 1024.0),
            format!("Free: {:.1}GB", mem.free_mb as f64 / 1024.0),
            format!(
                "Available: {:.1}GB",
                (mem.total_mb - mem.used_mb) as f64 / 1024.0
            ),
        ];

        let details_paragraph = Paragraph::new(lines.join(
            "
",
        ))
        .style(Style::default())
        .alignment(Alignment::Left);
        frame.render_widget(details_paragraph, mem_sections[1]);

        let mem_timeline_chart = TimelineChart::new("Memory Usage", host_id)
            .data(mem_timeline.timeline_data.clone())
            .y_bounds((0.0, 100.0))
            .y_unit("%")
            .color(Color::Green);

        mem_timeline_chart.render(frame, mem_sections[2]);
    } else {
        let paragraph = Paragraph::new("No Mem data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, mem_inner);
    }

    let gpu_disk_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    let disk_block = Block::default()
        .title("Disk")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let disk_inner = disk_block.inner(gpu_disk_chunks[1]);
    frame.render_widget(disk_block, gpu_disk_chunks[1]);

    if let Some(disk) = disk_detail {
        let free_mb = disk.total_mb.saturating_sub(disk.used_mb);
        let total_gb = disk.total_mb as f64 / 1024.0;
        let used_gb = disk.used_mb as f64 / 1024.0;
        let free_gb = free_mb as f64 / 1024.0;

        let (volume_count, overview_content) =
            if let Ok(volumes) = block_on(app.details_states.disk.get_volumes(host_id, &app.db)) {
                if !volumes.is_empty() {
                    let volume_count = volumes.len();
                    let content = vec![
                        format!("Storage Overview ({} volumes)", volume_count),
                        format!("Total Capacity: {:.1} GB", total_gb),
                        render_wide_bar("Usage", disk.used_percent),
                        format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                    ];
                    (volume_count, content)
                } else {
                    let content = vec![
                        format!("Total: {:.1} GB", total_gb),
                        render_wide_bar("Usage", disk.used_percent),
                        format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                    ];
                    (0, content)
                }
            } else {
                let content = vec![
                    format!("Total: {:.1} GB", total_gb),
                    render_bar("Usage", disk.used_percent),
                    format!("Used: {:.1} GB  •  Free: {:.1} GB", used_gb, free_gb),
                ];
                (0, content)
            };

        if volume_count > 0 {
            let disk_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Overview
                    Constraint::Length(1), // Spacer
                    Constraint::Min(0),    // Table takes remaining space
                ])
                .split(disk_inner);

            let overview_paragraph = Paragraph::new(overview_content.join(
                "
",
            ))
            .style(Style::default())
            .alignment(Alignment::Left);
            frame.render_widget(overview_paragraph, disk_chunks[0]);

            let spacer = Paragraph::new("").style(Style::default());
            frame.render_widget(spacer, disk_chunks[1]);

            if let Ok(volumes) = block_on(app.details_states.disk.get_volumes(host_id, &app.db)) {
                let mut sorted_volumes = volumes.clone();
                sorted_volumes.sort_by(|a, b| {
                    b.used_percent
                        .partial_cmp(&a.used_percent)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_volumes: Vec<_> = sorted_volumes.into_iter().take(5).collect();

                let table_block = Block::default().title("Volume Details (Top 5 by Usage)");
                let table_inner = table_block.inner(disk_chunks[2]);
                frame.render_widget(table_block, disk_chunks[2]);

                let header = Row::new(vec!["Mount Point", "Size", "Used", "Avail", "Use%"])
                    .style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .height(1);

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

    let gpu_block = Block::default()
        .title("GPU")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let gpu_inner = gpu_block.inner(gpu_disk_chunks[0]);
    frame.render_widget(gpu_block, gpu_disk_chunks[0]);

    if let Some(gpus) = gpu_detail {
        if !gpus.is_empty() {
            if gpus.len() == 1 {
                let gpu = &gpus[0];
                let gpu_sections = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Memory Gauge
                        Constraint::Length(3), // Temperature Gauge
                        Constraint::Min(0),    // Info
                    ])
                    .split(gpu_inner);

                if let (Some(total_mb), Some(used_mb)) = (gpu.memory_total_mb, gpu.memory_used_mb) {
                    let usage_percent = if total_mb > 0 {
                        (used_mb as f32 / total_mb as f32) * 100.0
                    } else {
                        0.0
                    };

                    let mem_bar = render_wide_bar("Mem", usage_percent);
                    let mem_paragraph = Paragraph::new(format!(
                        "GPU {}: {} - Memory {:.1}GB
{}",
                        gpu.index,
                        gpu.name,
                        total_mb as f64 / 1024.0,
                        mem_bar
                    ))
                    .block(Block::default())
                    .style(Style::default())
                    .alignment(Alignment::Left);
                    frame.render_widget(mem_paragraph, gpu_sections[0]);
                } else {
                    let no_mem_info = Paragraph::new(format!(
                        "GPU {}: {}
Memory: N/A",
                        gpu.index, gpu.name
                    ))
                    .block(Block::default())
                    .alignment(Alignment::Left);
                    frame.render_widget(no_mem_info, gpu_sections[0]);
                }

                if let Some(temp) = gpu.temperature_c {
                    let temp_paragraph = Paragraph::new(format!("Temperature: {:.1}°C", temp))
                        .block(Block::default())
                        .style(Style::default())
                        .alignment(Alignment::Left);
                    frame.render_widget(temp_paragraph, gpu_sections[1]);
                } else {
                    let no_temp_info = Paragraph::new("Temperature: N/A")
                        .block(Block::default())
                        .alignment(Alignment::Left);
                    frame.render_widget(no_temp_info, gpu_sections[1]);
                }

                let info_lines = [
                    format!("Index: {}", gpu.index),
                    format!("Name: {}", gpu.name),
                ];
                let info_paragraph = Paragraph::new(info_lines.join(
                    "
",
                ))
                .style(Style::default())
                .alignment(Alignment::Left);
                frame.render_widget(info_paragraph, gpu_sections[2]);
            } else {
                let mut lines = vec![format!("GPU Count: {}", gpus.len())];

                for gpu in &gpus {
                    lines.push(format!("┌─ GPU {}: {}", gpu.index, gpu.name));

                    if let (Some(total_mb), Some(used_mb)) =
                        (gpu.memory_total_mb, gpu.memory_used_mb)
                    {
                        let usage_percent = if total_mb > 0 {
                            (used_mb as f32 / total_mb as f32) * 100.0
                        } else {
                            0.0
                        };
                        lines.push(format!("│  Memory: {:.1}GB", total_mb as f64 / 1024.0));
                        lines.push(render_bar("│  Use", usage_percent));
                        lines.push(format!("│  Used: {:.1}GB", used_mb as f64 / 1024.0));
                    } else {
                        lines.push("│  Memory: N/A".to_string());
                    }

                    if let Some(temp) = gpu.temperature_c {
                        lines.push(format!("│  Temp: {:.1}°C", temp));
                    } else {
                        lines.push("│  Temp: N/A".to_string());
                    }

                    if gpu.index < gpus.len() as u32 - 1 {
                        lines.push("│".to_string());
                    }
                }

                let paragraph = Paragraph::new(lines.join(""))
                    .style(Style::default())
                    .alignment(Alignment::Left);
                frame.render_widget(paragraph, gpu_inner);
            }
        } else {
            let paragraph = Paragraph::new("No GPUs detected")
                .block(Block::default())
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, gpu_inner);
        }
    } else {
        let paragraph = Paragraph::new("No GPU data")
            .block(Block::default())
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, gpu_inner);
    }
}

fn render_bar(label: &str, percent: f32) -> String {
    let width = 8;
    let filled = (percent / 100.0 * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!(
        "{:>3} [{}{}] {:>3.0}%",
        label,
        "▓".repeat(filled),
        "░".repeat(empty),
        percent
    )
}

fn render_wide_bar(label: &str, percent: f32) -> String {
    let width = 20;
    let filled = (percent / 100.0 * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!(
        "{:>3} [{}{}] {:>4.1}%",
        label,
        "▓".repeat(filled),
        "░".repeat(empty),
        percent
    )
}
