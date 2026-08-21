//! Accounts tab rendering

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::app::App;
use crate::types::AccountStatus;

pub fn render_accounts(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Table
            Constraint::Length(8), // Details
        ])
        .split(area);

    render_accounts_table(frame, app, chunks[0]);
    render_account_details(frame, app, chunks[1]);
}

fn render_accounts_table(frame: &mut Frame, app: &App, area: Rect) {
    let header_cells = [
        "#",
        "Public Key",
        "Status",
        "Lamports",
        "SOL",
        "Last Checked",
        "Owner",
    ]
    .iter()
    .map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.accounts.iter().enumerate().map(|(i, account)| {
        let status_style = Style::default().fg(account.status.color());
        let selected_style = if i == app.account_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let cells = vec![
            Cell::from(format!("{}", i + 1)),
            Cell::from(account.short_pubkey()),
            Cell::from(account.status.to_string()).style(status_style),
            Cell::from(format!("{}", account.lamports)),
            Cell::from(format!("{:.6}", account.lamports_as_sol())),
            Cell::from(account.last_checked.format("%H:%M:%S").to_string()),
            Cell::from(format!("{}...", &account.owner.to_string()[..8])),
        ];

        Row::new(cells).style(selected_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),  // #
            Constraint::Length(16), // Public Key
            Constraint::Length(12), // Status
            Constraint::Length(14), // Lamports
            Constraint::Length(12), // SOL
            Constraint::Length(12), // Last Checked
            Constraint::Min(12),    // Owner
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Accounts ({}) ", app.accounts.len()))
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let mut state = TableState::default();
    state.select(Some(app.account_selected));

    frame.render_stateful_widget(table, area, &mut state);
}

fn render_account_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(account) = app.selected_account() {
        let status_color = account.status.color();

        vec![
            Line::from(vec![
                Span::styled("Public Key: ", Style::default().fg(Color::Gray)),
                Span::styled(account.pubkey.to_string(), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Owner: ", Style::default().fg(Color::Gray)),
                Span::styled(account.owner.to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    account.status.to_string(),
                    Style::default().fg(status_color),
                ),
                Span::raw("  |  "),
                Span::styled("Data Size: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} bytes", account.data_len),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Balance: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "{} lamports ({:.9} SOL)",
                        account.lamports,
                        account.lamports_as_sol()
                    ),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Sponsored: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    account.sponsored_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  |  "),
                Span::styled("Last Activity: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    account
                        .last_activity
                        .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![can_reclaim_text(account)]),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "No account selected",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [A] to add an account",
                Style::default().fg(Color::Gray),
            )),
        ]
    };

    let details = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Details ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(details, area);
}

fn can_reclaim_text(account: &crate::types::SponsoredAccount) -> Span<'static> {
    match account.status {
        AccountStatus::Empty | AccountStatus::Reclaimable | AccountStatus::Inactive => {
            Span::styled(
                "Eligible for reclaim - Press [R] to reclaim",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        }
        AccountStatus::Active => Span::styled(
            "Account is active - cannot reclaim",
            Style::default().fg(Color::DarkGray),
        ),
        AccountStatus::Closed => Span::styled(
            "Account already closed",
            Style::default().fg(Color::DarkGray),
        ),
        AccountStatus::Unknown => Span::styled(
            "Status unknown - scan to update",
            Style::default().fg(Color::Yellow),
        ),
    }
}
