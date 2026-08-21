//! TUI rendering module for Aang

mod accounts;
mod alerts;
mod dashboard;
mod logs;
mod settings;
mod widgets;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, InputMode, Tab};

pub use accounts::render_accounts;
pub use alerts::render_alerts;
pub use dashboard::render_dashboard;
pub use logs::render_logs;
pub use settings::render_settings;

/// Main UI rendering function
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render popups if in special input mode
    if app.input_mode != InputMode::Normal {
        render_popup(frame, app);
    }
}

/// Render the header with tabs
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(t.title(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Aang - Kora Rent Reclaim ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .highlight_style(Style::default().fg(Color::Cyan))
        .select(app.tab as usize);

    frame.render_widget(tabs, area);
}

/// Render main content based on active tab
fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    match app.tab {
        Tab::Dashboard => render_dashboard(frame, app, area),
        Tab::Accounts => render_accounts(frame, app, area),
        Tab::Logs => render_logs(frame, app, area),
        Tab::Alerts => render_alerts(frame, app, area),
        Tab::Settings => render_settings(frame, app, area),
    }
}

/// Render status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let alert_count = app.unacknowledged_alerts();
    let alert_indicator = if alert_count > 0 {
        Span::styled(
            format!(" {} alerts ", alert_count),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    } else {
        Span::raw("")
    };

    let scan_indicator = if app.is_scanning {
        Span::styled(
            " Scanning... ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        )
    } else {
        Span::raw("")
    };

    let dry_run_indicator = if app.config.operator.dry_run {
        Span::styled(
            " DRY RUN ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    } else {
        Span::raw("")
    };

    let network = Span::styled(
        format!(" {} ", &app.config.network.rpc_url),
        Style::default().fg(Color::DarkGray),
    );

    let help = Span::styled(app.help_text(), Style::default().fg(Color::Gray));

    let status_line = Line::from(vec![
        dry_run_indicator,
        Span::raw(" "),
        scan_indicator,
        alert_indicator,
        Span::raw(" "),
        help,
    ]);

    let status = Paragraph::new(status_line).block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, area);
}

/// Render popup dialogs
fn render_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 20, frame.area());

    // Clear the area
    frame.render_widget(ratatui::widgets::Clear, area);

    match app.input_mode {
        InputMode::AddAccount => {
            let popup = Paragraph::new(vec![
                Line::from("Enter account public key:"),
                Line::from(""),
                Line::from(Span::styled(
                    &app.input_buffer,
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "[Enter] Confirm  [Esc] Cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Add Account ")
                    .title_style(Style::default().fg(Color::Green)),
            );
            frame.render_widget(popup, area);
        }
        InputMode::ConfirmReclaim => {
            let account_info = if let Some(acc) = app.selected_account() {
                format!(
                    "{}\n{} lamports ({:.6} SOL)",
                    acc.short_pubkey(),
                    acc.lamports,
                    acc.lamports_as_sol()
                )
            } else {
                "No account selected".to_string()
            };

            let popup = Paragraph::new(vec![
                Line::from("Confirm reclaim:"),
                Line::from(""),
                Line::from(account_info),
                Line::from(""),
                Line::from(Span::styled(
                    "[y] Yes  [n] No  [Esc] Cancel",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm Reclaim ")
                    .title_style(Style::default().fg(Color::Yellow)),
            );
            frame.render_widget(popup, area);
        }
        InputMode::Command => {
            let popup = Paragraph::new(vec![
                Line::from("Enter command:"),
                Line::from(""),
                Line::from(Span::styled(
                    format!(":{}", &app.input_buffer),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Commands: scan, reclaim-all, export, import, quit",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Command ")
                    .title_style(Style::default().fg(Color::Magenta)),
            );
            frame.render_widget(popup, area);
        }
        _ => {}
    }
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
