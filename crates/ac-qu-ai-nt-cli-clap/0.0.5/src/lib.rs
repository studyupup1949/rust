pub fn main() {
    #[cfg(feature = "tracing")]
    tracing::info!("What's up, world?");
}
