use chrono;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct TimelineChart<'a> {
    pub title: &'a str,
    pub data: Vec<(String, f32, String)>, // (host_id, value, timestamp)
    pub host_id: &'a str,
    pub y_bounds: (f64, f64),
    pub y_unit: &'a str,
    pub color: Color,
}

impl<'a> TimelineChart<'a> {
    pub fn new(title: &'a str, host_id: &'a str) -> Self {
        Self {
            title,
            data: Vec::new(),
            host_id,
            y_bounds: (0.0, 100.0),
            y_unit: "%",
            color: Color::Cyan,
        }
    }

    pub fn data(mut self, data: Vec<(String, f32, String)>) -> Self {
        self.data = data;
        self
    }

    pub fn y_bounds(mut self, bounds: (f64, f64)) -> Self {
        self.y_bounds = bounds;
        self
    }

    pub fn y_unit(mut self, unit: &'a str) -> Self {
        self.y_unit = unit;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.data.is_empty() {
            let no_data = Paragraph::new("Loading timeline data...")
                .block(Block::default().title(self.title).borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(no_data, area);
            return;
        }

        let host_data_with_time: Vec<_> = self
            .data
            .iter()
            .filter(|(id, _, _)| id == self.host_id)
            .collect();

        if host_data_with_time.is_empty() {
            let no_data = Paragraph::new("No timeline data for this host")
                .block(Block::default().title(self.title).borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(no_data, area);
            return;
        }

        let chart_data: Vec<_> = host_data_with_time
            .iter()
            .rev() // Reverse to show oldest -> newest (left -> right)
            .enumerate()
            .map(|(i, &(_, value, _))| (i as f64, *value as f64))
            .collect();

        let data_count = chart_data.len();
        let max_x = (data_count - 1) as f64;

        let x_labels = self.create_time_labels(&host_data_with_time, data_count);

        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(self.color))
                .graph_type(GraphType::Line)
                .data(&chart_data),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default().title(self.title.to_string()))
            .x_axis(
                Axis::default()
                    .title("")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, max_x])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title(self.y_unit)
                    .style(Style::default().fg(Color::Gray))
                    .bounds([self.y_bounds.0, self.y_bounds.1])
                    .labels(self.create_y_labels()),
            );

        frame.render_widget(chart, area);
    }

    fn create_time_labels(
        &self,
        host_data_with_time: &[&(String, f32, String)],
        data_count: usize,
    ) -> Vec<String> {
        if data_count > 3 {
            let indices = [0, data_count / 3, (data_count * 2) / 3, data_count - 1];
            indices
                .iter()
                .map(|&idx| {
                    let original_idx = host_data_with_time.len() - 1 - idx;
                    if original_idx < host_data_with_time.len() {
                        let timestamp = &host_data_with_time[original_idx].2;

                        if let Ok(parsed) =
                            chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
                        {
                            let parsed_utc = parsed.and_utc();
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(parsed_utc);
                            let minutes_ago = duration.num_minutes();

                            if minutes_ago < 1 {
                                "now".to_string()
                            } else if minutes_ago < 60 {
                                format!("{}m", minutes_ago)
                            } else {
                                let hours_ago = minutes_ago / 60;
                                if hours_ago < 24 {
                                    format!("{}h", hours_ago)
                                } else {
                                    let days_ago = hours_ago / 24;
                                    format!("{}d", days_ago)
                                }
                            }
                        } else if timestamp.len() >= 5 {
                            timestamp[timestamp.len() - 5..].to_string()
                        } else {
                            format!("{}", idx)
                        }
                    } else {
                        format!("{}", idx)
                    }
                })
                .collect()
        } else {
            vec!["start".to_string(), "end".to_string()]
        }
    }

    fn create_y_labels(&self) -> Vec<String> {
        let (min, max) = self.y_bounds;
        let step = (max - min) / 4.0;
        (0..5)
            .map(|i| {
                let value = min + (i as f64 * step);
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{:.1}", value)
                }
            })
            .collect()
    }
}
