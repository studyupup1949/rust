pub fn main() {
    #[cfg(feature = "tracing")]
    tracing::info!("This is from the TUI based on Ratatui.");
}
