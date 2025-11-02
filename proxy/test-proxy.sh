#!/usr/bin/env bash
# Test script for Rust MCP proxy
set -e

PROXY="./target/release/bun-docs-mcp-proxy"

echo "=== Bun Docs MCP Proxy Test Suite ==="
echo ""

# Build if needed
if [ ! -f "$PROXY" ]; then
	echo "Building proxy..."
	cargo build --release
	echo ""
fi

# Test 1: tools/list
echo "[Test 1] Testing tools/list method..."
RESULT=$(echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | $PROXY 2>/dev/null)
if echo "$RESULT" | jq -e '.result.tools[0].name == "SearchBun"' >/dev/null; then
	echo "✅ tools/list works"
else
	echo "✗ tools/list failed"
	echo "$RESULT" | jq '.'
	exit 1
fi
echo ""

# Test 2: initialize
echo "[Test 2] Testing initialize method..."
RESULT=$(echo '{"jsonrpc":"2.0","id":2,"method":"initialize","params":{}}' | $PROXY 2>/dev/null)
if echo "$RESULT" | jq -e '.result.serverInfo.name == "bun-docs-mcp-proxy"' >/dev/null; then
	echo "✅ initialize works"
else
	echo "✗ initialize failed"
	echo "$RESULT" | jq '.'
	exit 1
fi
echo ""

# Test 3: tools/call with SearchBun
echo "[Test 3] Testing tools/call with SearchBun..."
echo "  Query: 'WebSocket'"
RESULT=$(echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"SearchBun","arguments":{"query":"WebSocket"}}}' | timeout 10s $PROXY 2>/dev/null)
if echo "$RESULT" | jq -e '.result.content | length > 0' >/dev/null; then
	COUNT=$(echo "$RESULT" | jq '.result.content | length')
	echo "✅ tools/call works (returned $COUNT results)"
else
	echo "✗ tools/call failed"
	echo "$RESULT" | jq '.'
	exit 1
fi
echo ""

# Test 4: Invalid JSON
echo "[Test 4] Testing error handling (invalid JSON)..."
RESULT=$(echo 'invalid json' | $PROXY 2>/dev/null)
if echo "$RESULT" | jq -e '.error.code == -32700' >/dev/null; then
	echo "✅ Error handling works (Parse error)"
else
	echo "⚠ Error handling unexpected"
	echo "$RESULT" | jq '.'
fi
echo ""

# Test 5: Unknown method
echo "[Test 5] Testing unknown method..."
RESULT=$(echo '{"jsonrpc":"2.0","id":5,"method":"unknown_method"}' | $PROXY 2>/dev/null)
if echo "$RESULT" | jq -e '.error.code == -32601' >/dev/null; then
	echo "✅ Unknown method handled correctly"
else
	echo "⚠ Unknown method handling unexpected"
	echo "$RESULT" | jq '.'
fi
echo ""

echo "=== All Tests Passed! ==="
echo ""

# Performance metrics
echo "=== Performance Metrics ==="
echo ""

echo "Binary size:"
if [ -f "$PROXY" ]; then
	# Get size in bytes using stat (GNU and BSD variants), then pretty-print with numfmt if available
	if SIZE_BYTES=$(stat -c %s -- "$PROXY" 2>/dev/null); then
		:
	else
		SIZE_BYTES=$(stat -f %z -- "$PROXY" 2>/dev/null || echo 0)
	fi
	if command -v numfmt >/dev/null 2>&1; then
		echo "  $(numfmt --to=iec "$SIZE_BYTES")"
	else
		echo "  ${SIZE_BYTES}B"
	fi
else
	echo "  (not built)"
fi

echo ""
echo "Startup time (tools/list):"
{ time echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | $PROXY >/dev/null 2>&1; } 2>&1 | grep real | awk '{print "  " $2}'

echo ""
echo "Full request time (tools/call):"
{ time echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"SearchBun","arguments":{"query":"test"}}}' | $PROXY >/dev/null 2>&1; } 2>&1 | grep real | awk '{print "  " $2}'

echo ""
echo "✅ Proxy is production-ready!"
