# Architecture

## Overview

This Zed extension provides MCP (Model Context Protocol) integration for searching Bun documentation. It's implemented as a pure Rust binary that communicates with Zed via stdio.

## Components

### 1. Zed Extension (Rust WASM)

- **Location**: [`src/lib.rs`](./src/lib.rs)
- **Purpose**: Implements `context_server_command` to provide the command Zed needs to start the MCP server
- **Build Target**: `wasm32-wasip2`

### 2. MCP Binary (Rust Native)

- **Repository**: [kjanat/bun-docs-mcp-proxy][bun-docs-mcp-proxy]
- **Purpose**: Standalone MCP server that proxies requests to `https://bun.com/docs/mcp`
- **Distribution**: Downloaded automatically from [GitHub Releases][releases]

## Communication Flow

```mermaid
graph LR
    A["Zed<br/>(stdio)"] <-->|Extension WASM| B["Native Binary<br/>(stdin/stdout)"]
    B -->|HTTPS| C["bun.com/docs/mcp"]
    C -->|HTTPS| B
```

1. **Extension**: Returns command to execute native binary
2. **Binary**: Reads JSON-RPC from stdin, makes HTTPS requests, writes responses to stdout
3. **Zed**: Communicates with binary via stdio pipes

## Platform Support

| Platform | Architecture | Binary Name                              |
| -------- | ------------ | ---------------------------------------- |
| Linux    | `x86_64`     | `bun-docs-mcp-proxy-linux-x86_64`        |
| Linux    | `aarch64`    | `bun-docs-mcp-proxy-linux-aarch64`       |
| macOS    | `x86_64`     | `bun-docs-mcp-proxy-macos-x86_64`        |
| macOS    | `aarch64`    | `bun-docs-mcp-proxy-macos-aarch64`       |
| Windows  | `x86_64`     | `bun-docs-mcp-proxy-windows-x86_64.exe`  |
| Windows  | `aarch64`    | `bun-docs-mcp-proxy-windows-aarch64.exe` |

## Build Process

```bash
# Build extension WASM
cargo build --target wasm32-wasip2 --release
```

The binary is downloaded automatically.  
See: [kjanat/bun-docs-mcp-proxy/releases][releases]

## Auto-Update Mechanism

The extension implements automatic updates with the following design:

### Version-Specific Storage

Binaries are stored in version-specific directories to support seamless updates:

```
~/.local/share/zed/extensions/work/bun-docs-mcp/
├── bun-docs-mcp-proxy-v0.1.2/
│   └── bun-docs-mcp-proxy
└── bun-docs-mcp-proxy-v0.1.3/  (after update)
    └── bun-docs-mcp-proxy
```

This approach ensures:
- **No Conflicts**: Multiple versions can coexist during updates
- **Safe Updates**: Old version remains available until new one is verified
- **Clean State**: Old versions are automatically cleaned up after successful updates

### Update Flow

1. **First Use**: Downloads latest binary from GitHub Releases
2. **Daily Checks**: Every 24 hours, checks for new releases using GitHub API
3. **Auto-Download**: When newer version found, downloads on next extension use
4. **Cleanup**: After successful download, removes old version directories
5. **No Disruption**: Updates happen transparently without affecting workflow

### Implementation Details

- **Update Interval**: 24 hours (configurable via `UPDATE_CHECK_INTERVAL_SECS`)
- **Version Comparison**: Uses semantic versioning (`semver` crate)
- **Error Handling**: Network errors are silently ignored to avoid disrupting users
- **Cache Management**: In-memory cache of binary path and version to minimize disk I/O

### Publishing Updates

Extension updates are published via GitHub Actions when new tags are pushed:

```bash
git tag v0.1.2
git push origin v0.1.2
```

The [release workflow](.github/workflows/release.yml) automatically:
1. Builds the WASM extension
2. Creates a PR to [zed-industries/extensions][zed-extensions]
3. Updates the extension registry when merged

[zed-extensions]: https://github.com/zed-industries/extensions

## Migration History

Originally implemented in TypeScript, migrated to pure Rust for:

- **Zero runtime dependencies** - No Node.js or Bun required
- **10x faster startup** - 4ms vs 40ms+ for JavaScript runtimes
- **Smaller footprint** - 2.7 MB vs 50+ MB with runtime
- **Better platform support** - Native binaries for 6 platforms

Migration completed in commit [`16daa94`](https://github.com/kjanat/bun-docs-mcp-zed/commit/16daa94) (November 2025).  
For detailed history, see git log from [`8a68491`](https://github.com/kjanat/bun-docs-mcp-zed/compare/8a68491...HEAD) onward.

[bun-docs-mcp-proxy]: https://github.com/kjanat/bun-docs-mcp-proxy
[releases]: https://github.com/kjanat/bun-docs-mcp-proxy/releases
