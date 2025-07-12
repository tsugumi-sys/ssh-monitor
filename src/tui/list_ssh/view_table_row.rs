use super::themed_table::TableColors;
use crate::backend::jobs::{cpu::CpuInfo, disk::DiskInfo, mem::MemInfo};
use crate::ssh_config::SshHostInfo;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[allow(clippy::too_many_arguments)]
pub fn render(
    i: usize,
    info: &SshHostInfo,
    cpu: Option<&CpuInfo>,
    mem: Option<&MemInfo>,
    disk: Option<&DiskInfo>,
    colors: &TableColors,
) -> Row<'static> {
    let bg = if i % 2 == 0 {
        colors.normal_row_color
    } else {
        colors.alt_row_color
    };

    let user_at_host = format!("{}@{}:{}", info.user, info.ip, info.port);

    let cpu_cell = cpu
        .map(|c| format!("{:.1}%", c.usage_percent))
        .unwrap_or_else(|| "-".into());
    let mem_cell = mem
        .map(|m| format!("{:.1}%", m.used_percent))
        .unwrap_or_else(|| "-".into());
    let disk_cell = disk
        .map(|d| format!("{:.1}%", d.used_percent))
        .unwrap_or_else(|| "-".into());

    Row::new(vec![
        Cell::from(info.name.clone()),
        Cell::from(user_at_host),
        Cell::from(cpu_cell),
        Cell::from(mem_cell),
        Cell::from(disk_cell),
    ])
    .style(Style::default().bg(bg))
    .height(2)
}
