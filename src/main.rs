use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use mcp_memex::{Args, handlers};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting MCP Memex");
    info!("Features: {}", args.features);
    info!("Cache: {}MB", args.cache_mb);

    // TODO: Initialize MCP server
    let server = handlers::create_server(args).await?;
    
    // Run server
    server.run().await?;

    Ok(())
}
