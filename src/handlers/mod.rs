use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{embeddings::MLXBridge, rag::RAGPipeline, storage::StorageManager, Args};

pub struct MCPServer {
    mlx_bridge: Arc<Mutex<Option<MLXBridge>>>,
    rag: Arc<RAGPipeline>,
    storage: Arc<StorageManager>,
}

impl MCPServer {
    pub async fn run(self) -> Result<()> {
        // For now, just a simple stdin/stdout loop
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut stdout = tokio::io::stdout();

        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
                let response = self.handle_request(request).await;
                let response_str = serde_json::to_string(&response)? + "\n";
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.flush().await?;
            }
            line.clear();
        }

        Ok(())
    }

    async fn handle_request(&self, request: serde_json::Value) -> serde_json::Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        let result = match method {
            "initialize" => json!({
                "protocolVersion": "1.0",
                "capabilities": {
                    "tools": true,
                    "resources": true,
                }
            }),

            "tools/list" => json!({
                "tools": [
                    {
                        "name": "rag_index",
                        "description": "Index a document for RAG",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {"type": "string"},
                                "namespace": {"type": "string"}
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "rag_index_text",
                        "description": "Index raw text for RAG/memory",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": {"type": "string"},
                                "id": {"type": "string"},
                                "namespace": {"type": "string"},
                                "metadata": {"type": "object"}
                            },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "rag_search",
                        "description": "Search documents using RAG",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string"},
                                "k": {"type": "integer", "default": 10},
                                "namespace": {"type": "string"}
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "memory_upsert",
                        "description": "Upsert a text chunk into vector memory",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "namespace": {"type": "string"},
                                "id": {"type": "string"},
                                "text": {"type": "string"},
                                "metadata": {"type": "object"}
                            },
                            "required": ["namespace", "id", "text"]
                        }
                    },
                    {
                        "name": "memory_get",
                        "description": "Get a stored chunk by namespace + id",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "namespace": {"type": "string"},
                                "id": {"type": "string"}
                            },
                            "required": ["namespace", "id"]
                        }
                    },
                    {
                        "name": "memory_search",
                        "description": "Semantic search within a namespace",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "namespace": {"type": "string"},
                                "query": {"type": "string"},
                                "k": {"type": "integer", "default": 5}
                            },
                            "required": ["namespace", "query"]
                        }
                    },
                    {
                        "name": "memory_delete",
                        "description": "Delete a chunk by namespace + id",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "namespace": {"type": "string"},
                                "id": {"type": "string"}
                            },
                            "required": ["namespace", "id"]
                        }
                    },
                    {
                        "name": "memory_purge_namespace",
                        "description": "Delete all chunks in a namespace",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "namespace": {"type": "string"}
                            },
                            "required": ["namespace"]
                        }
                    }
                ]
            }),

            "tools/call" => {
                let tool_name = request["params"]["name"].as_str().unwrap_or("");
                let args = &request["params"]["arguments"];

                match tool_name {
                    "rag_index" => {
                        let path = args["path"].as_str().unwrap_or("");
                        let namespace = args["namespace"].as_str();
                        match self
                            .rag
                            .index_document(std::path::Path::new(path), namespace)
                            .await
                        {
                            Ok(_) => json!({
                                "content": [{"type": "text", "text": format!("Indexed: {}", path)}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "rag_index_text" => {
                        let text = args["text"].as_str().unwrap_or("").to_string();
                        let namespace = args["namespace"].as_str();
                        let metadata = args.get("metadata").cloned().unwrap_or_else(|| json!({}));
                        let id = args
                            .get("id")
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                            .unwrap_or_else(|| Uuid::new_v4().to_string());

                        match self
                            .rag
                            .index_text(namespace, id.clone(), text, metadata)
                            .await
                        {
                            Ok(returned_id) => json!({
                                "content": [{"type": "text", "text": format!("Indexed text with id {}", returned_id)}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "rag_search" => {
                        let query = args["query"].as_str().unwrap_or("");
                        let k = args["k"].as_u64().unwrap_or(10) as usize;
                        let namespace = args["namespace"].as_str();

                        match self.rag.search_inner(namespace, query, k).await {
                            Ok(results) => json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&results).unwrap_or_default()
                                }]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "memory_upsert" => {
                        let namespace = args["namespace"].as_str().unwrap_or("default");
                        let id = args["id"].as_str().unwrap_or("").to_string();
                        let text = args["text"].as_str().unwrap_or("").to_string();
                        let metadata = args.get("metadata").cloned().unwrap_or_else(|| json!({}));

                        match self
                            .rag
                            .memory_upsert(namespace, id.clone(), text, metadata)
                            .await
                        {
                            Ok(_) => json!({
                                "content": [{"type": "text", "text": format!("Upserted {}", id)}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "memory_get" => {
                        let namespace = args["namespace"].as_str().unwrap_or("default");
                        let id = args["id"].as_str().unwrap_or("");
                        match self.rag.memory_get(namespace, id).await {
                            Ok(Some(doc)) => json!({
                                "content": [{"type": "text", "text": serde_json::to_string(&doc).unwrap_or_default()}]
                            }),
                            Ok(None) => json!({
                                "content": [{"type": "text", "text": "Not found"}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "memory_search" => {
                        let namespace = args["namespace"].as_str().unwrap_or("default");
                        let query = args["query"].as_str().unwrap_or("");
                        let k = args["k"].as_u64().unwrap_or(5) as usize;
                        match self.rag.memory_search(namespace, query, k).await {
                            Ok(results) => json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string(&results).unwrap_or_default()
                                }]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "memory_delete" => {
                        let namespace = args["namespace"].as_str().unwrap_or("default");
                        let id = args["id"].as_str().unwrap_or("");
                        match self.rag.memory_delete(namespace, id).await {
                            Ok(deleted) => json!({
                                "content": [{"type": "text", "text": format!("Deleted {} rows", deleted)}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    "memory_purge_namespace" => {
                        let namespace = args["namespace"].as_str().unwrap_or("default");
                        match self.rag.purge_namespace(namespace).await {
                            Ok(deleted) => json!({
                                "content": [{"type": "text", "text": format!("Purged namespace '{}', removed {} rows", namespace, deleted)}]
                            }),
                            Err(e) => json!({
                                "error": {"message": e.to_string()}
                            }),
                        }
                    }
                    _ => json!({"error": {"message": "Unknown tool"}}),
                }
            }

            _ => json!({"error": {"message": "Unknown method"}}),
        };

        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    }
}

pub async fn create_server(args: Args) -> Result<MCPServer> {
    // Initialize components
    let mlx_bridge = match MLXBridge::new().await {
        Ok(mlx) => Some(mlx),
        Err(e) => {
            tracing::warn!(
                "MLX bridge unavailable, falling back to fastembed only: {}",
                e
            );
            None
        }
    };
    let mlx_bridge = Arc::new(Mutex::new(mlx_bridge));
    let storage = Arc::new(StorageManager::new(args.cache_mb, &args.db_path).await?);
    storage.ensure_collection().await?;
    let rag = Arc::new(RAGPipeline::new(mlx_bridge.clone(), storage.clone()).await?);

    Ok(MCPServer {
        mlx_bridge,
        rag,
        storage,
    })
}
