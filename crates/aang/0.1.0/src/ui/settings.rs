//! Settings tab rendering

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render_settings(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    render_network_settings(frame, app, chunks[0]);
    render_monitoring_settings(frame, app, chunks[1]);
}

fn render_network_settings(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    // Network config
    let network = &app.config.network;
    let network_content = vec![
        Line::from(vec![
            Span::styled("RPC URL: ", Style::default().fg(Color::Gray)),
            Span::styled(&network.rpc_url, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("WebSocket: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network.ws_url.as_deref().unwrap_or("Not configured"),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Commitment: ", Style::default().fg(Color::Gray)),
            Span::styled(&network.commitment, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Timeout: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}s", network.timeout_secs),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let network_widget = Paragraph::new(network_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(network_widget, chunks[0]);

    // Operator config
    let operator = &app.config.operator;
    let operator_content = vec![
        Line::from(vec![
            Span::styled("Fee Payer: ", Style::default().fg(Color::Gray)),
            Span::styled(
                operator.fee_payer.as_deref().unwrap_or("Not configured"),
                Style::default().fg(if operator.fee_payer.is_some() {
                    Color::Cyan
                } else {
                    Color::Red
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Treasury: ", Style::default().fg(Color::Gray)),
            Span::styled(
                operator.treasury.as_deref().unwrap_or("Not configured"),
                Style::default().fg(if operator.treasury.is_some() {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Keypair: ", Style::default().fg(Color::Gray)),
            Span::styled(
                operator
                    .keypair_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "Not configured".to_string()),
                Style::default().fg(if operator.keypair_path.is_some() {
                    Color::White
                } else {
                    Color::Yellow
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Dry Run: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if operator.dry_run {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Style::default()
                    .fg(if operator.dry_run {
                        Color::Yellow
                    } else {
                        Color::Green
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let operator_widget = Paragraph::new(operator_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Operator ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(operator_widget, chunks[1]);
}

fn render_monitoring_settings(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    // Monitoring config
    let monitoring = &app.config.monitoring;
    let monitoring_content = vec![
        Line::from(vec![
            Span::styled("Scan Interval: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}s", monitoring.scan_interval_secs),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Auto Scan: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if monitoring.auto_scan {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Style::default().fg(if monitoring.auto_scan {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Inactivity Threshold: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} days", monitoring.inactivity_threshold_days),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Min Reclaim: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} lamports", monitoring.min_reclaim_lamports),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Max Accounts/Scan: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", monitoring.max_accounts_per_scan),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Whitelist: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} accounts", monitoring.whitelist.len()),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let monitoring_widget = Paragraph::new(monitoring_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Monitoring ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(monitoring_widget, chunks[0]);

    // Alert config
    let alerts = &app.config.alerts;
    let alerts_content = vec![
        Line::from(vec![
            Span::styled("Alerts: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if alerts.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Style::default().fg(if alerts.enabled {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Large Idle Threshold: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} SOL", alerts.large_idle_threshold_sol),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Idle Count Threshold: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} accounts", alerts.idle_count_threshold),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Alert on Failure: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if alerts.alert_on_failure { "Yes" } else { "No" },
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let alerts_widget = Paragraph::new(alerts_content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Alerts ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(alerts_widget, chunks[1]);
}
