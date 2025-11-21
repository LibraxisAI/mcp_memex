#!/bin/bash
set -e

echo "=== MCP Rust Server Installation ==="
echo "Using uv for Python dependencies"
echo ""

# Check for uv
if ! command -v uv &> /dev/null; then
    echo "Error: uv not found. Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
    exit 1
fi

# Install Python dependencies with uv
echo "Installing Python dependencies..."
cd ~/.mcp-servers
uv pip install -r pyproject.toml

# Download MLX models
echo ""
echo "Downloading Qwen3 embedding model..."
uv run python -c "
from mlx_lm import load
print('Downloading mlx-community/Qwen3-Embedding-4B-4bit-DWQ...')
model, tokenizer = load('mlx-community/Qwen3-Embedding-4B-4bit-DWQ')
print('✓ Embedder downloaded')
"

# Convert reranker (optional for now)
echo ""
echo "Reranker conversion:"
echo "To convert Qwen3-Reranker-4B, run:"
echo "  uv run python -m mlx_lm.convert --model Qwen/Qwen3-Reranker-4B --output models/Qwen3-Reranker-4B-mlx --quantize 4bit"
echo "(Skipping for now - will use embedder for reranking)"

# Build Rust server
echo ""
echo "Building Rust server..."
./build-macos.sh

echo ""
echo "=== Installation Complete! ==="
echo ""
echo "Add to Claude Desktop config:"
echo '{'
echo '  "mcpServers": {'
echo '    "rust-rag": {'
echo '      "command": "'"$HOME/.mcp-servers/MCPServer.app/Contents/MacOS/mcp-rust-server"'",'
echo '      "env": {'
echo '        "EMBEDDER_MODEL": "mlx-community/Qwen3-Embedding-4B-4bit-DWQ",'
echo '        "CACHE_MB": "4096"'
echo '      }'
echo '    }'
echo '  }'
echo '}'