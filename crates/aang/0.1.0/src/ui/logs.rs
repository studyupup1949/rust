//! Logs tab rendering

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::App;

pub fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .logs
        .iter()
        .skip(app.log_scroll)
        .map(|entry| {
            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();
            let level_color = entry.level.color();
            let icon = entry.level.icon();

            let mut spans = vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", icon),
                    Style::default()
                        .fg(level_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&entry.message, Style::default().fg(Color::White)),
            ];

            if let Some(details) = &entry.details {
                spans.push(Span::styled(
                    format!(" - {}", details),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Activity Log ({}/{}) ",
                    app.log_scroll + 1,
                    app.logs.len().max(1)
                ))
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}
