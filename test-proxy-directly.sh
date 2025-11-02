#!/usr/bin/env bash
# Direct proxy test without Zed
# Sends a mock MCP request directly to the proxy to test logging

set -e

echo "=== Direct Proxy Test (No Zed Required) ==="
echo ""

# Clean up old logs
rm -f /tmp/mcp-traffic.jsonl /tmp/mcp-debug.log

# Create a test JSON-RPC request (simulating what Zed would send)
TEST_REQUEST='{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"SearchBun","arguments":{"query":"Bun.serve"}}}'

echo "Test request:"
echo "$TEST_REQUEST"
echo ""

echo "Starting proxy with DEBUG_CAPTURE=1..."
echo ""

# Run proxy with test input
echo "$TEST_REQUEST" | DEBUG_CAPTURE=1 timeout 5s bun --bun run proxy.ts 2>/tmp/mcp-debug.log || true

echo ""
echo "=== Capture Analysis ==="
echo ""

if [ -f /tmp/mcp-traffic.jsonl ]; then
	LINE_COUNT=$(wc -l </tmp/mcp-traffic.jsonl)
	echo "✓ Captured $LINE_COUNT log entries"
	echo ""

	# Show what was captured
	echo "=== Captured Traffic ==="
	cat /tmp/mcp-traffic.jsonl | jq '.' || cat /tmp/mcp-traffic.jsonl
	echo ""

	# Extract message types
	echo "=== Message Type Breakdown ==="
	echo "STDIN:     $(grep -c '"STDIN"' /tmp/mcp-traffic.jsonl || echo 0)"
	echo "HTTP_REQ:  $(grep -c '"HTTP_REQ"' /tmp/mcp-traffic.jsonl || echo 0)"
	echo "HTTP_RES:  $(grep -c '"HTTP_RES"' /tmp/mcp-traffic.jsonl || echo 0)"
	echo "SSE_CHUNK: $(grep -c '"SSE_CHUNK"' /tmp/mcp-traffic.jsonl || echo 0)"
	echo "STDOUT:    $(grep -c '"STDOUT"' /tmp/mcp-traffic.jsonl || echo 0)"
	echo ""

	echo "✓ Logging is working!"
	echo ""
	echo "Files created:"
	echo "  - /tmp/mcp-traffic.jsonl (captured protocol traffic)"
	echo "  - /tmp/mcp-debug.log (stderr output)"

else
	echo "✗ No traffic captured"
	echo ""
	echo "Debug log:"
	cat /tmp/mcp-debug.log || echo "(no debug log)"
fi

echo ""
echo "Next: Analyze captured data to document protocol format"
