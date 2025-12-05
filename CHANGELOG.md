# Changelog

## [0.1.6] - 2025-12-04
- **Transport fix**: Switched from LSP-style Content-Length framing to newline-delimited JSON (standard MCP transport). Fixes compatibility with Codex and other MCP hosts.
- **Dependency optimization**: Removed unused crates (`octocrab`, `scraper`, `quick-xml`); disabled LanceDB cloud features (`aws`, `azure`, `gcs`, `oss`, `dynamodb`). Reduced unique dependencies from ~1011 to ~618 (~39% reduction).
- Added Loctree integration proposal (`docs/LOCTREE_INTEGRATION_PROPOSAL.md`).
- Silenced vendored protoc build warning.
- Updated documentation and improved Codex config example.

## [0.1.5] - 2025-12-03
- Renamed crate/binary from `mcp_memex` to `rmcp_memex`.
- Added GitHub Actions CI: fmt, clippy, semgrep, tests, tarpaulin coverage (with protoc install).
- Introduced config loader (`--config <toml>`) with flag overrides; added `max_request_bytes` limit (default 5 MB) and improved log-level parsing.
- Added `health` tool (version, db_path, cache_dir, backend) and safer JSON-RPC framing.
- Improved temp LanceDB isolation in tests; clarified embed example and env handling.
- Added pre-push hook with full quality gate (fmt, clippy, test, semgrep).

## [0.1.1] - 2025-11-25
- Defaulted fastembed/HF cache to `$HOME/.cache/fastembed` to avoid `.fastembed_cache` in CWDs.
- Refined hooks/clippy and installer/build scripts.
- Fixed build script path (`build = "src/build.rs"`).

## [0.1.0] - 2024-11-20
- Switched vector storage from ChromaDB to embedded LanceDB.
- Added namespaces and memory tools to the RAG server.
- Initial MCP Rust server structure, README, and build scripts.
