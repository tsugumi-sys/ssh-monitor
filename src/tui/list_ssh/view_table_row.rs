use super::themed_table::TableColors;
use crate::ssh_config::SshHostInfo;
use crate::tui::list_ssh::states::CpuSnapshot;
use ratatui::prelude::*;
use ratatui::widgets::*;

/// Render a single table row for a given SSH host and optional CPU snapshot.
pub fn render(
    i: usize,
    info: &SshHostInfo,
    colors: &TableColors,
    cpu: &Option<CpuSnapshot>,
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

    Row::new(vec![
        Cell::from(info.name.clone()),
        Cell::from(user_at_host),
        Cell::from(cpu_text),
    ])
    .style(Style::default().bg(bg))
    .height(2)
}
