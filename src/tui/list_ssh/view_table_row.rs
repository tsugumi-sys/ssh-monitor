use super::themed_table::TableColors;
use crate::ssh_config::SshHostInfo;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[allow(clippy::too_many_arguments)]
pub fn render(i: usize, info: &SshHostInfo, colors: &TableColors) -> Row<'static> {
    let bg = if i % 2 == 0 {
        colors.normal_row_color
    } else {
        colors.alt_row_color
    };

    let user_at_host = format!("{}@{}:{}", info.user, info.ip, info.port);

    Row::new(vec![
        Cell::from(info.name.clone()),
        Cell::from(user_at_host),
    ])
    .style(Style::default().bg(bg))
    .height(2)
}
