use anyhow::{anyhow, Result};
use moka::future::Cache;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use sled::Db;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct ChromaDocument {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
    pub document: String,
}

pub struct StorageManager {
    cache: Arc<Cache<String, Vec<u8>>>,
    db: Db,
    chroma_client: Client,
    chroma_url: String,
    collection_name: String,
}

impl StorageManager {
    pub fn new(cache_mb: usize, _chroma_path: &str) -> Result<Self> {
        // Initialize cache with size limit
        let cache_bytes = cache_mb * 1024 * 1024;
        let cache = Cache::builder()
            .max_capacity(cache_bytes as u64)
            .time_to_live(Duration::from_secs(3600))
            .build();

        // Initialize Sled DB
        let db_path = shellexpand::tilde("~/.mcp-servers/sled").to_string();
        let db = sled::open(db_path)?;

        // Initialize ChromaDB client
        let chroma_url = std::env::var("CHROMA_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        let chroma_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            cache: Arc::new(cache),
            db,
            chroma_client,
            chroma_url,
            collection_name: "mcp_documents".to_string(),
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Check cache first
        if let Some(value) = self.cache.get(key).await {
            return Ok(Some(value));
        }

        // Check persistent storage
        if let Some(value) = self.db.get(key)? {
            let vec = value.to_vec();
            // Add to cache
            self.cache.insert(key.to_string(), vec.clone()).await;
            return Ok(Some(vec));
        }

        Ok(None)
    }

    pub async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        // Store in cache
        self.cache.insert(key.to_string(), value.clone()).await;
        
        // Store persistently
        self.db.insert(key, value)?;
        self.db.flush()?;
        
        Ok(())
    }

    pub async fn add_to_chroma(&self, documents: Vec<ChromaDocument>) -> Result<()> {
        let url = format!("{}/api/v1/collections/{}/add", self.chroma_url, self.collection_name);
        
        let ids: Vec<String> = documents.iter().map(|d| d.id.clone()).collect();
        let embeddings: Vec<Vec<f32>> = documents.iter().map(|d| d.embedding.clone()).collect();
        let metadatas: Vec<serde_json::Value> = documents.iter().map(|d| d.metadata.clone()).collect();
        let documents_text: Vec<String> = documents.iter().map(|d| d.document.clone()).collect();
        
        let payload = json!({
            "ids": ids,
            "embeddings": embeddings,
            "metadatas": metadatas,
            "documents": documents_text,
        });
        
        let _response = self.chroma_client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
            
        if !_response.status().is_success() {
            return Err(anyhow!("Failed to add to ChromaDB: {}", _response.status()));
        }
        
        Ok(())
    }
    
    pub async fn search_chroma(&self, embedding: Vec<f32>, k: usize) -> Result<Vec<ChromaDocument>> {
        let url = format!("{}/api/v1/collections/{}/query", self.chroma_url, self.collection_name);
        
        let payload = json!({
            "query_embeddings": [embedding],
            "n_results": k,
        });
        
        let _response = self.chroma_client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
            
        // Parse response
        // TODO: Implement proper response parsing
        Ok(vec![])
    }
    
    pub fn get_collection_name(&self) -> &str {
        &self.collection_name
    }
}