//! Alerts tab rendering

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::types::AlertSeverity;

pub fn render_alerts(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Alert list
            Constraint::Length(6), // Alert details
        ])
        .split(area);

    render_alert_list(frame, app, chunks[0]);
    render_alert_details(frame, app, chunks[1]);
}

fn render_alert_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let severity_color = alert.severity.color();
            let severity_icon = match alert.severity {
                AlertSeverity::Low => "o",
                AlertSeverity::Medium => "*",
                AlertSeverity::High => "!",
                AlertSeverity::Critical => "X",
            };

            let ack_marker = if alert.acknowledged {
                Span::styled(" [ACK] ", Style::default().fg(Color::DarkGray))
            } else {
                Span::raw("")
            };

            let selected_style = if i == app.alert_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            let timestamp = alert.timestamp.format("%m-%d %H:%M").to_string();

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", severity_icon),
                    Style::default()
                        .fg(severity_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&alert.title, Style::default().fg(Color::White)),
                ack_marker,
            ]))
            .style(selected_style)
        })
        .collect();

    let unack_count = app.unacknowledged_alerts();
    let title = if unack_count > 0 {
        format!(" Alerts ({} unacknowledged) ", unack_count)
    } else {
        format!(" Alerts ({}) ", app.alerts.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

fn render_alert_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(alert) = app.alerts.get(app.alert_selected) {
        let severity_color = alert.severity.color();

        vec![
            Line::from(vec![
                Span::styled("Severity: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:?}", alert.severity),
                    Style::default()
                        .fg(severity_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  |  "),
                Span::styled("Time: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![Span::styled(
                &alert.message,
                Style::default().fg(Color::White),
            )]),
            Line::from(vec![Span::styled(
                format!("Related accounts: {}", alert.related_accounts.len()),
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![if alert.acknowledged {
                Span::styled("Acknowledged", Style::default().fg(Color::Green))
            } else {
                Span::styled(
                    "Press [Enter] to acknowledge",
                    Style::default().fg(Color::Yellow),
                )
            }]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No alerts",
            Style::default().fg(Color::Green),
        ))]
    };

    let details = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(details, area);
}
