use tui::{
  style::{Color, Modifier, Style},
  text::Span,
  widgets::{Block, Borders, Gauge},
};

use crate::fetch::fetch_stats::StatsResponse;

pub fn make_gauge(stats: &StatsResponse) -> Gauge<'_> {
  let total_blocked = stats.num_blocked_filtering
    + stats.num_replaced_parental
    + stats.num_replaced_safebrowsing
    + stats.num_replaced_safesearch;

  // `max(1)` avoids a divide-by-zero, and the clamp keeps it in the 0..=100
  // range that `Gauge::percent` requires (it panics otherwise)
  let percent =
    (total_blocked as f64 / stats.num_dns_queries.max(1) as f64 * 100.0).min(100.0) as u16;

  let label = format!(
    "Blocked {} out of {} ({}%)",
    total_blocked, stats.num_dns_queries, percent
  );

  Gauge::default()
    .block(
      Block::default()
        .title(Span::styled(
          "Block Percentage",
          Style::default().add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL),
    )
    .gauge_style(Style::default().fg(Color::Red).bg(Color::Green))
    .percent(percent)
    .label(label)
}
