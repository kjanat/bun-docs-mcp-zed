# Changelog

All notable changes to the Bun Docs MCP extension for Zed.

## [0.2.0] - 2025-11-04

### Added

- **Version-Specific Binary Storage**: Binaries are now stored in version-specific directories
  - Example: `bun-docs-mcp-proxy-v0.1.2/` instead of fixed `bun-docs-mcp-proxy/`
  - Enables safe updates with multiple versions coexisting during transitions
  - Follows Zed extension best practices for dependency management

- **Automatic Cleanup**: Old binary versions are automatically removed after successful updates
  - Keeps storage footprint minimal
  - Prevents accumulation of outdated binaries
  - Runs after each successful binary download

- **Improved Update Mechanism**: Refactored update checking logic
  - Version is now cached alongside binary path for faster checks
  - Update checks no longer require executing the binary
  - Cleaner separation between update detection and download

### Changed

- **Binary Download Path**: Downloads now go to `bun-docs-mcp-proxy-v{VERSION}/` directories
  - **Breaking**: Manual cleanups should use new path pattern
  - Improves update reliability and prevents conflicts

- **Extension Version**: Bumped to 0.2.0 to reflect new version management features
  - Minor version bump follows semantic versioning (new features, backward compatible)

### Technical Details

#### Version-Specific Storage Example

**Before:**
```
~/.local/share/zed/extensions/work/bun-docs-mcp/
└── bun-docs-mcp-proxy/
    └── bun-docs-mcp-proxy
```

**After:**
```
~/.local/share/zed/extensions/work/bun-docs-mcp/
└── bun-docs-mcp-proxy-v0.1.2/
    └── bun-docs-mcp-proxy
```

#### New Functions

- `cleanup_old_versions()` - Removes old version directories after updates
- Updated `check_and_update_binary()` - Uses cached version instead of executing binary
- Updated `ensure_binary()` - Downloads to version-specific directories and triggers cleanup

#### Documentation Updates

- Added "Auto-Update Behavior" section to README.md
- Enhanced ARCHITECTURE.md with auto-update mechanism details
- Documented version-specific storage and cleanup strategies
- Added manual update instructions

#### Tests

- Added `test_version_directory_naming()` to verify directory naming conventions
- Updated `test_binary_path_construction()` to use version-specific paths
- All 13 unit tests passing
- Zero compiler warnings

## [0.1.1] - 2025-11-03

### Fixed

- **Platform Detection Bug**: Fixed "Unsupported platform: wasm32" error by migrating from `std::env::consts` to `zed::current_platform()`
  - `std::env::consts::OS` returns `"wasm32"` when running as WASM, breaking platform detection
  - `zed::current_platform()` correctly returns the host OS and architecture
  - Fixes binary download failures on all platforms

### Changed

- **Code Organization**: Extracted platform-specific archive names to constants
  - Reduces duplication between implementation and tests
  - Improves maintainability and reduces risk of typos
  - Constants: `ARCHIVE_LINUX_X64`, `ARCHIVE_LINUX_ARM64`, `ARCHIVE_MACOS_X64`, `ARCHIVE_MACOS_ARM64`, `ARCHIVE_WINDOWS_X64`, `ARCHIVE_WINDOWS_ARM64`

- **Test Strategy**: Updated tests to acknowledge WASM runtime limitations
  - Platform detection functions require Zed's WASM runtime
  - Tests now validate patterns and constants instead of actual platform detection
  - Added clear documentation about testing limitations

### Added

- **Documentation**: Created comprehensive `TESTING.md` with manual testing procedures
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

---

## Version Format

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: Backwards-compatible functionality additions
- **PATCH** version: Backwards-compatible bug fixes
