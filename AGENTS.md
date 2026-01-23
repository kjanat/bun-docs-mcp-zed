# Bun Docs MCP Extension for Zed

Repository: [github:kjanat/bun-docs-mcp-zed]

## Quick Reference

See [ARCHITECTURE.md] for complete technical details.

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

1. Fork [github:zed-industries/extensions]
2. Add submodule:

   ```sh
   git submodule add https://github.com/kjanat/bun-docs-mcp-zed.git extensions/bun-docs-mcp
   ```

3. Update `extensions.toml`:

   ```toml
   [bun-docs-mcp]
   submodule = "extensions/bun-docs-mcp"
   version   = "X.X.X"
   ```

4. Run `pnpm sort-extensions`
5. Open PR

## Official Documentation

- [Zed Extension Development]
- [MCP Server Extensions]
- [Extension API Reference]

## Project Conventions

- **Language**: Pure Rust (no TypeScript/JavaScript)
- **Build Target**: wasm32-wasip2 for extension WASM
- **Binary Distribution**: Auto-download from GitHub Releases
- **Testing**: Unit tests in src/lib.rs
- **Formatting**: cargo fmt (enforced by pre-commit)

## Git Restrictions

- **NEVER** create, push, modify, or delete git tags unless the user explicitly
  requests it
- Only push commits when explicitly asked

## Habits

Update CHANGELOG.md with your modifications. If a [x.x.x] is the top of the
CHANGELOG.md, then add a new [Unreleased] section.

Before comitting, make sure to `dprint fmt`.

<!--link-definitions-->

[ARCHITECTURE.md]: https://github.com/kjanat/bun-docs-mcp-zed/blob/master/ARCHITECTURE.md
[Extension API Reference]: https://github.com/zed-industries/zed/tree/main/crates/extension_api
[github:kjanat/bun-docs-mcp-zed]: https://github.com/kjanat/bun-docs-mcp-zed
[github:zed-industries/extensions]: https://github.com/zed-industries/extensions
[MCP Server Extensions]: https://zed.dev/docs/extensions/mcp-extensions
[Zed Extension Development]: https://zed.dev/docs/extensions/developing-extensions
