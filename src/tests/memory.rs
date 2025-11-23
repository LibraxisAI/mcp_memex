use mcp_memex::{rag::RAGPipeline, storage::StorageManager};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn memory_roundtrip_and_search() {
    let tmp = tempfile::tempdir().unwrap();
    let db_path = tmp.path().join("../../.lancedb");

    let storage = Arc::new(
        StorageManager::new(64, &db_path.to_string_lossy())
            .await
            .expect("storage"),
    );
    storage.ensure_collection().await.expect("collection");

    let mlx = Arc::new(Mutex::new(None));
    let rag = RAGPipeline::new(mlx, storage.clone()).await.expect("rag");

    // Upsert a memory chunk
    rag.memory_upsert(
        "testns",
        "doc1".to_string(),
        "Ala ma kota".to_string(),
        json!({"lang": "pl"}),
    )
    .await
    .expect("upsert");

    // Read it back
    let fetched = rag
        .memory_get("testns", "doc1")
        .await
        .expect("get")
        .expect("doc exists");
    assert_eq!(fetched.text, "Ala ma kota");
    assert_eq!(fetched.namespace, "testns");

    // Semantic search within namespace
    let results = rag
        .memory_search("testns", "kota", 1)
        .await
        .expect("search");
    assert!(!results.is_empty(), "expected at least one search result");
    assert_eq!(results[0].namespace, "testns");
}
