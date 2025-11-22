pub mod embeddings;
pub mod handlers;
pub mod rag;
pub mod storage;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Enable specific features (comma-separated)
    #[arg(long, default_value = "filesystem,memory,search")]
    pub features: String,

    /// Cache size in MB
    #[arg(long, default_value = "4096")]
    pub cache_mb: usize,

    /// Path for embedded vector store (LanceDB)
    #[arg(long, default_value = "~/.mcp-servers/mcp_memex/lancedb")]
    pub db_path: String,

    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(2 + 2, 4);
    }
}
