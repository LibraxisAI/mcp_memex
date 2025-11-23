use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use mcp_memex::{handlers, ServerConfig};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable specific features (comma-separated)
    #[arg(long, default_value = "filesystem,memory,search")]
    features: String,

    /// Cache size in MB
    #[arg(long, default_value = "4096")]
    cache_mb: usize,

    /// Path for embedded vector store (LanceDB)
    #[arg(long, default_value = "~/.mcp-servers/mcp_memex/lancedb")]
    db_path: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

impl Args {
    fn into_config(self) -> ServerConfig {
        ServerConfig {
            features: self
                .features
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            cache_mb: self.cache_mb,
            db_path: shellexpand::tilde(&self.db_path).to_string(),
            log_level: parse_log_level(&self.log_level),
        }
    }
}

fn parse_log_level(level: &str) -> Level {
    match level {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = args.clone().into_config();

    // Send logs to stderr to keep stdout clean for JSON-RPC.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config.log_level)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting MCP Memex");
    info!("Features: {}", args.features);
    info!("Cache: {}MB", args.cache_mb);
    info!("DB Path: {}", config.db_path);

    let server = handlers::create_server(config).await?;
    server.run_stdio().await?;

    Ok(())
}
