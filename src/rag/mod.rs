use anyhow::Result;
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{embeddings::MLXBridge, storage::{StorageManager, ChromaDocument}};

pub struct RAGPipeline {
    mlx_bridge: Arc<Mutex<MLXBridge>>,
    storage: Arc<StorageManager>,
}

impl RAGPipeline {
    pub async fn new(
        mlx_bridge: Arc<Mutex<MLXBridge>>,
        storage: Arc<StorageManager>,
    ) -> Result<Self> {
        Ok(Self {
            mlx_bridge,
            storage,
        })
    }

    pub async fn index_document(&self, path: &Path) -> Result<()> {
        // TODO: Extract text from document
        let text = self.extract_text(path).await?;
        
        // Chunk the text
        let chunks = self.chunk_text(&text, 512, 128)?;
        
        // Embed chunks
        let mut embeddings = Vec::new();
        let mut mlx = self.mlx_bridge.lock().await;
        
        for chunk in &chunks {
            let embedding = mlx.embed(chunk).await?;
            embeddings.push(embedding);
        }
        
        // Store in ChromaDB
        let mut documents = Vec::new();
        for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            let doc = ChromaDocument {
                id: format!("{}_{}", path.to_str().unwrap_or("unknown"), i),
                embedding: embedding.clone(),
                metadata: json!({
                    "path": path.to_str(),
                    "chunk_index": i,
                    "total_chunks": chunks.len(),
                }),
                document: chunk.clone(),
            };
            documents.push(doc);
        }
        
        self.storage.add_to_chroma(documents).await?;
        
        Ok(())
    }

    pub async fn search(&self, query: &str, k: usize) -> Result<Vec<SearchResult>> {
        // Embed query
        let mut mlx = self.mlx_bridge.lock().await;
        let query_embedding = mlx.embed(query).await?;
        
        // Search in ChromaDB
        let candidates = self.storage.search_chroma(query_embedding, k * 3).await?;
        
        // Rerank if we have candidates
        if !candidates.is_empty() {
            let documents: Vec<String> = candidates
                .iter()
                .map(|c| c.document.clone())
                .collect();
            
            let reranked = mlx.rerank(query, &documents).await?;
            
            // Return top-k reranked results
            let results: Vec<SearchResult> = reranked
                .into_iter()
                .take(k)
                .map(|(idx, score)| SearchResult {
                    text: documents[idx].clone(),
                    score,
                    metadata: json!({}),
                })
                .collect();
            
            return Ok(results);
        }
        
        Ok(vec![])
    }

    async fn extract_text(&self, path: &Path) -> Result<String> {
        // TODO: Implement document extraction
        // For now, just read as text
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| e.into())
    }

    fn chunk_text(&self, text: &str, chunk_size: usize, overlap: usize) -> Result<Vec<String>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        
        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);
            
            if end >= chars.len() {
                break;
            }
            
            start = end - overlap;
        }
        
        Ok(chunks)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub text: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}