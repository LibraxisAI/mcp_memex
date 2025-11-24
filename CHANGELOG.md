# Changelog

## [0.1.5] - 2025-XX-XX
- Renamed crate/binary to `rmcp_memex`.
- Added GitHub Actions CI: fmt, clippy, semgrep, tests, tarpaulin coverage (with protoc install).
- Introduced config loader (`--config <toml>`) with flag overrides; added `max_request_bytes` limit (default 5 MB) and improved log-level parsing.
- Added `health` tool (version, db_path, cache_dir, backend) and safer JSON-RPC framing (bounded Content-Length, graceful recovery on bad frames).
- Improved temp LanceDB isolation in tests; clarified embed example and env handling.
- Bumped crate version to 0.1.5.

## [0.1.1] - 2025-XX-XX
- Defaulted fastembed/HF cache to `$HOME/.cache/fastembed` to avoid `.fastembed_cache` in CWDs.
- Refined hooks/clippy and installer/build scripts.
- Fixed build script path (`build = "src/build.rs"`).

## [0.1.0] - 2024-XX-XX
- Switched vector storage from ChromaDB to embedded LanceDB.
- Added namespaces and memory tools to the RAG server.
- Initial MCP Rust server structure, README, and build scripts.
