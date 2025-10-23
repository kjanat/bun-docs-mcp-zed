# Bun Docs MCP Extension for Zed

MCP server integration for searching [Bun documentation](https://bun.sh) directly in Zed editor.

## Architecture

This extension bridges Zed's stdio-based MCP client with the Bun HTTP MCP server:

```
Zed Editor (stdio) ←→ proxy.ts (Bun) ←→ https://bun.com/docs/mcp (HTTP)
```

The `proxy.ts` script translates between stdio and HTTP transports, allowing Zed to communicate with the Bun documentation server. Built with Bun's native `fetch()` API for optimal performance.

## Features

- **SearchBun Tool**: Search across the Bun knowledge base
- Direct access to API references, guides, and code examples
- Contextual content with direct links to documentation pages
- Automatic HTTP-to-stdio protocol translation

## Requirements

- **Bun**: Required to run the HTTP-to-stdio proxy ([bun.sh](https://bun.sh))
- **Rust**: Required for building the extension (via rustup)

## Installation

### Dev Installation (Local Development)

1. Install [Bun](https://bun.sh) (v1.0 or higher)
2. Install Rust via [rustup](https://www.rust-lang.org/tools/install)
3. Clone this repository
4. Build the extension: `cargo build --target wasm32-wasip1 --release`
5. In Zed, open the Extensions page
6. Click `Install Dev Extension` (or use `zed::InstallDevExtension` action)
7. Select the `bun-docs-mcp-zed` directory

### Published Installation

Once published to the Zed extension registry:

1. Open Zed
2. Go to Extensions (`Cmd/Ctrl + Shift + X`)
3. Search for "Bun Docs MCP"
4. Click Install

## Usage

After installation, the Bun documentation MCP server will be available in Zed. You can:

1. Use the `SearchBun` tool via the assistant panel
2. Ask questions about Bun functionality
3. Look up API references and examples

### Example Queries

- "How do I use Bun's HTTP server?"
- "Show me examples of Bun.serve"
- "What are Bun's testing features?"
- "How to configure bun.lockb?"

## MCP Server Details

- **Name**: Bun
- **Version**: 1.0.0
- **Transport**: HTTP
- **Endpoint**: https://bun.com/docs/mcp

### Available Tools

#### SearchBun

Search across the Bun knowledge base to find relevant information, code examples, API references, and guides.

**Parameters:**

- `query` (string, required): A query to search the content with

**Returns:**

- Contextual content with titles
- Direct links to documentation pages
- Relevant code examples

## Development

### Project Structure

```
bun-docs-mcp-zed/
├── extension.toml      # Extension metadata & context server registration
├── Cargo.toml          # Rust build configuration
├── src/
│   └── lib.rs          # Extension implementation (context_server_command)
├── proxy.ts            # Bun-native HTTP-to-stdio bridge (SSE support)
├── LICENSE             # MIT license
└── README.md           # This file
```

### Building

```bash
# Add WASM target (first time only)
rustup target add wasm32-wasip1

# Build the WASM extension
cargo build --target wasm32-wasip1 --release

# Test the proxy independently
echo '{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "test",
      "version": "1.0"
    }
  }
}' | jq -c \
   | bun proxy.ts \
   | jq .

# Test in Zed
# Use "Install Dev Extension" from the Extensions page
```

### How It Works

1. **Extension Registration**: `extension.toml` registers the `bun-docs` context server
2. **Command Provider**: `src/lib.rs` implements `context_server_command` to return the Bun command
3. **Protocol Bridge**: `proxy.ts` (written in TypeScript, runs on Bun) translates between:
   - Zed's stdio JSON-RPC messages
   - HTTP POST requests to `https://bun.com/docs/mcp`
   - Server-Sent Events (SSE) responses from the Bun server
4. **Response Handling**: SSE responses are parsed and forwarded back to Zed via stdout

### Why Bun?

The proxy is built with Bun's native APIs for several reasons:

- **Dogfooding**: The Bun documentation MCP uses Bun itself! 🐰
- **Performance**: Bun's native `fetch()` is faster than Node.js http/https modules
- **Simplicity**: Clean async/await code without callback-based APIs
- **Native SSE**: Built-in support for parsing Server-Sent Events responses
- **TypeScript**: First-class TypeScript support without transpilation

## License

MIT

## Links

- [Bun Documentation](https://bun.sh)
- [Bun MCP Server](https://bun.com/docs/mcp)
- [Zed Extensions Guide](https://zed.dev/docs/extensions)
