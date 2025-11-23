# Repository Guidelines

## Project Structure & Module Organization
- `src/bin/rmcp_memex.rs` — CLI entrypoint for the MCP server; keeps stdout clean for JSON-RPC framing.
- `src/lib.rs` — public API with `ServerConfig`, `run_stdio_server`, and helpers for embedding this crate.
- `src/handlers` — JSON-RPC framing, request dispatch, transport plumbing.
- `src/embeddings` & `src/rag` — embedding backends (fastembed/MLX) and RAG pipeline.
- `src/storage` — LanceDB integration and namespace utilities; tests live in `src/tests`.
- `scripts/` — `build-macos.sh`, `install.sh`; `tools/` — git hooks (`tools/githooks/pre-commit`), setup scripts, and `.semgrep.yaml`.

## Build, Test, and Development Commands
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets --all-features -D warnings`
- Test: `cargo test --all-targets` (keep DB writes inside tempdirs)
- Full pre-commit suite: `tools/setup_hooks.sh` to install, then `git commit` runs fmt + clippy + tests + semgrep.
- Build/release: `cargo build --release` or `scripts/build-macos.sh`; install locally via `scripts/install.sh`.
- Run server: `FASTEMBED_CACHE_PATH=$HOME/.cache/fastembed HF_HUB_CACHE=$FASTEMBED_CACHE_PATH cargo run --bin rmcp_memex -- --db-path ~/.rmcp_servers/rmcp_memex/.lancedb --log-level info`

## Coding Style & Naming Conventions
- Rust default style enforced by rustfmt; modules/files use `snake_case`.
- Avoid `unwrap`/`expect` in production code and tests; prefer `Result<()>` + `?` to satisfy `.semgrep.yaml` and hooks.
- Keep stdout reserved for protocol replies; log to stderr (`tracing`) with lower-case levels (`info`, `warn`, `debug`).
- Do not reintroduce `allow(dead_code)`; delete unused code or gate it behind features.

## Testing Guidelines
- Use temporary directories and keep LanceDB artifacts inside the tempdir (e.g., `tmp.path().join(".lancedb")`).
- Mark async tests with `#[tokio::test]`; return `Result<()>` to avoid unwraps.
- When changing schema or storage layout, add integration tests that cover both write and search paths.
- Semgrep runs on tests; keep them clean of forbidden patterns unless explicitly excluded.

## Commit & Pull Request Guidelines
- Commits: short, imperative subject; one logical change per commit; bump `Cargo.toml`/`Cargo.lock` together when versioning.
- PRs: include what changed, why, how to run/verify (commands above), and note any config/env updates (`FASTEMBED_CACHE_PATH`, `--db-path`, hook changes).
- Update README/AGENTS when altering entrypoints, scripts, or config surfaces; avoid leaving stale instructions.
- Keep generated caches (`.fastembed_cache`, `.lancedb/`) and build artifacts out of git; they are ignored but should not be added.
