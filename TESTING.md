# Testing Guide

This document describes how to test the Bun Docs MCP extension for Zed,
including manual testing procedures and troubleshooting steps.

## Testing Limitations

The extension uses `zed::current_platform()` from the Zed Extension API, which
only works within Zed's WASM runtime environment. This means:

- :white_check_mark: **Unit tests** verify logic that doesn't depend on the Zed
  runtime (version parsing, semver comparison, path construction, etc.)
- :x: **Unit tests cannot** verify platform detection or binary download
  behavior
- :white_check_mark: **Integration tests** must be performed manually by
  installing the extension in Zed

## Manual Testing Procedure

### 1. Build the Extension

Build the WASM binary for the extension:

```bash
cargo build --target wasm32-wasip2 --release
```

**Expected output:**

```console
Compiling bun-docs-mcp-zed v0.1.0
Finished `release` profile [optimized] target(s) in X.XXs
```

**Verify WASM binary exists:**

```bash
ls -lh target/wasm32-wasip2/release/bun_docs_mcp_zed.wasm
```

---

### 2. Install as Dev Extension in Zed

1. **Open Zed**
2. **Open Command Palette**: <kbd>Cmd/Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd>
3. **Run**: "zed: install dev extension"
4. **Select**: Navigate to this project directory (`/path/to/bun-docs-mcp-zed`)

**Expected behavior:**

- Extension loads successfully
- No errors in Zed's log panel

---

### 3. Verify Binary Download

The extension should automatically download the platform-specific binary on
first use.

#### Check Expected Binary Location

The binary is downloaded to Zed's extension work directory. To find it:

1. **Open Zed's log panel**: <kbd>Cmd/Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Y</kbd>
2. **Look for log messages** about binary download

**Expected download locations by platform:**

| Platform            | Binary Path Pattern                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------ |
| **Linux x86_64**    | `~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy`                |
| **Linux aarch64**   | `~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy`                |
| **macOS x86_64**    | `~/Library/Application Support/Zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy` |
| **macOS aarch64**   | `~/Library/Application Support/Zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy` |
| **Windows x86_64**  | `%APPDATA%\Zed\extensions\work\bun-docs-mcp\bun-docs-mcp-proxy\bun-docs-mcp-proxy.exe`                 |
| **Windows aarch64** | `%APPDATA%\Zed\extensions\work\bun-docs-mcp\bun-docs-mcp-proxy\bun-docs-mcp-proxy.exe`                 |

#### Verify Binary Downloaded

**On Unix (Linux/macOS):**

```bash
ls -lh ~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/
# or on macOS:
ls -lh ~/Library/Application\ Support/Zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/
```

**On Windows:**

```powershell
dir "$env:APPDATA\Zed\extensions\work\bun-docs-mcp\bun-docs-mcp-proxy\"
```

**Expected output:**

- Binary file exists
- File size is reasonable (typically 2-5 MB)
- Binary is executable (Unix: has `x` permission)

---

### 4. Test MCP Server Functionality

The extension provides a Model Context Protocol server for Bun documentation.

#### Enable MCP Server

1. Open Zed settings: <kbd>Cmd/Ctrl</kbd>+<kbd>,</kbd>
2. Add context server configuration:

   ```json
   {
     "context_servers": {
       "bun-docs-mcp": {
         "enabled": true
       }
     }
   }
   ```

#### Verify Server Starts

**Check Zed logs** for MCP server startup:

1. Open log panel: <kbd>Cmd/Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Y</kbd>
2. Look for messages about context server "bun-docs-mcp"

**Expected log messages:**

- Context server started successfully
- No error messages about missing binary
- No platform detection errors

---

### 5. Verify Platform Detection

The extension should detect your platform correctly and download the appropriate
binary.

#### Platform Detection Test Matrix

| Your Platform                 | Expected Archive                          | Binary Name              |
| ----------------------------- | ----------------------------------------- | ------------------------ |
| Linux x86_64                  | `bun-docs-mcp-proxy-linux-x86_64.tar.gz`  | `bun-docs-mcp-proxy`     |
| Linux aarch64                 | `bun-docs-mcp-proxy-linux-aarch64.tar.gz` | `bun-docs-mcp-proxy`     |
| macOS x86_64 (Intel)          | `bun-docs-mcp-proxy-macos-x86_64.tar.gz`  | `bun-docs-mcp-proxy`     |
| macOS aarch64 (Apple Silicon) | `bun-docs-mcp-proxy-macos-aarch64.tar.gz` | `bun-docs-mcp-proxy`     |
| Windows x86_64                | `bun-docs-mcp-proxy-windows-x86_64.zip`   | `bun-docs-mcp-proxy.exe` |
| Windows aarch64               | `bun-docs-mcp-proxy-windows-aarch64.zip`  | `bun-docs-mcp-proxy.exe` |

#### Verify Correct Binary Downloaded

Check that the downloaded binary matches your platform:

**On Linux:**

```bash
file ~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy
```

**Expected output examples:**

- Linux x86_64: `ELF 64-bit LSB executable, x86-64`
- Linux aarch64: `ELF 64-bit LSB executable, ARM aarch64`

**On macOS:**

```bash
file ~/Library/Application\ Support/Zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy
```

**Expected output examples:**

- macOS x86_64: `Mach-O 64-bit executable x86_64`
- macOS aarch64: `Mach-O 64-bit executable arm64`

**On Windows:**

```powershell
(Get-Item "$env:APPDATA\Zed\extensions\work\bun-docs-mcp\bun-docs-mcp-proxy\bun-docs-mcp-proxy.exe").VersionInfo
```

---

### 6. Test Binary Auto-Update (Optional)

The extension checks for updates every 24 hours.

#### Force Update Check

1. Delete the binary:

   ```bash
   # Unix:
   rm -rf ~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/
   # or macOS:
   rm -rf ~/Library/Application\ Support/Zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/
   ```

2. Reload Zed extensions: <kbd>Cmd/Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd> →
   "zed: reload extensions"

3. Verify binary re-downloads automatically

**Expected behavior:**

- Binary downloads on next context server startup
- Same version or newer version downloaded
- No errors in logs

---

## Troubleshooting

### Error: "Unsupported platform"

**Symptom:** Zed logs show `ERROR: Unsupported platform: ...`

**Possible Causes:**

1. Running on unsupported platform/architecture
2. Platform detection bug (should not occur with `zed::current_platform()`)

**Solution:**

1. Check your platform: `uname -m` and `uname -s`
2. Verify it's in the supported list (see Platform Detection Test Matrix above)
3. If supported but still failing, file an issue:
   https://github.com/kjanat/bun-docs-mcp-zed/issues

---

### Error: "Failed to download binary"

**Symptom:** Extension fails to download the binary from GitHub

**Possible Causes:**

1. Network connectivity issues
2. GitHub Releases unavailable
3. Rate limiting from GitHub API

**Solution:**

1. Check internet connection
2. Verify GitHub is accessible:
   https://github.com/kjanat/bun-docs-mcp-proxy/releases
3. Check Zed logs for specific error message
4. Wait a few minutes and try reloading extensions

---

### Error: "Binary not found after extraction"

**Symptom:** Download succeeds but binary is not found

**Possible Causes:**

1. Archive extraction failed
2. Binary name mismatch
3. Incorrect path construction

**Solution:**

1. Check Zed logs for extraction errors
2. Manually inspect the downloaded archive structure
3. File an issue with logs: https://github.com/kjanat/bun-docs-mcp-zed/issues

---

### Binary Downloaded but Context Server Won't Start

**Symptom:** Binary exists but MCP server fails to start

**Possible Causes:**

1. Binary not executable (Unix)
2. Missing dependencies
3. Binary corruption

**Solution:**

**On Unix:**

```bash
# Check permissions
ls -l ~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy

# Make executable if needed
chmod +x ~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy

# Test binary manually
~/.local/share/zed/extensions/work/bun-docs-mcp/bun-docs-mcp-proxy/bun-docs-mcp-proxy --version
```

**On Windows:**

```powershell
# Test binary manually
& "$env:APPDATA\Zed\extensions\work\bun-docs-mcp\bun-docs-mcp-proxy\bun-docs-mcp-proxy.exe" --version
```

**Expected output:** Version string like `bun-docs-mcp-proxy 0.1.2`

---

## Testing Checklist

Use this checklist when testing the extension:

### Build & Installation

- [ ] Extension builds successfully with
      `cargo build --target wasm32-wasip2 --release`
- [ ] WASM binary exists at `target/wasm32-wasip2/release/bun_docs_mcp_zed.wasm`
- [ ] Extension installs as dev extension in Zed without errors
- [ ] No errors in Zed log panel after installation

### Binary Download

- [ ] Binary downloads automatically on first use
- [ ] Binary is placed in correct platform-specific location
- [ ] Binary has correct platform/architecture (verified with `file` command)
- [ ] Binary is executable (Unix: has execute permission)
- [ ] Binary version can be retrieved with `--version` flag

### MCP Server

- [ ] Context server can be enabled in Zed settings
- [ ] Context server starts without errors
- [ ] No "unsupported platform" errors in logs
- [ ] MCP server responds to requests (if applicable)

### Auto-Update

- [ ] Deleting binary triggers re-download
- [ ] Update check respects 24-hour interval
- [ ] Newer versions are downloaded when available

### Cross-Platform (if testing multiple platforms)

- [ ] Linux x86_64 downloads correct binary
- [ ] Linux aarch64 downloads correct binary
- [ ] macOS x86_64 downloads correct binary
- [ ] macOS aarch64 downloads correct binary
- [ ] Windows x86_64 downloads correct binary
- [ ] Windows aarch64 downloads correct binary (if available for testing)

---

## Reporting Issues

When reporting issues, please include:

1. **Platform information:**
   - OS: `uname -s` (or Windows version)
   - Architecture: `uname -m` (or Windows architecture)
   - Zed version

2. **Extension information:**
   - Extension version from `extension.toml`
   - Build command used
   - WASM binary hash:
     `shasum -a 256 target/wasm32-wasip2/release/bun_docs_mcp_zed.wasm`

3. **Logs:**
   - Complete Zed log output from the log panel
   - Any error messages
   - Expected vs actual behavior

4. **Steps to reproduce:**
   - Exact steps that trigger the issue
   - Whether issue is reproducible

**Issue tracker:** https://github.com/kjanat/bun-docs-mcp-zed/issues

---

## Additional Resources

- **Zed Extension Development:**
  https://zed.dev/docs/extensions/developing-extensions
- **MCP Server Extensions:** https://zed.dev/docs/extensions/mcp-extensions
- **Binary Repository:** https://github.com/kjanat/bun-docs-mcp-proxy
- **Extension API Reference:** https://docs.rs/zed_extension_api/

<!--markdownlint-disable-file no-inline-html no-bare-urls-->
