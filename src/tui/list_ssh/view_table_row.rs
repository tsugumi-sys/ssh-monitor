use super::themed_table::TableColors;
use crate::ssh_config::SshHostInfo;
use crate::tui::list_ssh::states::{CpuSnapshot, DiskSnapshot, MemSnapshot};
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Render a single table row for a given SSH host and optional CPU snapshot.
pub fn render(
    i: usize,
    info: &SshHostInfo,
    colors: &TableColors,
    cpu: &Option<CpuSnapshot>,
    mem: &Option<MemSnapshot>,
    disk: &Option<DiskSnapshot>,
) -> Row<'static> {
    // Alternate row background
    let bg = if i % 2 == 0 {
        colors.normal_row_color
    } else {
        colors.alt_row_color
    };

    // Format user@host:port
    let user_at_host = format!("{}@{}:{}", info.user, info.ip, info.port);

    // Format CPU usage text
    let cpu_text = cpu
        .as_ref()
        .map(|c| format!("{:.1}% / {}cores", c.usage_percent, c.core_count))
        .unwrap_or_else(|| "-".to_string());

    // Format memory usage text
    let mem_text = mem
        .as_ref()
        .map(|m| {
            let gb = m.total_mb as f64 / 1024.0;
            if gb >= 1024.0 {
                format!("{:.1}% / {:.1}TB", m.used_percent, gb / 1024.0)
            } else {
                format!("{:.1}% / {:.1}GB", m.used_percent, gb)
            }
        })
        .unwrap_or_else(|| "-".to_string());

    // Format disk usage text
    let disk_text = disk
        .as_ref()
        .map(|d| {
            let gb = d.total_mb as f64 / 1024.0;
            if gb >= 1024.0 {
                format!("{:.1}% / {:.1}TB", d.used_percent, gb / 1024.0)
            } else {
                format!("{:.1}% / {:.1}GB", d.used_percent, gb)
            }
        })
        .unwrap_or_else(|| "-".to_string());

    Row::new(vec![
        Cell::from(info.name.clone()),
        Cell::from(user_at_host),
        Cell::from(cpu_text),
        Cell::from(mem_text),
        Cell::from(disk_text),
    ])
    .style(Style::default().bg(bg))
    .height(2)
}
