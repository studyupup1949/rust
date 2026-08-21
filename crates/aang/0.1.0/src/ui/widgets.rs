//! Custom widgets and helpers

use ratatui::style::{Color, Style};
use ratatui::text::Span;

/// Create a styled label-value pair
pub fn label_value<'a>(label: &'a str, value: &'a str, value_color: Color) -> Vec<Span<'a>> {
    vec![
        Span::styled(label, Style::default().fg(Color::Gray)),
        Span::styled(value, Style::default().fg(value_color)),
    ]
}

/// Format SOL amount with appropriate precision
pub fn format_sol(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    if sol >= 1.0 {
        format!("{:.4} SOL", sol)
    } else if sol >= 0.001 {
        format!("{:.6} SOL", sol)
    } else {
        format!("{:.9} SOL", sol)
    }
}

/// Format lamports with thousand separators
pub fn format_lamports(lamports: u64) -> String {
    let s = lamports.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Truncate a pubkey for display
pub fn truncate_pubkey(pubkey: &str, prefix_len: usize, suffix_len: usize) -> String {
    if pubkey.len() <= prefix_len + suffix_len + 3 {
        pubkey.to_string()
    } else {
        format!(
            "{}...{}",
            &pubkey[..prefix_len],
            &pubkey[pubkey.len() - suffix_len..]
        )
    }
}
