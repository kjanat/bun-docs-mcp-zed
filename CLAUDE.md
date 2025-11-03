# Bun Docs MCP Extension for Zed

Repository: https://github.com/kjanat/bun-docs-mcp-zed

## Quick Reference

See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete technical details.

## Development

```sh
# Build extension WASM
cargo build --target wasm32-wasip2 --release

# Install as dev extension in Zed
# Cmd/Ctrl+Shift+P → "zed: install dev extension"

# Run tests
cargo test
```

## Publishing to Zed Extensions

1. Fork https://github.com/zed-industries/extensions
2. Add submodule:
   ```sh
   git submodule add https://github.com/kjanat/bun-docs-mcp-zed.git extensions/bun-docs-mcp
   ```
3. Update `extensions.toml`:
   ```toml
   [bun-docs-mcp]
   submodule = "extensions/bun-docs-mcp"
   version = "0.1.0"
   ```
4. Run `pnpm sort-extensions`
5. Open PR

## Official Documentation

- [Zed Extension Development](https://zed.dev/docs/extensions/developing-extensions)
- [MCP Server Extensions](https://zed.dev/docs/extensions/mcp-extensions)
- [Extension API Reference](https://github.com/zed-industries/zed/tree/main/crates/extension_api)

## Project Conventions

- **Language**: Pure Rust (no TypeScript/JavaScript)
- **Build Target**: wasm32-wasip2 for extension WASM
- **Binary Distribution**: Auto-download from GitHub Releases
- **Testing**: Unit tests in src/lib.rs
- **Formatting**: cargo fmt (enforced by pre-commit)
