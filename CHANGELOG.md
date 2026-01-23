# Changelog

All notable changes to the Bun Docs MCP extension for Zed.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

## [Unreleased]

## [0.2.0] - 2026-01-23

### Changed

- **Pinned Proxy Version**: Proxy binary pinned to v0.3.0, removing auto-update
  polling
- **Rust Edition**: Bump from 2021 to 2024
- **Settings Schema**: Add `BunDocsMcpSettings` struct with JsonSchema for Zed
  settings UI integration
- **Configuration**: Implement `context_server_configuration` providing
  installation instructions and default settings
- **Custom Binary Path**: Support custom binary path via settings
- **Dependencies**: Remove semver, add schemars/serde
- **Formatting**: Compact YAML workflow syntax, add schema comments to workflows
- **CI**: Upgrade actions
  - actions/checkout v5->v6
  - actions/cache v4->v5
  - actions/upload-artifact v4->v6
  - huacnlee/zed-extension-action v1->v2 with `create-pullrequest`
- **Dependabot**: Change schedule from weekly to monthly
- **License**: Update copyright year to 2026

### Added

- `configuration/` directory with `default_settings.jsonc` and
  `installation_instructions.md`
- `.dprint.jsonc` and `tombi.toml` for formatting configuration
- `clippy.toml` to allow transitive hashbrown duplicate (zed_extension_api dep)
- `autofix.ci` workflow for auto-formatting PRs
- `AGENTS.md` for AI agent instructions (CLAUDE.md now references it)
- Cargo.toml metadata: repository, keywords, categories

### Removed

- `.pre-commit-config.yaml` (replaced by autofix.ci workflow)
- Release profile overrides in Cargo.toml

### Fixed

- Trim whitespace from default settings content

## [0.1.1] - 2025-11-03

### Fixed

- **Platform Detection Bug**: Fixed "Unsupported platform: wasm32" error by
  migrating from `std::env::consts` to `zed::current_platform()`
  - `std::env::consts::OS` returns `"wasm32"` when running as WASM, breaking
    platform detection
  - `zed::current_platform()` correctly returns the host OS and architecture
  - Fixes binary download failures on all platforms

### Changed

- **Code Organization**: Extracted platform-specific archive names to constants
  - Reduces duplication between implementation and tests
  - Improves maintainability and reduces risk of typos
  - Constants: `ARCHIVE_LINUX_X64`, `ARCHIVE_LINUX_ARM64`, `ARCHIVE_MACOS_X64`,
    `ARCHIVE_MACOS_ARM64`, `ARCHIVE_WINDOWS_X64`, `ARCHIVE_WINDOWS_ARM64`

- **Test Strategy**: Updated tests to acknowledge WASM runtime limitations
  - Platform detection functions require Zed's WASM runtime
  - Tests now validate patterns and constants instead of actual platform
    detection
  - Added clear documentation about testing limitations

### Added

- **Documentation**: Created comprehensive `TESTING.md` with manual testing
  procedures
  - Platform detection verification steps
  - Binary download verification
  - Troubleshooting common issues
  - Cross-platform testing checklist

- **Inline Documentation**: Added rustdoc comments to all major functions
  - `get_platform_archive_name()` - Platform-specific archive detection
  - `get_binary_name()` - Binary filename for current platform
  - `get_binary_version()` - Version extraction from binary
  - `should_check_for_update()` - Update check throttling
  - `check_and_update_binary()` - Auto-update mechanism
  - `ensure_binary()` - Main binary download/cache logic

### Technical Details

#### Platform Detection Migration

**Before (broken):**

```rust
match (std::env::consts::OS, std::env::consts::ARCH) {
    ("linux", "x86_64") => Ok("..."),
    // Returns "wasm32" when running as WASM ❌
}
```

**After (working):**

```rust
let (os, arch) = zed::current_platform();
match (os, arch) {
    (zed::Os::Linux, zed::Architecture::X8664) => Ok(ARCHIVE_LINUX_X64),
    // Returns actual host platform ✅
}
```

#### Supported Platforms

- Linux: x86_64, aarch64
- macOS: x86_64 (Intel), aarch64 (Apple Silicon)
- Windows: x86_64, aarch64

#### Build Details

- WASM binary size: 172 KB (optimized release build)
- All 12 unit tests passing
- Zero compiler warnings
- Clean clippy output

## [0.1.0] - 2025-11-02

### Added

- Initial release with pure Rust implementation
- Automatic binary download from GitHub Releases
- Platform detection for 6 platforms (Linux/macOS/Windows × x86_64/aarch64)
- Auto-update mechanism (checks every 24 hours)
- Semantic versioning support for updates
- Binary caching for performance
- MCP server integration for Bun documentation

### Technical

- Pure Rust implementation (no Node.js/Bun runtime required)
- WASM32-WASIP2 target for Zed extension
- Native binaries for all platforms
- 10x faster startup than TypeScript implementation
- 2.7 MB binary footprint vs 50+ MB with runtime

<!--tag-link-definitions-start-->

[Unreleased]: https://github.com/kjanat/bun-docs-mcp-zed/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/kjanat/bun-docs-mcp-zed/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/kjanat/bun-docs-mcp-zed/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/kjanat/bun-docs-mcp-zed/compare/v0.0.1...v0.1.0

<!--tag-link-definitions-end-->

<!--link-definitions-start-->

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html

<!--link-definitions-end-->

<!--markdownlint-disable-file no-duplicate-heading-->
