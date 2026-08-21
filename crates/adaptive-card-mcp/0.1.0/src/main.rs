//! `adaptive-card-mcp` — MCP server binary wrapping `adaptive-card-core`.

mod errors;
mod prompts;
mod resources;
mod server;
mod tools;

use clap::{Parser, ValueEnum};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about = "Adaptive Cards MCP server")]
struct Cli {
    /// Transport to use for MCP communication.
    #[arg(long, env = "TRANSPORT", value_enum, default_value_t = Transport::Stdio)]
    transport: Transport,

    /// Port to bind when using SSE transport.
    #[arg(long, env = "PORT", default_value_t = 3001)]
    port: u16,

    /// Bind address when using SSE transport.
    #[arg(long, env = "BIND", default_value = "127.0.0.1")]
    bind: String,

    /// Optional bearer API key for SSE transport.
    #[arg(long, env = "MCP_API_KEY")]
    api_key: Option<String>,

    /// Log filter (tracing-subscriber EnvFilter syntax).
    #[arg(long, env = "RUST_LOG", default_value = "adaptive_card_mcp=info")]
    log: String,

    /// Optional external knowledge base directory (for dev iteration).
    #[arg(long, env = "KNOWLEDGE_BASE_DIR")]
    knowledge_base: Option<std::path::PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Transport {
    Stdio,
    Sse,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // For stdio MCP, logs MUST go to stderr so they don't corrupt JSON-RPC on stdout.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&cli.log))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("starting adaptive-card-mcp");

    let kb = load_kb(cli.knowledge_base.as_deref())?;
    let srv = server::Server::new(kb);

    match cli.transport {
        Transport::Stdio => server::run_stdio(srv).await,
        Transport::Sse => server::run_sse(srv, &cli.bind, cli.port, cli.api_key.as_deref()).await,
    }
}

fn load_kb(
    dir: Option<&std::path::Path>,
) -> anyhow::Result<&'static adaptive_card_core::KnowledgeBase> {
    if let Some(path) = dir {
        tracing::info!("loading knowledge base from {}", path.display());
        let kb = adaptive_card_core::KnowledgeBase::from_dir(path)?;
        // Leak to get a 'static reference — server lives for the entire process lifetime.
        Ok(Box::leak(Box::new(kb)))
    } else {
        tracing::info!("using embedded knowledge base");
        Ok(adaptive_card_core::KnowledgeBase::embedded())
    }
}
