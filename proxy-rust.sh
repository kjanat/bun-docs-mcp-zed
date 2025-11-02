#!/usr/bin/env bash
# Wrapper script to run Rust MCP proxy
# This allows easy integration with Zed extension

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROXY_BINARY="$SCRIPT_DIR/proxy/target/release/bun-docs-mcp-proxy"

# Check if binary exists
if [ ! -f "$PROXY_BINARY" ]; then
	echo "Error: Rust proxy binary not found at: $PROXY_BINARY" >&2
	echo "Build it with: cd proxy && cargo build --release" >&2
	exit 1
fi

# Execute the proxy
exec "$PROXY_BINARY"
