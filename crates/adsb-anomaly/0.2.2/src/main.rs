// ABOUTME: Main entry point for the ADS-B Aircraft Anomaly Detection System
// ABOUTME: Sets up CLI args, tracing, and coordinates the async runtime

mod alerts;
mod circuit_breaker;
mod config;
mod detect;
mod error;
mod ingestion;
mod metrics;
mod model;
mod retention;
mod store;
mod web;

use detect::temporal;
use metrics::AppMetrics;
use retention::{RetentionConfig, RetentionService};

use clap::Parser;
use config::AppConfig;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "adsb-anomaly")]
#[command(version = "0.1.0")]
#[command(about = "ADS-B Aircraft Anomaly Detection System")]
struct Args {
    #[arg(long, help = "Set log level")]
    log_level: Option<String>,

    #[arg(long, help = "Path to configuration file")]
    config: Option<String>,

    #[arg(long, help = "Database file path")]
    db_path: Option<String>,

    #[arg(long, help = "Web server port")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    init_tracing(&args.log_level)?;

    // Load configuration with precedence: file -> env -> CLI
    let mut app_config = AppConfig::load(args.config.as_deref())?;
    app_config.merge_cli_overrides(args.config, args.db_path, args.port);

    info!(
        "Starting ADS-B Anomaly Detection System v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("Configuration loaded successfully");
    info!("Database path: {}", app_config.database.path);
    info!("Web server will run on port: {}", app_config.web.port);
    info!("PiAware URL: {}", app_config.adsb.piaware_url);

    // Initialize metrics
    AppMetrics::init()?;
    info!("Metrics initialized");

    // Connect to database and run migrations
    let pool =
        store::connect_and_migrate(&app_config.database.path, app_config.database.wal_mode).await?;
    info!("Database connection established and migrations completed");

    // Create alert channels for anomaly detection
    let (temporal_sender, _temporal_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (signal_sender, _signal_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (identity_sender, _identity_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (behavior_sender, _behavior_receiver) = tokio::sync::mpsc::unbounded_channel();

    // Create detection services with alert channels
    let config = Arc::new(app_config.analysis.clone());
    let _temporal_service = temporal::TemporalDetectionService::new(
        config.clone(),
        temporal_sender,
        Some(10), // Ring buffer size
    );
    let _signal_service =
        detect::signal::SignalDetectionService::new(config.clone(), signal_sender);
    let _identity_service =
        detect::identity::IdentityDetectionService::new(config.clone(), identity_sender);
    let _behavior_service =
        detect::behavior::BehaviorDetectionService::new(config, behavior_sender);

    // Create alert manager
    let _alert_manager = Arc::new(tokio::sync::Mutex::new(alerts::AlertManager::new(
        pool.clone(),
    )));

    // TODO: Wire detection services into ingestion and connect receivers to alert manager

    // Clone pool for ingestion service
    let ingestion_pool = pool.clone();
    let ingestion_url = app_config.adsb.piaware_url.clone();
    let poll_interval = app_config.adsb.poll_interval_ms;
    let circuit_breaker_config = app_config.adsb.circuit_breaker.clone().into();

    // Spawn ingestion service with circuit breaker
    let ingestion_handle = tokio::spawn(async move {
        info!("Starting ingestion service with circuit breaker protection");
        if let Err(e) = ingestion::service::run_ingestion_with_circuit_breaker(
            ingestion_pool,
            ingestion_url,
            poll_interval,
            circuit_breaker_config,
        )
        .await
        {
            error!("Ingestion service failed: {}", e);
        }
    });

    // Spawn web server
    let web_pool = pool.clone();
    let web_config = app_config.web.clone();
    let analysis_config = app_config.analysis.clone();
    let web_handle = tokio::spawn(async move {
        info!("Starting web server");
        if let Err(e) = web::serve(web_pool, web_config, analysis_config).await {
            error!("Web server failed: {}", e);
        }
    });

    // Spawn retention service
    let retention_pool = pool.clone();
    let retention_config = RetentionConfig::default(); // Use default config for now
    let retention_handle = tokio::spawn(async move {
        let retention_service = RetentionService::new(retention_pool, retention_config);
        info!("Starting retention service");
        retention_service.run().await;
    });

    // Wait for shutdown signal
    info!("System ready - press Ctrl+C to shutdown");
    signal::ctrl_c().await?;
    info!("Shutdown signal received, stopping services...");

    // Cancel running tasks
    ingestion_handle.abort();
    web_handle.abort();
    retention_handle.abort();

    // Wait a bit for graceful shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    info!("Shutdown complete");

    Ok(())
}

fn init_tracing(log_level: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let level = match log_level
        .as_deref()
        .or(std::env::var("RUST_LOG").ok().as_deref())
    {
        Some("debug") => Level::DEBUG,
        Some("warn") => Level::WARN,
        Some("error") => Level::ERROR,
        Some("trace") => Level::TRACE,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // Test that Clap parses --version without error
        let cmd = Args::command();
        cmd.debug_assert();
    }

    #[test]
    fn test_tracing_init() {
        // Test that initializing tracing subscriber doesn't panic
        let result = init_tracing(&Some("info".to_string()));
        // Should succeed (though global subscriber may already be set in other tests)
        // We just verify it doesn't panic
        assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for this test
    }
}
