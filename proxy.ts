/**
 * HTTP to stdio proxy for Bun Docs MCP Server
 *
 * This proxy bridges the Bun HTTP MCP server with Zed's stdio-based
 * MCP client. It forwards JSON-RPC messages between stdin/stdout and
 * the HTTP endpoint at https://bun.com/docs/mcp
 *
 * Written in TypeScript using Bun's native APIs for optimal performance.
 */
import { fetch } from "bun";

/** The url of the Bun Docs MCP HTTP server */
const PROTOCOL = "https";
const HOST = "bun.com";
const PORT = 443;
const PATH = "/docs/mcp";

const MCP_SERVER_URL = `${PROTOCOL}://${HOST}:${PORT}${PATH}`;

console.log(`${MCP_SERVER_URL}`);

// Preconnect to MCP server for performance
console.log(`Preconnecting to MCP server at ${MCP_SERVER_URL}...`);
fetch.preconnect(`${PROTOCOL}://${HOST}:${PORT}`);

/**
 * JSON-RPC 2.0 Request types
 */
interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: string | number | null;
  method: string;
  params?: unknown;
}

/**
 * JSON-RPC 2.0 Response type
 */
interface JsonRpcResponse {
  jsonrpc: "2.0";
  id: string | number | null;
  result?: unknown;
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
}

/**
 * Log error message to stderr
 */
function logError(message: string, error?: unknown): void {
  console.error(`[BunDocsMCP] ${message}`, error || "");
}

/**
 * Send JSON-RPC response to stdout
 */
function sendResponse(response: JsonRpcResponse): void {
  console.log(JSON.stringify(response));
}

/**
 * Parse SSE formatted text into JSON-RPC response
 */
function parseSSE(sseText: string): JsonRpcResponse | null {
  const lines = sseText.split("\n");
  for (const line of lines) {
    if (line.startsWith("data: ")) {
      const jsonStr = line.slice(6); // Remove "data: " prefix
      try {
        return JSON.parse(jsonStr) as JsonRpcResponse;
      } catch {}
    }
  }
  return null;
}

/**
 * Forward JSON-RPC request to HTTP server and handle response
 */
async function forwardToHttpServer(request: JsonRpcRequest): Promise<void> {
  try {
    const response = await fetch(MCP_SERVER_URL, {
      signal: AbortSignal.timeout(1000),

      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json, text/event-stream",
      },
      body: JSON.stringify(request),

      // Control automatic response decompression (default: true)
      // Supports gzip, deflate, brotli (br), and zstd
      decompress: true,

      // Disable connection reuse for this request
      keepalive: false,

      // Debug logging level
      verbose: true,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const contentType = response.headers.get("content-type") || "";

    // Handle SSE response
    if (contentType.includes("text/event-stream")) {
      const text = await response.text();
      const data = parseSSE(text);
      if (data) {
        sendResponse(data);
      } else {
        throw new Error("Failed to parse SSE response");
      }
    } else {
      // Handle regular JSON response
      const data = await response.json();
      sendResponse(data as JsonRpcResponse);
    }
  } catch (error) {
    logError("HTTP request failed:", error);
    sendResponse({
      jsonrpc: "2.0",
      id: request.id,
      error: {
        code: -32603,
        message: `Internal error: ${error instanceof Error ? error.message : String(error)}`,
      },
    });
  }
}

// Process incoming JSON-RPC request
async function handleRequest(line: string): Promise<void> {
  if (!line.trim()) {
    return;
  }

  try {
    const request = JSON.parse(line) as JsonRpcRequest;

    // Validate JSON-RPC structure
    if (request.jsonrpc !== "2.0" || !request.method) {
      throw new Error("Invalid JSON-RPC request");
    }

    // Forward all requests to HTTP server
    await forwardToHttpServer(request);
  } catch (error) {
    logError("Failed to parse request:", error);
    sendResponse({
      jsonrpc: "2.0",
      id: null,
      error: {
        code: -32700,
        message: "Parse error",
      },
    });
  }
}

// Main: Read from stdin and process line by line
async function main(): Promise<void> {
  logError("Bun Docs MCP proxy started");

  const stdin = Bun.stdin.stream();
  const reader = stdin.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  try {
    while (true) {
      const { value, done } = await reader.read();

      if (done) {
        break;
      }

      // Decode chunk and append to buffer
      buffer += decoder.decode(value, { stream: true });

      // Process complete lines
      let newlineIndex: number;
      while ((newlineIndex = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, newlineIndex);
        buffer = buffer.slice(newlineIndex + 1);

        await handleRequest(line);
      }
    }
  } catch (error) {
    logError("Fatal error:", error);
    process.exit(1);
  }
}

// Handle process termination
process.on("SIGINT", () => {
  logError("Received SIGINT, shutting down");
  process.exit(0);
});

process.on("SIGTERM", () => {
  logError("Received SIGTERM, shutting down");
  process.exit(0);
});

// Start the proxy
main();
