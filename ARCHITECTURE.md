# Architecture

## Overview

This Zed extension provides MCP (Model Context Protocol) integration for searching Bun documentation. It's implemented as a pure Rust binary that communicates with Zed via stdio.

## Components

### 1. Zed Extension (Rust WASM)
- **Location**: `src/lib.rs`
- **Purpose**: Implements `context_server_command` to provide the command Zed needs to start the MCP server
- **Build Target**: `wasm32-wasip2`

### 2. MCP Binary (Rust Native)
- **Repository**: https://github.com/kjanat/bun-docs-mcp-proxy
- **Purpose**: Standalone MCP server that proxies requests to https://bun.com/docs/mcp
- **Distribution**: Downloaded automatically from GitHub Releases

## Communication Flow

```
Zed (stdio) ← Extension WASM → Native Binary (stdin/stdout) → HTTPS → bun.com/docs/mcp
```

1. **Extension**: Returns command to execute native binary
2. **Binary**: Reads JSON-RPC from stdin, makes HTTPS requests, writes responses to stdout
3. **Zed**: Communicates with binary via stdio pipes

## Platform Support

| Platform | Architecture | Binary Name |
|----------|-------------|-------------|
| Linux | x86_64 | bun-docs-mcp-proxy-linux-x64 |
| Linux | aarch64 | bun-docs-mcp-proxy-linux-arm64 |
| macOS | x86_64 | bun-docs-mcp-proxy-macos-x64 |
| macOS | aarch64 | bun-docs-mcp-proxy-macos-arm64 |
| Windows | x86_64 | bun-docs-mcp-proxy-windows-x64.exe |

## Build Process

```bash
# Build extension WASM
cargo build --target wasm32-wasip2 --release

# Binary is downloaded automatically on first use
# See: https://github.com/kjanat/bun-docs-mcp-proxy/releases
```

## Migration History

Originally implemented in TypeScript with Node.js, migrated to pure Rust for:
- Zero runtime dependencies
- 10x faster startup time
- Smaller distribution size
- Better platform compatibility

For detailed migration history, see git history from commit `8a68491` onward.
