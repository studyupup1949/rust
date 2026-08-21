use crossterm::{
  cursor::Show,
  event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
  },
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::watch;
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  style::Color,
  Terminal,
};

use crate::fetch::fetch_filters::{AdGuardFilteringStatus, Filter};
use crate::fetch::fetch_query_log::Query;
use crate::fetch::fetch_stats::StatsResponse;
use crate::fetch::fetch_status::StatusResponse;

use crate::widgets::chart::{make_history_chart, prepare_chart_data};
use crate::widgets::filters::make_filters_list;
use crate::widgets::gauge::make_gauge;
use crate::widgets::list::make_list;
use crate::widgets::status::render_status_paragraph;
use crate::widgets::table::make_query_table;

pub async fn draw_ui(
  mut data_rx: tokio::sync::mpsc::Receiver<Vec<Query>>,
  mut stats_rx: tokio::sync::mpsc::Receiver<StatsResponse>,
  mut status_rx: tokio::sync::mpsc::Receiver<StatusResponse>,
  filters: AdGuardFilteringStatus,
  shutdown_tx: watch::Sender<bool>,
) -> Result<(), anyhow::Error> {
  // Guard restores the terminal on drop, even if we return early via `?`
  let _guard = TerminalGuard::new()?;
  let backend = CrosstermBackend::new(stdout());
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;

  // Handle quit keys (q / Ctrl+C) in a separate task, so input never blocks on data
  let shutdown_tx = Arc::new(shutdown_tx);
  let input_shutdown_tx = Arc::clone(&shutdown_tx);
  let input_task = tokio::spawn(async move {
    let mut reader = EventStream::new();
    let mut shutdown_rx = input_shutdown_tx.subscribe();
    loop {
      tokio::select! {
          maybe_event = reader.next() => {
              match maybe_event {
                  Some(Ok(Event::Key(key))) if is_quit_key(key) => {
                      let _ = input_shutdown_tx.send(true);
                      break;
                  }
                  Some(Ok(_)) => {}
                  Some(Err(_)) | None => break,
              }
          }
          // Stop if shutdown was triggered elsewhere (e.g. channels closed)
          _ = shutdown_rx.changed() => break,
      }
    }
  });

  let mut shutdown_rx = shutdown_tx.subscribe();

  loop {
    // Wait for the next batch of data, but bail out immediately on shutdown
    let data = tokio::select! {
        biased;
        _ = shutdown_rx.changed() => break,
        maybe_data = data_rx.recv() => match maybe_data {
            Some(data) => data,
            None => break,
        },
    };
    let mut stats = match stats_rx.recv().await {
      Some(stats) => stats,
      None => break,
    };
    let status = match status_rx.recv().await {
      Some(status) => status,
      None => break,
    };

    // Prepare the data for the chart
    prepare_chart_data(&mut stats);

    terminal.draw(|f| {
      let size = f.size();

      // Make the charts
      let gauge = make_gauge(&stats);
      let table = make_query_table(&data, size.width);
      let graph = make_history_chart(&stats);
      let paragraph = render_status_paragraph(&status, &stats);
      let filter_items: &[Filter] = filters.filters.as_deref().unwrap_or(&[]);
      let filters_list = make_filters_list(filter_items, size.width);
      let top_queried_domains = make_list(
        "Top Queried Domains",
        &stats.top_queried_domains,
        Color::Green,
        size.width,
      );
      let top_blocked_domains = make_list(
        "Top Blocked Domains",
        &stats.top_blocked_domains,
        Color::Red,
        size.width,
      );
      let top_clients = make_list("Top Clients", &stats.top_clients, Color::Cyan, size.width);

      let constraints = if size.height > 42 {
        vec![
          Constraint::Percentage(30),
          Constraint::Min(1),
          Constraint::Percentage(20),
        ]
      } else {
        vec![
          Constraint::Percentage(30),
          Constraint::Min(1),
          Constraint::Percentage(0),
        ]
      };

      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(&*constraints)
        .split(size);

      // Split the top part (charts + gauge) into left (gauge + block) and right (line chart)
      let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[0]);

      // Split the left part of top (gauge + block) into top (gauge) and bottom (block)
      let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(top_chunks[0]);

      let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
          [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
          ]
          .as_ref(),
        )
        .split(chunks[2]);

      // Render the widgets to the UI
      f.render_widget(paragraph, left_chunks[0]);
      f.render_widget(gauge, left_chunks[1]);
      f.render_widget(graph, top_chunks[1]);
      f.render_widget(table, chunks[1]);
      if size.height > 42 {
        f.render_widget(filters_list, bottom_chunks[0]);
        f.render_widget(top_queried_domains, bottom_chunks[1]);
        f.render_widget(top_blocked_domains, bottom_chunks[2]);
        f.render_widget(top_clients, bottom_chunks[3]);
      }
    })?;
  }

  // Signal shutdown to the input task and fetcher
  let _ = shutdown_tx.send(true);
  let _ = input_task.await;
  Ok(())
}

/// Enables raw mode + alternate screen, and restores them on drop.
struct TerminalGuard;

impl TerminalGuard {
  fn new() -> Result<Self, anyhow::Error> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Self)
  }
}

impl Drop for TerminalGuard {
  fn drop(&mut self) {
    let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show);
    let _ = disable_raw_mode();
  }
}

/// Returns `true` if a key event should quit the app: `q`, `Q`, or Ctrl+C.
fn is_quit_key(key: KeyEvent) -> bool {
  match key.code {
    KeyCode::Char('q') | KeyCode::Char('Q') => true,
    KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
    _ => false,
  }
}
