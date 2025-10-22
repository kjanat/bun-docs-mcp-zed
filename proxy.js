#!/usr/bin/env node

/**
 * HTTP to stdio proxy for Bun Docs MCP Server
 *
 * This proxy bridges the Bun HTTP MCP server with Zed's stdio-based
 * MCP client. It forwards JSON-RPC messages between stdin/stdout and
 * the HTTP endpoint at https://bun.com/docs/mcp
 */

const https = require('https');
const readline = require('readline');

const MCP_SERVER_URL = 'https://bun.com/docs/mcp';

// Create readline interface for stdin
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

// Log errors to stderr
function logError(message, error) {
  console.error(`[BunDocsMCP] ${message}`, error || '');
}

// Send response to stdout
function sendResponse(response) {
  console.log(JSON.stringify(response));
}

// Forward request to HTTP MCP server
function forwardToHttpServer(request) {
  const requestData = JSON.stringify(request);

  const options = {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Content-Length': Buffer.byteLength(requestData)
    }
  };

  const req = https.request(MCP_SERVER_URL, options, (res) => {
    let responseData = '';

    res.on('data', (chunk) => {
      responseData += chunk;
    });

    res.on('end', () => {
      try {
        const response = JSON.parse(responseData);
        sendResponse(response);
      } catch (error) {
        logError('Failed to parse response from HTTP server:', error);
        sendResponse({
          jsonrpc: '2.0',
          id: request.id,
          error: {
            code: -32603,
            message: 'Internal error: Failed to parse server response'
          }
        });
      }
    });
  });

  req.on('error', (error) => {
    logError('HTTP request failed:', error);
    sendResponse({
      jsonrpc: '2.0',
      id: request.id,
      error: {
        code: -32603,
        message: `Internal error: ${error.message}`
      }
    });
  });

  req.write(requestData);
  req.end();
}

// Handle initialization
function handleInitialize(request) {
  // Forward initialize request to HTTP server to get capabilities
  forwardToHttpServer(request);
}

// Handle incoming messages from stdin
rl.on('line', (line) => {
  if (!line.trim()) {
    return;
  }

  try {
    const request = JSON.parse(line);

    // Handle different request types
    if (request.method === 'initialize') {
      handleInitialize(request);
    } else {
      // Forward all other requests to HTTP server
      forwardToHttpServer(request);
    }
  } catch (error) {
    logError('Failed to parse request:', error);
    sendResponse({
      jsonrpc: '2.0',
      id: null,
      error: {
        code: -32700,
        message: 'Parse error'
      }
    });
  }
});

// Handle process termination
process.on('SIGINT', () => {
  process.exit(0);
});

process.on('SIGTERM', () => {
  process.exit(0);
});

// Signal ready
logError('Bun Docs MCP proxy started');
