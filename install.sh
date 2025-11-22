#!/bin/bash
set -euo pipefail

echo "=== mcp_memex setup ==="

# Build Rust binary
echo "Building release binary..."
cargo build --release

if [[ "${1:-}" == "--bundle-macos" ]]; then
  echo "Creating macOS app bundle..."
  ./build-macos.sh
  BIN_PATH="$HOME/.mcp-servers/MCPServer.app/Contents/MacOS/mcp_memex"
else
  BIN_PATH="$(pwd)/target/release/mcp_memex"
fi

echo ""
echo "=== Done ==="
echo "Binary: $BIN_PATH"
echo ""
echo "Example MCP host config:"
cat <<JSON
{
  "mcpServers": {
    "mcp_memex": {
      "command": "$BIN_PATH",
      "args": ["--log-level", "info"]
    }
  }
}
JSON

echo ""
echo "Notes:"
echo "- The MLX HTTP bridge is optional. By default, DRAGON_BASE_URL=http://localhost."
echo "- To force local-only embeddings, set DISABLE_MLX=1."