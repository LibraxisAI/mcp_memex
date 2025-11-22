 mcp_memex

 Lightweight Model Context Protocol (MCP) server written in Rust. It provides a local Retrieval-Augmented Generation (RAG) toolset backed by an embedded LanceDB vector store and local embeddings. If an MLX HTTP server is available, it is used for embeddings and reranking; otherwise the server falls back to on‑device embeddings via fastembed.

 Tools exposed to MCP clients
 - rag_index(path) — index a file (UTF‑8 text or PDF) into the local vector store
 - rag_search(query, k=10) — search indexed chunks and return the top‑k results

 Overview
 - Stack: Rust 2021, Tokio, Clap
 - Vector store: Embedded LanceDB (no external DB needed)
 - Embeddings: Optional MLX HTTP bridge; automatic fastembed fallback
 - Caching/persistence: moka (in‑memory) + sled (local key/value)
 - IO: reqwest for HTTP; pdf-extract for PDF text
 - Transport: JSON‑RPC over stdin/stdout (compatible with MCP hosts)

 Entry point: src/main.rs (binary name: mcp_memex). The server logs to stdout/stderr and reads JSON‑RPC requests from stdin.

 Requirements
 - Rust toolchain with Cargo (stable)
 - macOS or Linux (Windows likely works but untested)
 - Protobuf compiler: required by some dependencies at build time. If you don’t have it, install it (e.g., macOS: `brew install protobuf`; Linux: `apt install protobuf-compiler`).
 - Optional MLX bridge: HTTP server with /v1/embeddings, /v1/rerank, /v1/models.

 Quick start
 ```bash
 # build
 cargo build --release

 # run (uses local fastembed by default; LanceDB at ~/.mcp-servers/mcp_memex/lancedb)
 cargo run --release -- --log-level info
 ```

 Configuration
 CLI flags (from src/lib.rs)
 - --features string (default "filesystem,memory,search")
 - --cache-mb usize (default 4096)
 - --db-path string (default "~/.mcp-servers/mcp_memex/lancedb")
 - --log-level trace|debug|info|warn|error (default info)

 Environment variables
 - DISABLE_MLX — if set, disables MLX bridge; fastembed only
 - DRAGON_BASE_URL — base URL for MLX HTTP (default http://localhost)
 - MLX_JIT_MODE — "true" to use a single port for all models (default false)
 - MLX_JIT_PORT — JIT mode port (default 1234)
 - EMBEDDER_PORT — non‑JIT embeddings port (default 12345)
 - RERANKER_PORT — non‑JIT rerank port (default 12346)
 - EMBEDDER_MODEL — embeddings model id (default Qwen/Qwen3-Embedding-4B)
 - RERANKER_MODEL — reranker model id (default Qwen/Qwen3-Reranker-4B)
 - LANCEDB_PATH — overrides the --db-path for the embedded DB (default ~/.mcp-servers/mcp_memex/lancedb)

 Example (MLX non‑JIT)
 ```bash
 export DRAGON_BASE_URL=http://localhost
 export EMBEDDER_PORT=5555
 export RERANKER_PORT=5556
 ```

 Tools (RPC)
 - rag_index(path: string)
   - Extracts text (PDF via pdf-extract; others as UTF‑8)
   - Chunks to size 512 with overlap 128; embeds (MLX or fastembed)
   - Inserts into LanceDB table mcp_documents (auto‑created)

 - rag_search(query: string, k: number=10)
   - Embeds the query, searches LanceDB, reranks with MLX if available (cosine fallback)
   - Returns text, score, metadata

 Scripts
 - build-macos.sh — builds release and creates a minimal app bundle at ~/.mcp-servers/MCPServer.app with CFBundleExecutable=mcp_memex
 - install.sh — builds the release binary; pass --bundle-macos to also create the app bundle

 Project structure
 - src/main.rs • src/lib.rs • src/handlers • src/embeddings • src/rag • src/storage
 - build.rs • build-macos.sh • install.sh • Cargo.toml

 Tests
 ```bash
 cargo test
 ```

 License
 MIT — see LICENSE.

 Known limitations
 - Only text and PDF ingestion are supported (no HTML/Markdown parsing yet)
 - LanceDB collection name is fixed to mcp_documents
 - Minimal JSON‑RPC loop intended for MCP hosts; richer transport may be added