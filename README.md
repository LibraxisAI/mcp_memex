# mcp_memex

Lightweight MCP server with embedded LanceDB vector store + local embeddings (MLX when available, otherwise `fastembed`).  
Tools: `rag_index(path)` and `rag_search(query, k=10)`.

## Quick start

Embedded DB lives at `~/.mcp-servers/mcp_memex/lancedb` by default. No external services needed.

```bash
# Optional: point to a custom location
export LANCEDB_PATH=/tmp/memex-db

# Optional MLX endpoints (if running your own service)
export EMBEDDER_PORT=8986
export RERANKER_PORT=8985
# or JIT: export MLX_JIT_MODE=true && export MLX_JIT_PORT=1234

cargo run --release -- --cache-mb 512 --log-level debug
```

CLI flags (`Args`):

- `--features` (default `filesystem,memory,search`) – informational.
- `--cache-mb` – cache size (moka + sled), default 4096.
- `--db-path` – embedded LanceDB path (defaults to `~/.mcp-servers/mcp_memex/lancedb`).
- `--log-level` – `trace|debug|info|warn|error` (default `info`).
- If build complains about `protoc`, point `PROTOC` to the vendored binary (from `protoc-bin-vendored`) or install `protobuf` (`brew install protobuf` on macOS).

## How it works

- `rag_index(path)`:
  1) extract text (PDF via `pdf_extract`, else UTF-8),
  2) chunk 512/128 characters,
  3) embed each chunk (MLX if available, fallback to `fastembed`),
  4) store in LanceDB table `mcp_documents` (auto-created).

- `rag_search(query, k)`:
  1) embed query,
  2) LanceDB nearest-neighbor search,
  3) rerank via MLX `rerank` or cosine fallback,
  4) return top‑k with metadata and distance.

## Known limitations / TODO

- No HTML/Markdown parsing – text/PDF only.
- Collection name fixed to `mcp_documents`.
- No separate “memory” API – sled is just a cache; K/V tools could be added.
