#!/usr/bin/env bash
# Test script to capture MCP protocol traffic
# This runs the proxy with debug logging enabled

set -e

echo "=== MCP Protocol Traffic Capture Test ==="
echo "This will run the proxy with DEBUG_CAPTURE=1"
echo ""
echo "INSTRUCTIONS:"
echo "1. This script will start the proxy in the background"
echo "2. In Zed, open the Assistant (Cmd+Shift+A or Ctrl+Shift+A)"
echo "3. Use the 'bun-docs' context server"
echo "4. Ask: 'How does Bun.serve work?'"
echo "5. Wait for response (5-10 seconds)"
echo "6. Press ENTER here to stop capture"
echo ""

# Clean up old logs
rm -f /tmp/mcp-traffic.jsonl /tmp/mcp-debug.log

# Start proxy with debug logging
echo "Starting proxy with debug capture..."
DEBUG_CAPTURE=1 bun --bun run proxy.ts 2>/tmp/mcp-debug.log &
PROXY_PID=$!

echo "Proxy started (PID: $PROXY_PID)"
echo ""
echo "Ready! Now trigger a search in Zed..."
echo "Press ENTER when you're done to stop capture and analyze..."

# Wait for user input
read -r

# Stop proxy
echo ""
echo "Stopping proxy..."
kill $PROXY_PID 2>/dev/null || true
sleep 1

# Analyze captured data
echo ""
echo "=== Capture Complete ==="
echo ""

if [ -f /tmp/mcp-traffic.jsonl ]; then
	LINE_COUNT=$(wc -l </tmp/mcp-traffic.jsonl)
	echo "Captured $LINE_COUNT log entries in /tmp/mcp-traffic.jsonl"

	# Format the JSON for readability
	echo "Formatting JSON..."
	cat /tmp/mcp-traffic.jsonl | jq '.' >/tmp/mcp-traffic-formatted.json

	# Extract specific message types
	cat /tmp/mcp-traffic.jsonl | jq 'select(.direction == "STDIN")' >/tmp/stdin-messages.json
	cat /tmp/mcp-traffic.jsonl | jq 'select(.direction == "SSE_CHUNK")' >/tmp/sse-chunks.json
	cat /tmp/mcp-traffic.jsonl | jq 'select(.direction == "STDOUT")' >/tmp/stdout-messages.json
	cat /tmp/mcp-traffic.jsonl | jq 'select(.direction == "HTTP_REQ")' >/tmp/http-req.json
	cat /tmp/mcp-traffic.jsonl | jq 'select(.direction == "HTTP_RES")' >/tmp/http-res.json

	echo ""
	echo "Extracted files:"
	echo "  - /tmp/mcp-traffic-formatted.json (all traffic, formatted)"
	echo "  - /tmp/stdin-messages.json (Zed → Proxy)"
	echo "  - /tmp/http-req.json (Proxy → Bun Docs)"
	echo "  - /tmp/http-res.json (Bun Docs response headers)"
	echo "  - /tmp/sse-chunks.json (SSE stream chunks)"
	echo "  - /tmp/stdout-messages.json (Proxy → Zed)"
	echo ""

	# Show summary
	echo "=== Traffic Summary ==="
	echo "STDIN messages: $(cat /tmp/stdin-messages.json | jq -s length)"
	echo "HTTP requests: $(cat /tmp/http-req.json | jq -s length)"
	echo "HTTP responses: $(cat /tmp/http-res.json | jq -s length)"
	echo "SSE chunks: $(cat /tmp/sse-chunks.json | jq -s length)"
	echo "STDOUT messages: $(cat /tmp/stdout-messages.json | jq -s length)"
	echo ""

	# Show first STDIN message
	echo "=== First STDIN Message (Zed → Proxy) ==="
	cat /tmp/stdin-messages.json | jq -s '.[0]' 2>/dev/null || echo "No STDIN messages captured"
	echo ""

	# Show first SSE chunk
	echo "=== First SSE Chunk ==="
	cat /tmp/sse-chunks.json | jq -s '.[0]' 2>/dev/null || echo "No SSE chunks captured"

else
	echo "ERROR: No traffic captured. /tmp/mcp-traffic.jsonl not found"
	echo "Check /tmp/mcp-debug.log for errors"
fi

echo ""
echo "Next: Review captured data and create docs/protocol-analysis.md"
