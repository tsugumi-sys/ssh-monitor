use ratatui::prelude::*;

pub struct TableColors {
    pub header_bg: Color,
    pub header_fg: Color,
    pub row_fg: Color,
    pub normal_row_color: Color,
    pub alt_row_color: Color,
    pub selected_row_style: Style,
    pub footer_border_color: Color,
}

impl TableColors {
    pub fn default() -> Self {
        Self {
            header_bg: Color::DarkGray,
            header_fg: Color::White,
            row_fg: Color::Gray,
            normal_row_color: Color::Black,
            alt_row_color: Color::Black,
            selected_row_style: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue),
            footer_border_color: Color::Blue,
        }
    }
}
