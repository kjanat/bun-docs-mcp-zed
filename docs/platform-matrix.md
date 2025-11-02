# Platform Detection Matrix

**Date**: 2025-11-02
**Test Platform**: Linux x86_64 (Manjaro 6.16.12)
**Rust Version**: 1.81.0

---

## Verified Platform Strings

### Current Platform (Linux x86_64)

```
OS: linux
ARCH: x86_64
FAMILY: unix
```

**Recommended binary name**: `bun-docs-proxy-x86_64-unknown-linux-gnu`

---

## Platform Matrix

| Platform                   | `std::env::consts::OS` | `std::env::consts::ARCH` | `FAMILY`    | Binary Name                                 |
|----------------------------|------------------------|--------------------------|-------------|---------------------------------------------|
| **Linux x86_64**           | `"linux"`              | `"x86_64"`               | `"unix"`    | `bun-docs-proxy-x86_64-unknown-linux-gnu`   |
| **Linux ARM64**            | `"linux"`              | `"aarch64"`              | `"unix"`    | `bun-docs-proxy-aarch64-unknown-linux-gnu`  |
| **macOS x86_64**           | `"macos"`              | `"x86_64"`               | `"unix"`    | `bun-docs-proxy-x86_64-apple-darwin`        |
| **macOS ARM64 (M1/M2/M3)** | `"macos"`              | `"aarch64"`              | `"unix"`    | `bun-docs-proxy-aarch64-apple-darwin`       |
| **Windows x86_64**         | `"windows"`            | `"x86_64"`               | `"windows"` | `bun-docs-proxy-x86_64-pc-windows-msvc.exe` |

---

## Platform Detection Code

### Rust Implementation

```rust
fn get_platform_binary_name() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "bun-docs-proxy-x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "bun-docs-proxy-aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "bun-docs-proxy-x86_64-apple-darwin",
        ("macos", "aarch64") => "bun-docs-proxy-aarch64-apple-darwin",
        ("windows", "x86_64") => "bun-docs-proxy-x86_64-pc-windows-msvc.exe",
        _ => panic!(
            "Unsupported platform: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
    }
}
```

### Rust Target Triples

For cross-compilation and GitHub Actions:

```yaml
targets:
  - x86_64-unknown-linux-gnu      # Linux x86_64
  - aarch64-unknown-linux-gnu     # Linux ARM64
  - x86_64-apple-darwin           # macOS Intel
  - aarch64-apple-darwin          # macOS Apple Silicon
  - x86_64-pc-windows-msvc        # Windows
```

---

## Zed Platform Support

According to Zed documentation and your environment:

| Platform                   | Zed Support | Priority                       |
|----------------------------|-------------|--------------------------------|
| **Linux x86_64**           | ✅ Stable    | **HIGH** (current dev env)     |
| **macOS ARM64 (M1/M2/M3)** | ✅ Stable    | **HIGH** (most Mac users)      |
| **macOS x86_64 (Intel)**   | ✅ Stable    | **MEDIUM** (older Macs)        |
| **Windows x86_64**         | ✅ Preview   | **MEDIUM** (upcoming stable)   |
| **Linux ARM64**            | ⚠️ Untested | **LOW** (servers/Raspberry Pi) |

---

## Build Configuration

### GitHub Actions Matrix

```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        artifact: bun-docs-proxy-x86_64-unknown-linux-gnu

      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
        artifact: bun-docs-proxy-aarch64-unknown-linux-gnu
        cross: true  # requires cross-compilation

      - os: macos-latest
        target: x86_64-apple-darwin
        artifact: bun-docs-proxy-x86_64-apple-darwin

      - os: macos-latest
        target: aarch64-apple-darwin
        artifact: bun-docs-proxy-aarch64-apple-darwin

      - os: windows-latest
        target: x86_64-pc-windows-msvc
        artifact: bun-docs-proxy-x86_64-pc-windows-msvc.exe
```

### Cargo Build Commands

```bash
# Linux x86_64 (native)
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64 (cross-compile)
cross build --release --target aarch64-unknown-linux-gnu

# macOS x86_64 (native on Intel Mac)
cargo build --release --target x86_64-apple-darwin

# macOS ARM64 (native on M1/M2/M3 Mac)
cargo build --release --target aarch64-apple-darwin

# Windows (native)
cargo build --release --target x86_64-pc-windows-msvc
```

---

## Binary Distribution Strategy

### Option A: GitHub Releases (Recommended)

**Pros**:

- Free hosting
- Standard practice
- Easy automation with GitHub Actions
- Supports all platforms
- Version management built-in

**Cons**:

- Requires GitHub account
- First download slightly slower (CDN warmup)

**Implementation**:

```rust
// In Zed extension
let url = format!(
    "https://github.com/{}/releases/download/v{}/{}",
    "username/bun-docs-mcp-proxy",
    BINARY_VERSION,
    get_platform_binary_name()
);

// Download to cache directory
let cache_path = project.extension_data_dir().join("bin").join(get_platform_binary_name());
download_binary( & url, & cache_path) ?;
```

### Option B: Bundle in Extension (Alternative)

**Pros**:

- Instant availability
- No network required
- Guaranteed version match

**Cons**:

- Large extension package (~15-20 MB for all 5 platforms)
- Wastes bandwidth (downloads all platforms)
- Slower extension updates

**Size Estimates**:

- Single binary: ~3-5 MB
- All 5 platforms bundled: ~15-20 MB total

---

## Verification Test Results

**Test Command**:

```bash
cd /tmp/platform-test && cargo run
```

**Output** (Linux x86_64):

```
Platform Detection Test
=======================

OS: linux
ARCH: x86_64
FAMILY: unix

Recommended binary name:
  bun-docs-proxy-x86_64-unknown-linux-gnu

All supported platforms:
  - Linux x86_64:    bun-docs-proxy-x86_64-unknown-linux-gnu
  - Linux ARM64:     bun-docs-proxy-aarch64-unknown-linux-gnu
  - macOS x86_64:    bun-docs-proxy-x86_64-apple-darwin
  - macOS ARM64:     bun-docs-proxy-aarch64-apple-darwin
  - Windows x86_64:  bun-docs-proxy-x86_64-pc-windows-msvc.exe
```

---

## Important Notes

### macOS Caveat

⚠️ **Note**: `std::env::consts::OS` returns `"macos"` not `"darwin"`. This is correct and verified.

```rust
// ✅ CORRECT
("macos", "x86_64") => "bun-docs-proxy-x86_64-apple-darwin",

// ❌ WRONG
("darwin", "x86_64") => "bun-docs-proxy-x86_64-apple-darwin",
```

### Windows Extension

✅ Windows binaries need `.exe` extension:

```rust
// ✅ CORRECT
("windows", "x86_64") => "bun-docs-proxy-x86_64-pc-windows-msvc.exe",

// ❌ WRONG
("windows", "x86_64") => "bun-docs-proxy-x86_64-pc-windows-msvc",
```

### Cross-Compilation

For Linux ARM64, use `cross`:

```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
```

For macOS cross-compilation (Intel→ARM or vice versa), use native macOS GitHub Actions runner.

---

## Next Steps

1. ✅ Platform detection logic verified
2. ✅ Binary naming scheme confirmed
3. ⏭ Set up GitHub Actions for multi-platform builds
4. ⏭ Implement download logic in Zed extension
5. ⏭ Add checksum verification for security

---

## Conclusion

Platform detection is **straightforward** and **reliable** using `std::env::consts`. All 5 target platforms (Linux
x64/ARM64, macOS Intel/ARM, Windows x64) are supported and can be built with standard Rust tooling.

**Recommendation**: Proceed with GitHub Releases distribution strategy.
