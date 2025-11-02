# Manual Test Instructions - Rust Proxy with Zed

**Objective**: Test the new Rust MCP proxy binary directly in Zed before full automation.

---

## Setup Complete ✅

The following changes have been made:

1. ✅ **Backed up TypeScript proxy**: `proxy.ts.backup`
2. ✅ **Created wrapper script**: `proxy-rust.sh` → calls Rust binary
3. ✅ **Updated extension.toml**: Now uses `./proxy-rust.sh` instead of `node`
4. ✅ **Rust binary ready**: `proxy/target/release/bun-docs-mcp-proxy` (2.7 MB, 4 ms startup)

---

## How to Test in Zed

### Step 1: Reload Extension in Zed

Since you're using a dev extension:

```bash
# Option A: Reinstall dev extension
# In Zed: Cmd+Shift+P (or Ctrl+Shift+P)
# Run: "zed: install dev extension"
# Select: ~/projects/bun-docs-mcp-zed

# Option B: Just restart Zed
# Close and reopen Zed
```

### Step 2: Verify Extension Loaded

Check Zed's extension log:

- Open: `zed: open log` (<kbd>Cmd/Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd>)
- Look for: "Bun Docs MCP Proxy starting"

### Step 3: Test Search

1. **Open Assistant**: <kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>A</kbd> (macOS) or
   <kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>A</kbd> (Linux/Windows)

2. **Select bun-docs context**:
    - In the context dropdown, enable "bun-docs-mcp"

3. **Ask a question**:
   ```
   How does Bun.serve work?
   ```
   or
   ```
   Explain WebSocket support in Bun
   ```

### Step 4: Verify Results

**Expected**:

- Response appears in ~400-500ms (same as TypeScript)
- Contains Bun documentation snippets with titles, links, content
- No errors in Zed log

**Success indicators**:

- ✅ Faster response time (4ms startup vs 100-200ms)
- ✅ Same quality results
- ✅ No Node.js/Bun process running (check `ps aux | grep node`)
- ✅ Lower memory usage (check Activity Monitor/htop)

---

## Troubleshooting

### Issue: "Command not found: ./proxy-rust.sh"

**Fix**: Ensure script is executable

```bash
chmod +x ~/projects/bun-docs-mcp-zed/proxy-rust.sh
```

### Issue: "Rust proxy binary not found"

**Fix**: Build the binary

```bash
cd ~/projects/bun-docs-mcp-zed/proxy
cargo build --release
```

### Issue: No response or timeout

**Check logs**:

```bash
# In Zed: zed: open log
# Look for errors in [BunDocsMCP] or proxy output
```

**Test binary directly**:

```bash
cd ~/projects/bun-docs-mcp-zed/proxy
./test-proxy.sh
# Should show: ✅ All Tests Passed!
```

### Issue: "Context server not available"

**Reload extension**:

- Zed → Extensions → Find "Bun Docs MCP" → Click reload/reinstall

---

## Rollback if Needed

If the Rust proxy doesn't work, easily roll back:

```bash
cd ~/projects/bun-docs-mcp-zed

# Restore TypeScript proxy
mv proxy.ts.backup proxy.ts

# Restore extension.toml
git checkout extension.toml

# Reload extension in Zed
```

---

## Performance Comparison

### Before (TypeScript)

```bash
ps aux | grep "node.*proxy.js"
# Memory: ~30-50 MB
# CPU: ~1-2% idle
```

### After (Rust)

```bash
ps aux | grep "bun-docs-mcp-proxy"
# Memory: ~2-5 MB
# CPU: ~0.1% idle
```

**Expected improvement**: 10x less memory, virtually zero CPU when idle

---

## What Success Looks Like

1. ✅ Extension loads without errors
2. ✅ Search query returns Bun documentation
3. ✅ Results quality same as before
4. ✅ Faster response time (barely noticeable, but measurable)
5. ✅ Lower system resource usage
6. ✅ No Node.js process running

---

## After Successful Test

Once manual test passes, you can:

**A. Keep using it as-is** (works fine, just not automated)

**B. Proceed to Phase 2** - Update extension to handle:

- Platform detection
- Binary download/caching
- Automatic updates

**C. Ship it** - If it works well enough, this hybrid approach is viable:

- Extension calls `./proxy-rust.sh`
- Users install extension normally
- Binary is bundled or downloaded once

---

## Current Configuration

**extension.toml**:

```toml
[context_servers.bun-docs-mcp]
command = "./proxy-rust.sh"
```

**proxy-rust.sh**:

```bash
#!/usr/bin/env bash
exec ./proxy/target/release/bun-docs-mcp-proxy
```

---

## Next Steps After Testing

**If test succeeds**: 🎉 Continue to Phase 2 or use as-is
**If test fails**: 🔍 Debug → Fix → Retry
**If unsure**: 🤔 Ask questions, review logs, compare behavior

---

**Ready to test?** Just reload the extension in Zed and try searching! 🚀
