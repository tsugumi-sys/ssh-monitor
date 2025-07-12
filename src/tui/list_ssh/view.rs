use super::themed_table::TableColors;
use super::view_table_row::render as render_table_row;
use crate::{App, AppMode};
use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::widgets::*;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let colors = TableColors::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    if app.mode == AppMode::Search {
        let input = Paragraph::new(app.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, chunks[0]);
    } else {
        let title = Paragraph::new("SSH Hosts Overview")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);
    }

    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());
    let hosts = &*hosts_guard;

    let connected = 0;
    let loading = 0;
    let failed = 0;

    let overview_lines = vec![Line::from(vec![
        Span::styled("● ", Style::default().fg(Color::Green)),
        Span::raw(format!("Connected: {}  ", connected)),
        Span::styled("● ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("Loading: {}  ", loading)),
        Span::styled("● ", Style::default().fg(Color::Red)),
        Span::raw(format!("Failed: {}", failed)),
    ])];

    let overview_block = Block::default()
        .borders(Borders::ALL)
        .title("Connection Summary")
        .border_style(Style::default().fg(Color::White));

    let overview = Paragraph::new(overview_lines)
        .block(overview_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(overview, chunks[1]);

    let grid_area = chunks[2];
    app.table_height = grid_area.height.saturating_sub(3) as usize;
    let mut host_entries: Vec<_> = hosts
        .iter()
        .filter(|(_, h)| {
            app.search_query.is_empty() || {
                let q = app.search_query.to_lowercase();
                h.name.to_lowercase().contains(&q)
                    || h.user.to_lowercase().contains(&q)
                    || h.ip.to_lowercase().contains(&q)
            }
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    host_entries.sort_by_key(|(_, h)| h.name.clone());
    app.visible_hosts = host_entries.clone();

    let visible_rows = grid_area.height.max(1) as usize;
    app.vertical_scroll_state = app
        .vertical_scroll_state
        .content_length(host_entries.len())
        .position(app.vertical_scroll);
    app.vertical_scroll = app
        .vertical_scroll
        .min(host_entries.len().saturating_sub(visible_rows));

    let start_index = app.vertical_scroll;
    let end_index = (start_index + visible_rows).min(host_entries.len());

    let rows = host_entries[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(i, (_id, info))| render_table_row(i, info, &colors));

    let header = Row::new(vec![Cell::from("Name"), Cell::from("User@Host:Port")]).style(
        Style::default()
            .fg(colors.header_fg)
            .bg(colors.header_bg)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(40),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("SSH Hosts"))
    .row_highlight_style(colors.selected_row_style)
    .highlight_symbol("▶ ")
    .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(table, grid_area, &mut app.table_state);

    let footer = Paragraph::new(vec![Line::from("ESC: Exit | ↑↓: Scroll | /: Search")])
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(colors.row_fg)
                .bg(colors.normal_row_color),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .border_style(Style::default().fg(colors.footer_border_color)),
        );

    frame.render_widget(footer, chunks[3]);
}
