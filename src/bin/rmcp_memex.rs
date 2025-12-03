use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use rmcp_memex::{handlers, ServerConfig};

fn parse_features(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn load_file_config(path: &str) -> Result<FileConfig> {
    let expanded = shellexpand::tilde(path).to_string();
    let contents = std::fs::read_to_string(&expanded)?;
    toml::from_str(&contents).map_err(Into::into)
}

#[derive(serde::Deserialize, Default)]
struct FileConfig {
    features: Option<String>,
    cache_mb: Option<usize>,
    db_path: Option<String>,
    max_request_bytes: Option<usize>,
    log_level: Option<String>,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional config file (TOML) to load settings from; CLI flags override file when set.
    #[arg(long)]
    config: Option<String>,

    /// Enable specific features (comma-separated)
    #[arg(long)]
    features: Option<String>,

    /// Cache size in MB
    #[arg(long)]
    cache_mb: Option<usize>,

    /// Path for embedded vector store (LanceDB)
    #[arg(long)]
    db_path: Option<String>,

    /// Max allowed request size in bytes for JSON-RPC framing
    #[arg(long)]
    max_request_bytes: Option<usize>,

    /// Log level
    #[arg(long)]
    log_level: Option<String>,
}

impl Args {
    fn into_config(self) -> Result<ServerConfig> {
        let file_cfg = self
            .config
            .as_deref()
            .map(load_file_config)
            .transpose()?
            .unwrap_or_default();
        let default_cfg = ServerConfig::default();

        Ok(ServerConfig {
            features: self
                .features
                .or(file_cfg.features)
                .map(|s| parse_features(&s))
                .unwrap_or(default_cfg.features),
            cache_mb: self
                .cache_mb
                .or(file_cfg.cache_mb)
                .unwrap_or(default_cfg.cache_mb),
            db_path: self
                .db_path
                .or(file_cfg.db_path)
                .unwrap_or(default_cfg.db_path),
            max_request_bytes: self
                .max_request_bytes
                .or(file_cfg.max_request_bytes)
                .unwrap_or(default_cfg.max_request_bytes),
            log_level: self
                .log_level
                .or(file_cfg.log_level)
                .map(|s| parse_log_level(&s))
                .unwrap_or(default_cfg.log_level),
        })
    }
}

fn parse_log_level(level: &str) -> Level {
    match level.to_ascii_lowercase().as_str() {
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
    let config = args.into_config()?;

    // Send logs to stderr to keep stdout clean for JSON-RPC.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(config.log_level)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting RMCP Memex");
    info!("Features (informational): {:?}", config.features);
    info!("Cache: {}MB", config.cache_mb);
    info!("DB Path: {}", config.db_path);

    let server = handlers::create_server(config).await?;
    server.run_stdio().await?;

    Ok(())
}
