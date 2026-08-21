//! ACE Player CLI — play a media file from the command line.
use anyhow::Result;
use ace_player::{AcePlayer, PlayerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let source = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ace-player <file-or-url>");
        std::process::exit(1);
    });

    let config = PlayerConfig {
        source: source.clone(),
        ..Default::default()
    };

    let mut player = AcePlayer::new(config);
    let info = player.load().await?;

    println!("ACE Player v{}", env!("CARGO_PKG_VERSION"));
    println!("Source:   {source}");
    println!("Duration: {:.1}s", info.duration_secs);
    println!("Video:    {}x{} @ {:.0}fps ({})",
        info.width, info.height, info.fps,
        info.codec_video.as_deref().unwrap_or("unknown"));
    println!("Audio:    {}",
        info.codec_audio.as_deref().unwrap_or("none"));

    player.play().await?;

    // Keep alive while playing
    tokio::signal::ctrl_c().await?;
    println!("\nStopped.");
    Ok(())
}
