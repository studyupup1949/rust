//! Dashboard tab rendering

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

use crate::app::App;

pub fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Stats cards
            Constraint::Length(10), // Charts
            Constraint::Min(0),     // Summary
        ])
        .split(area);

    render_stats_cards(frame, app, chunks[0]);
    render_charts(frame, app, chunks[1]);
    render_summary(frame, app, chunks[2]);
}

fn render_stats_cards(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(area);

    // Total Accounts Card
    let accounts_card = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{}", app.stats.total_accounts),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Total Accounts",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("{} active", app.stats.active_accounts),
            Style::default().fg(Color::Green),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Tracked ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(accounts_card, chunks[0]);

    // Rent Locked Card
    let locked_card = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{:.4} SOL", app.stats.locked_sol()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Rent Locked",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("{} lamports", app.stats.total_rent_locked),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Locked ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(locked_card, chunks[1]);

    // Reclaimable Card
    let reclaimable_sol = app.stats.potential_reclaim_sol();
    let reclaimable_card = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{:.4} SOL", reclaimable_sol),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Reclaimable",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("{} accounts", app.stats.reclaimable_accounts),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Available ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(reclaimable_card, chunks[2]);

    // Reclaimed Card
    let reclaimed_card = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{:.4} SOL", app.stats.reclaimed_sol()),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Reclaimed", Style::default().fg(Color::Gray))),
        Line::from(Span::styled(
            format!("{} operations", app.stats.reclaim_operations),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Recovered ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(reclaimed_card, chunks[3]);
}

fn render_charts(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    // Account status distribution
    let active = app.stats.active_accounts as u64;
    let reclaimable = app.stats.reclaimable_accounts as u64;
    let closed = app.stats.closed_accounts as u64;
    let other = app.stats.total_accounts.saturating_sub(
        app.stats.active_accounts + app.stats.reclaimable_accounts + app.stats.closed_accounts,
    ) as u64;

    let bar_group = BarGroup::default().bars(&[
        Bar::default()
            .value(active)
            .label("Active".into())
            .style(Style::default().fg(Color::Green)),
        Bar::default()
            .value(reclaimable)
            .label("Reclaimable".into())
            .style(Style::default().fg(Color::Yellow)),
        Bar::default()
            .value(closed)
            .label("Closed".into())
            .style(Style::default().fg(Color::DarkGray)),
        Bar::default()
            .value(other)
            .label("Other".into())
            .style(Style::default().fg(Color::Blue)),
    ]);

    let chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Account Status ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(bar_group)
        .bar_width(8)
        .bar_gap(2)
        .value_style(Style::default().fg(Color::White))
        .label_style(Style::default().fg(Color::Gray));

    frame.render_widget(chart, chunks[0]);

    // Recent activity sparkline (would need historical data)
    let data: Vec<u64> = vec![0; 20]; // Placeholder

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Rent Reclaimed (Recent) ")
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(&data)
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(sparkline, chunks[1]);
}

fn render_summary(frame: &mut Frame, app: &App, area: Rect) {
    let last_scan = app
        .stats
        .last_scan
        .map(|t| t.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Never".to_string());

    let fee_payer = app
        .config
        .operator
        .fee_payer
        .as_ref()
        .map(|s| {
            if s.len() > 20 {
                format!("{}...{}", &s[..8], &s[s.len() - 4..])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "Not configured".to_string());

    let treasury = app
        .config
        .operator
        .treasury
        .as_ref()
        .map(|s| {
            if s.len() > 20 {
                format!("{}...{}", &s[..8], &s[s.len() - 4..])
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "Not configured".to_string());

    let summary = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Last Scan: ", Style::default().fg(Color::Gray)),
            Span::styled(&last_scan, Style::default().fg(Color::White)),
            Span::raw("  |  "),
            Span::styled("Fee Payer: ", Style::default().fg(Color::Gray)),
            Span::styled(&fee_payer, Style::default().fg(Color::Cyan)),
            Span::raw("  |  "),
            Span::styled("Treasury: ", Style::default().fg(Color::Gray)),
            Span::styled(&treasury, Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Network: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &app.config.network.rpc_url,
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  |  "),
            Span::styled("Auto-scan: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if app.auto_scan_enabled {
                    "Enabled"
                } else {
                    "Disabled"
                },
                Style::default().fg(if app.auto_scan_enabled {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw("  |  "),
            Span::styled("Scan interval: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}s", app.config.monitoring.scan_interval_secs),
                Style::default().fg(Color::White),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Configuration ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(summary, area);
}
