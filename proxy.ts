/**
 * HTTP to stdio proxy for Bun Docs MCP Server
 *
 * This proxy bridges the Bun HTTP MCP server with Zed's stdio-based
 * MCP client. It forwards JSON-RPC messages between stdin/stdout and
 * the HTTP endpoint at https://bun.com/docs/mcp
 *
 * Written in TypeScript using Bun's native APIs for optimal performance.
 */

// Default MCP server URL with env override
const MCP_SERVER_URL: string =
  (typeof process !== "undefined" && process.env?.MCP_SERVER_URL) ||
  "https://bun.com/docs/mcp";

// Allow Bun-specific fetch options without upsetting TypeScript
type ExtendedRequestInit = RequestInit & {
  decompress?: boolean;
  verbose?: boolean;
};

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
 * Forward JSON-RPC request to the HTTP server and handle response
 */
async function forwardToHttpServer(request: JsonRpcRequest): Promise<void> {
  try {
    const init: ExtendedRequestInit = {
      signal: AbortSignal.timeout(1000),
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json, text/event-stream",
      },
      body: JSON.stringify(request),
      // Bun-specific extras (ignored by other runtimes)
      decompress: true,
      verbose: true,
      // Avoid keepalive for simplicity across runtimes
      keepalive: false,
    };

    const response = await fetch(MCP_SERVER_URL, init);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const contentType = response.headers.get("content-type") || "";

    // Handle SSE response
    if (contentType.includes("text/event-stream")) {
      // Stream and parse SSE incrementally
      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error("SSE response has no readable body");
      }

      const decoder = new TextDecoder();
      let buffer = "";
      let eventData = "";

      while (true) {
        const { value, done } = await reader.read();
        if (done) {
          // Flush any pending event
          if (eventData) {
            const data = parseSSE(`data: ${eventData}\n\n`);
            if (data) sendResponse(data);
            eventData = "";
          }
          break;
        }

        buffer += decoder.decode(value, { stream: true });

        let idx: number;
        while ((idx = buffer.indexOf("\n")) !== -1) {
          let line = buffer.slice(0, idx);
          buffer = buffer.slice(idx + 1);

          // Trim trailing CR
          if (line.endsWith("\r")) line = line.slice(0, -1);

          if (line === "") {
            // Dispatch accumulated event
            if (eventData) {
              const json = eventData;
              eventData = "";
              try {
                const parsed = JSON.parse(json) as JsonRpcResponse;
                sendResponse(parsed);
              } catch (e) {
                logError("Failed to parse SSE event JSON:", e);
              }
            }
            continue;
          }

          if (line.startsWith(":")) {
            // Comment line, ignore
            continue;
          }

          if (line.startsWith("data:")) {
            // Append data line (spec allows multiple)
            let payload = line.slice(5); // after 'data:'
            if (payload.startsWith(" ")) payload = payload.slice(1);
            eventData += payload;
            continue;
          }

          // Other SSE fields (event, id, retry) are ignored for now
        }
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

    // Forward all requests to the HTTP server
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

  // If running under Bun, use its streaming stdin API
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const isBun = typeof (globalThis as any).Bun !== "undefined";

  if (isBun) {
    const stdin = (globalThis as any).Bun.stdin.stream();
    const reader = stdin.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      while (true) {
        const { value, done } = await reader.read();

        if (done) {
          // Flush any trailing data
          if (buffer.length) {
            await handleRequest(buffer);
          }
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
    return;
  }

  // Node.js fallback: process stdin in flowing mode
  try {
    let buffer = "";

    // Ensure utf8 strings
    process.stdin.setEncoding("utf8");

    process.stdin.on("data", async (chunk: string) => {
      buffer += chunk;

      let idx: number;
      while ((idx = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, idx);
        buffer = buffer.slice(idx + 1);
        await handleRequest(line);
      }
    });

    process.stdin.on("end", async () => {
      if (buffer.length) {
        await handleRequest(buffer);
        buffer = "";
      }
    });

    process.stdin.on("error", (err) => {
      logError("stdin error:", err);
      process.exit(1);
    });

    // If input is already ended (e.g., no TTY), resume to receive 'end'
    if (process.stdin.readable && (process.stdin as any).isPaused?.()) {
      process.stdin.resume();
    }
  } catch (error) {
    logError("Fatal error:", error);
    process.exit(1);
  }
}

// Handle process termination
process.on("SIGINT", () => {
  logError("\nReceived SIGINT, shutting down");
  process.exit(0);
});

process.on("SIGTERM", () => {
  logError("\nReceived SIGTERM, shutting down");
  process.exit(0);
});

// Start the proxy
void main();
