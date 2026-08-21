use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio_util::sync::CancellationToken;
use tracing::info;

use acompose::compositor::Compositor;
use acompose::runner::{ServiceHandles, run_services};

#[derive(Parser, Debug)]
#[command(name = "acompose")]
#[command(about = "Spawn persistent agents via ACP and expose them through an MCP server.")]
struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, default_value = "acompose.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let main_cancel = CancellationToken::new();
    let (compositor, config, state) =
        Compositor::from_config_file(&cli.config, None, None, Some(main_cancel.clone()))
            .await
            .context("loading config")?;
    let compositor = Arc::new(compositor);

    let handles: ServiceHandles = run_services(
        Arc::clone(&compositor),
        &config,
        &state,
        main_cancel.clone(),
    )
    .await?;

    // Wait for shutdown signal.
    tokio::signal::ctrl_c()
        .await
        .context("waiting for shutdown signal")?;
    info!("shutdown signal received, stopping acompose");

    main_cancel.cancel();
    handles.shutdown().await;

    compositor.shutdown().await;
    info!("acompose finished");
    Ok(())
}
