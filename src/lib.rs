pub mod embeddings;
pub mod handlers;
pub mod rag;
pub mod storage;

use anyhow::Result;
use tracing::Level;

pub use handlers::{create_server, MCPServer};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Enabled features (namespaced strings)
    pub features: Vec<String>,

    /// Cache size in MB for sled/moka
    pub cache_mb: usize,

    /// Path for embedded vector store (LanceDB)
    pub db_path: String,

    /// Max allowed request size (bytes) for JSON-RPC framing
    pub max_request_bytes: usize,

    /// Default log level to use when wiring tracing
    pub log_level: Level,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            features: vec![
                "filesystem".to_string(),
                "memory".to_string(),
                "search".to_string(),
            ],
            cache_mb: 4096,
            db_path: "~/.rmcp_servers/rmcp_memex/lancedb".to_string(),
            max_request_bytes: 5 * 1024 * 1024,
            log_level: Level::INFO,
        }
    }
}

impl ServerConfig {
    pub fn with_db_path(mut self, db_path: impl Into<String>) -> Self {
        self.db_path = db_path.into();
        self
    }
}

/// Helper to build and run the stdin/stdout server for library consumers.
pub async fn run_stdio_server(config: ServerConfig) -> Result<()> {
    let server = create_server(config).await?;
    server.run_stdio().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = ServerConfig::default();
        assert!(cfg.features.contains(&"filesystem".to_string()));
        assert_eq!(cfg.cache_mb, 4096);
        assert_eq!(cfg.db_path, "~/.rmcp_servers/rmcp_memex/lancedb");
        assert_eq!(cfg.max_request_bytes, 5 * 1024 * 1024);
    }
}
