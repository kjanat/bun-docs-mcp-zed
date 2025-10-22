# github: https://github.com/kjanat/bun-docs-mcp-zed.git

**Final Structure**

```sh
bun-docs-mcp-zed/
├── extension.toml   # Registers [context_servers.bun-docs-mcp]
├── Cargo.toml       # WASM build config
├── src/lib.rs       # Implements context_server_command
├── proxy.js         # HTTP ←→ stdio bridge
├── LICENSE          # MIT
└── README.md        # Complete docs
```

Key Changes from Initial Implementation

Before (Incorrect):

- Used non-existent mcp_server_config method
- Tried to configure HTTP directly in Rust

After (Correct):

- [x] Registered context server in extension.toml:9
- [x] Implemented context_server_command in src/lib.rs:11
- [x] Created proxy.js to bridge stdio ←→ HTTP transport
- [x] Returns Node.js command to run proxy

Architecture Flow

```text
Zed (stdio) → proxy.js → HTTPS POST    → bun.com/docs/mcp
Zed (stdio) ← proxy.js ← HTTP Response ← bun.com/docs/mcp
```

## Testing

```sh
cd bun-docs-mcp-zed

# Build WASM

cargo build --target wasm32-wasi --release

# Test proxy

echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | node proxy.js

# Install in Zed
# Extensions → Install Dev Extension → select bun-docs-mcp-zed/
```

## Publishing Checklist

1. Update extension.toml:5 → your name/email
2. Update extension.toml:7 → your repo URL
3. Fork https://github.com/zed-industries/extensions
4. Add submodule: git submodule add https://github.com/kjanat/bun-docs-mcp-zed extensions/bun-docs-mcp
5. Update extensions.toml:

   ```toml
   [bun-docs-mcp]
   submodule = "extensions/bun-docs-mcp"
   version = "0.0.1"
   ```

6. Run pnpm sort-extensions
7. Open PR

Extension now correctly implements MCP context server pattern for Zed!

---

<details><summary>Zed - Developing Extensions</summary>

# Developing Extensions

## Extension Capabilities

Extensions can add the following capabilities to Zed:

- [Languages](./languages.md)
- [Debuggers](./debugger-extensions.md)
- [Themes](./themes.md)
- [Icon Themes](./icon-themes.md)
- [Slash Commands](./slash-commands.md)
- [MCP Servers](./mcp-extensions.md)

## Developing an Extension Locally

Before starting to develop an extension for Zed, be sure to [install Rust via rustup](https://www.rust-lang.org/tools/install).

> Rust must be installed via rustup. If you have Rust installed via homebrew or otherwise, installing dev extensions will not work.

When developing an extension, you can use it in Zed without needing to publish it by installing it as a _dev extension_.

From the extensions page, click the `Install Dev Extension` button (or the `zed::InstallDevExtension` action) and select the directory containing your extension.

If you need to troubleshoot, you can check the Zed.log (`zed::OpenLog`) for additional output. For debug output, close and relaunch zed with the `zed --foreground` from the command line which show more verbose INFO evel logging.

If you already have the published version of the extension installed, the published version will be uninstalled prior to the installation of the dev extension. After successful installation, the `Extensions` page will indicate hat the upstream extension is "Overridden by dev extension".

## Directory Structure of a Zed Extension

A Zed extension is a Git repository that contains an `extension.toml`. This file must contain some
basic information about the extension:

```toml
id = "bun-docs-mcp"
name = "Bun Docs MCP"
version = "0.0.1"
schema_version = 1
authors = ["Kaj Kowalski <dev@kajkowalski.nl>"]
description = "MCP server integration for searching Bun documentation directly in Zed"
repository = "https://github.com/kjanat/bun-docs-mcp-zed"
```

In addition to this, there are several other optional files and directories that can be used to add functionality to a Zed extension. An example directory structure of an extension that provides all capabilities is as follows:

```sh
bun-docs-mcp-zed/
  extension.toml
  Cargo.toml
  src/
    lib.rs
  # languages/
  #   my-language/
  #     config.toml
  #     highlights.scm
  # themes/
  #   my-theme.json
```

## WebAssembly

Procedural parts of extensions are written in Rust and compiled to WebAssembly. To develop an extension that includes custom code, include a `Cargo.toml` like this:

```toml
[package]
name = "bun-docs-mcp-zed"
version = "0.0.1"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
zed_extension_api = "0.1.0"
```

Use the latest version of the [`zed_extension_api`](https://crates.io/crates/zed_extension_api) available on crates.io. Make sure it's still [compatible with Zed versions]https://github.com/zed-industries/zed/blob/main/crates/extension_api#compatible-zed-versions) you want to support.

In the `src/lib.rs` file in your Rust crate you will need to define a struct for your extension and implement the `Extension` trait, as well as use the `register_extension!` macro to register your extension:

```rs
use zed_extension_api as zed;

struct MyExtension {
    // ... state
}

impl zed::Extension for MyExtension {
    // ...
}

zed::register_extension!(MyExtension);
```

> `stdout`/`stderr` is forwarded directly to the Zed process. In order to see `println!`/`dbg!` output from your extension, you can start Zed in your terminal with a `--foreground` flag.

## Forking and cloning the repo

1. Fork the repo

> Note: It is very helpful if you fork the `zed-industries/extensions` repo to a personal GitHub account instead of a GitHub organization, as this allows Zed staff to push any needed changes to your PR to expedite the ublishing process.

2. Clone the repo to your local machine

```sh
# Substitute the url of your fork here:
# git clone https://github.com/zed-industries/extensions
cd extensions
git submodule init
git submodule update
```

## Extension License Requirements

As of October 1st, 2025, extension repositories must include one of the following licenses:

- [MIT](https://opensource.org/license/mit)
- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)

This allows us to distribute the resulting binary produced from your extension code to our users.
Without a valid license, the pull request to add or update your extension in the following steps will fail CI.

Your license file should be at the root of your extension repository. Any filename that has `LICENCE` or `LICENSE` as a prefix (case insensitive) will be inspected to ensure it matches one of the accepted licenses. See the [license validation source code](https://github.com/zed-industries/extensions/blob/main/src/lib/license.js).

> This license requirement applies only to your extension code itself (the code that gets compiled into the extension binary).
> It does not apply to any tools your extension may download or interact with, such as language servers or other external dependencies.
> If your repository contains both extension code and other projects (like a language server), you are not required to relicense those other projects—only the extension code needs to be one of the aforementioned accepted icenses.

## Publishing your extension

To publish an extension, open a PR to [the `zed-industries/extensions` repo](https://github.com/zed-industries/extensions).

In your PR, do the following:

1. Add your extension as a Git submodule within the `extensions/` directory

```sh
git submodule add https://github.com/kjanat/bun-docs-mcp-zed.git extensions/bun-docs-mcp
git add extensions/bun-docs-mcp
```

> All extension submodules must use HTTPS URLs and not SSH URLS (`git@github.com`).

2. Add a new entry to the top-level `extensions.toml` file containing your extension:

```toml
[bun-docs-mcp]
submodule = "extensions/bun-docs-mcp"
version = "0.0.1"
```

> If your extension is in a subdirectory within the submodule you can use the `path` field to point to where the extension resides.

3. Run `pnpm sort-extensions` to ensure `extensions.toml` and `.gitmodules` are sorted

Once your PR is merged, the extension will be packaged and published to the Zed extension registry.

> Extension IDs and names should not contain `zed` or `Zed`, since they are all Zed extensions.

## Updating an extension

To update an extension, open a PR to [the `zed-industries/extensions` repo](https://github.com/zed-industries/extensions).

In your PR do the following:

1. Update the extension's submodule to the commit of the new version.
2. Update the `version` field for the extension in `extensions.toml`
   - Make sure the `version` matches the one set in `extension.toml` at the particular commit.

If you'd like to automate this process, there is a [community GitHub Action](https://github.com/huacnlee/zed-extension-action) you can use.

> **Note:** If your extension repository has a different license, you'll need to update it to be one of the [accepted extension licenses](#extension-license-requirements) before publishing your update.

</details>

---

<details><summary>Zed - MCP Server Extensions</summary>

# MCP Server Extensions

[Model Context Protocol servers](https://zed.dev/docs/ai/mcp.html) can be exposed as extensions for use in the Agent Panel.

## Defining MCP Extensions

A given extension may provide one or more MCP servers.
Each MCP server must be registered in the `extension.toml`:

```toml
[context_servers.bun-docs-mcp]
```

Then, in the Rust code for your extension, implement the `context_server_command` method on your extension:

```rust
impl zed::Extension for MyExtension {
    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &zed::Project,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: get_path_to_context_server_executable()?,
            args: get_args_for_context_server()?,
            env: get_env_for_context_server()?,
        })
    }
}
```

This method should return the command to start up an MCP server, along with any arguments or environment variables necessary for it to function.

If you need to download the MCP server from an external source—like GitHub Releases or npm—you can also do that in this function.

## Available Extensions

Check out all the MCP servers that have already been exposed as extensions [on Zed's site](https://zed.dev/extensions?filter=context-servers).

We recommend taking a look at their repositories as a way to understand how they are generally created and structured.

## Testing

To test your new MCP server extension, you can [install it as a dev extension](./developing-extensions.md#developing-an-extension-locally).

</details>

---

<details><summary>Bun - Server</summary>

# Server

> Use `Bun.serve` to start a high-performance HTTP server in Bun

## Basic Setup

```ts title="index.ts" icon="/icons/typescript.svg" theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  // `routes` requires Bun v1.2.3+
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        const body = await req.json();
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file by buffering it in memory
    "/favicon.ico": new Response(await Bun.file("./favicon.ico").bytes(), {
      headers: {
        "Content-Type": "image/x-icon",
      },
    }),
  },

  // (optional) fallback for unmatched routes:
  // Required if Bun's version < 1.2.3
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## HTML imports

Bun supports importing HTML files directly into your server code, enabling full-stack applications with both server-side and client-side code. HTML imports work in two modes:

without a full page reload.

this manifest to serve optimized assets with zero runtime bundling overhead. This is ideal for deploying to production.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
import myReactSinglePageApp from "./index.html";

Bun.serve({
  routes: {
    "/": myReactSinglePageApp,
  },
});
```

with React, TypeScript, Tailwind CSS, and more.

For a complete guide on building full-stack applications with HTML imports, including detailed examples and best practices, see [/docs/bundler/fullstack](/bundler/fullstack).

---

## Configuration

### Changing the `port` and `hostname`

To configure which port and hostname the server will listen on, set `port` and `hostname` in the options object.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
Bun.serve({
  port: 8080, // defaults to $BUN_PORT, $PORT, $NODE_PORT otherwise 3000 // [!code ++]
  hostname: "mydomain.com", // defaults to "0.0.0.0" // [!code ++]
  fetch(req) {
    return new Response("404!");
  },
});
```

To randomly select an available port, set `port` to `0`.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  port: 0, // random port // [!code ++]
  fetch(req) {
    return new Response("404!");
  },
});

// server.port is the randomly selected port
console.log(server.port);
```

You can view the chosen port by accessing the `port` property on the server object, or by accessing the `url` property.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
console.log(server.port); // 3000
console.log(server.url); // http://localhost:3000
```

### Configuring a default port

Bun supports several options and environment variables to configure the default port. The default port is used when the `port` option is not set.

- `--port` CLI flag

```sh theme={"theme":{"light":"github-light","dark":"dracula"}}
bun --port=4002 server.ts
```

- `BUN_PORT` environment variable

```sh theme={"theme":{"light":"github-light","dark":"dracula"}}
bun_PORT=4002 bun server.ts
```

- `PORT` environment variable

```sh terminal icon="terminal" theme={"theme":{"light":"github-light","dark":"dracula"}}
PORT=4002 bun server.ts
```

- `NODE_PORT` environment variable

```sh terminal icon="terminal" theme={"theme":{"light":"github-light","dark":"dracula"}}
NODE_PORT=4002 bun server.ts
```

---

## Unix domain sockets

To listen on a [unix domain socket](https://en.wikipedia.org/wiki/Unix_domain_socket), pass the `unix` option with the path to the socket.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
Bun.serve({
  unix: "/tmp/my-socket.sock", // path to socket
  fetch(req) {
    return new Response(`404!`);
  },
});
```

### Abstract namespace sockets

Bun supports Linux abstract namespace sockets. To use an abstract namespace socket, prefix the `unix` path with a null byte.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
Bun.serve({
  unix: "\0my-abstract-socket", // abstract namespace socket
  fetch(req) {
    return new Response(`404!`);
  },
});
```

Unlike unix domain sockets, abstract namespace sockets are not bound to the filesystem and are automatically removed when the last reference to the socket is closed.

---

## idleTimeout

To configure the idle timeout, set the `idleTimeout` field in Bun.serve.

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
Bun.serve({
  // 10 seconds:
  idleTimeout: 10,

  fetch(req) {
    return new Response("Bun!");
  },
});
```

This is the maximum amount of time a connection is allowed to be idle before the server closes it. A connection is idling if there is no data sent or received.

---

## export default syntax

Thus far, the examples on this page have used the explicit `Bun.serve` API. Bun also supports an alternate syntax.

```ts server.ts theme={"theme":{"light":"github-light","dark":"dracula"}}
import { type Serve } from "bun";

export default {
  fetch(req) {
    return new Response("Bun!");
  },
} satisfies Serve;
```

hood.

---

## Hot Route Reloading

Update routes without server restarts using `server.reload()`:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  routes: {
    "/api/version": () => Response.json({ version: "1.0.0" }),
  },
});

// Deploy new routes without downtime
server.reload({
  routes: {
    "/api/version": () => Response.json({ version: "2.0.0" }),
  },
});
```

---

## Server Lifecycle Methods

### `server.stop()`

To stop the server from accepting new connections:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  fetch(req) {
    return new Response("Hello!");
  },
});

// Gracefully stop the server (waits for in-flight requests)
await server.stop();

// Force stop and close all active connections
await server.stop(true);
```

By default, `stop()` allows in-flight requests and WebSocket connections to complete. Pass `true` to immediately terminate all connections.

### `server.ref()` and `server.unref()`

Control whether the server keeps the Bun process alive:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
// Don't keep process alive if server is the only thing running
server.unref();

// Restore default behavior - keep process alive
server.ref();
```

### `server.reload()`

Update the server's handlers without restarting:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  routes: {
    "/api/version": Response.json({ version: "v1" }),
  },
  fetch(req) {
    return new Response("v1");
  },
});

// Update to new handler
server.reload({
  routes: {
    "/api/version": Response.json({ version: "v2" }),
  },
  fetch(req) {
    return new Response("v2");
  },
});
```

This is useful for development and hot reloading. Only `fetch`, `error`, and `routes` can be updated.

---

## Per-Request Controls

### `server.timeout(Request, seconds)`

Set a custom idle timeout for individual requests:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  fetch(req, server) {
    // Set 60 second timeout for this request
    server.timeout(req, 60);

    // If they take longer than 60 seconds to send the body, the request will be aborted
    await req.text();

    return new Response("Done!");
  },
});
```

Pass `0` to disable the timeout for a request.

### `server.requestIP(Request)`

Get client IP and port information:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  fetch(req, server) {
    const address = server.requestIP(req);
    if (address) {
      return new Response(
        `Client IP: ${address.address}, Port: ${address.port}`,
      );
    }
    return new Response("Unknown client");
  },
});
```

Returns `null` for closed requests or Unix domain sockets.

---

## Server Metrics

### `server.pendingRequests` and `server.pendingWebSockets`

Monitor server activity with built-in counters:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  fetch(req, server) {
    return new Response(
      `Active requests: ${server.pendingRequests}\n` +
        `Active WebSockets: ${server.pendingWebSockets}`,
    );
  },
});
```

### `server.subscriberCount(topic)`

Get count of subscribers for a WebSocket topic:

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
const server = Bun.serve({
  fetch(req, server) {
    const chatUsers = server.subscriberCount("chat");
    return new Response(`${chatUsers} users in chat`);
  },
  websocket: {
    message(ws) {
      ws.subscribe("chat");
    },
  },
});
```

---

## Benchmarks

Below are Bun and Node.js implementations of a simple HTTP server that responds `Bun!` to each incoming `Request`.

```ts Bun theme={"theme":{"light":"github-light","dark":"dracula"}}
Bun.serve({
  fetch(req: Request) {
    return new Response("Bun!");
  },
  port: 3000,
});
```

```ts theme={"theme":{"light":"github-light","dark":"dracula"}}
require("http")
  .createServer((req, res) => res.end("Bun!"))
  .listen(8080);
```

The `Bun.serve` server can handle roughly 2.5x more requests per second than Node.js on Linux.

| Runtime | Requests per second |
| ------- | ------------------- |
| Node 16 | \~64,000            |
| Bun     | \~160,000           |

<Frame>
  ![image](https://user-images.githubusercontent.com/709451/162389032-fc302444-9d03-46be-ba87-c12bd8ce89a0.png)
</Frame>

---

## Practical example: REST API

Here's a basic database-backed REST API using Bun's router with zero dependencies:

<CodeGroup>
  ```ts server.ts expandable icon="file-code" theme={"theme":{"light":"github-light","dark":"dracula"}}
  import type {Post} from './types.ts';
  import {Database} from 'bun:sqlite';

const db = new Database('posts.db');
db.exec(`     CREATE TABLE IF NOT EXISTS posts (
      id TEXT PRIMARY KEY,
      title TEXT NOT NULL,
      content TEXT NOT NULL,
      created_at TEXT NOT NULL
    )
  `);

Bun.serve({
routes: {
// List posts
'/api/posts': {
GET: () => {
const posts = db.query('SELECT \* FROM posts').all();
return Response.json(posts);
},

// Create post
POST: async req => {
const post: Omit<Post, 'id' | 'created_at'> = await req.json();
const id = crypto.randomUUID();

db.query(
`INSERT INTO posts (id, title, content, created_at)
             VALUES (?, ?, ?, ?)`,
).run(id, post.title, post.content, new Date().toISOString());

return Response.json({id, ...post}, {status: 201});
},
},

// Get post by ID
'/api/posts/:id': req => {
const post = db.query('SELECT \* FROM posts WHERE id = ?').get(req.params.id);

if (!post) {
return new Response('Not Found', {status: 404});
}

return Response.json(post);
},
},

error(error) {
console.error(error);
return new Response('Internal Server Error', {status: 500});
},
});

````

```ts types.ts icon="/icons/typescript.svg" theme={"theme":{"light":"github-light","dark":"dracula"}}
export interface Post {
	id: string;
	title: string;
	content: string;
	created_at: string;
}
````

</CodeGroup>

---

## Reference

```ts expandable See TypeScript Definitions theme={"theme":{"light":"github-light","dark":"dracula"}}
interface Server extends Disposable {
  /**
   * Stop the server from accepting new connections.
   * @param closeActiveConnections If true, immediately terminates all connections
   * @returns Promise that resolves when the server has stopped
   */
  stop(closeActiveConnections?: boolean): Promise<void>;

  /**
   * Update handlers without restarting the server.
   * Only fetch and error handlers can be updated.
   */
  reload(options: Serve): void;

  /**
   * Make a request to the running server.
   * Useful for testing or internal routing.
   */
  fetch(request: Request | string): Response | Promise<Response>;

  /**
   * Upgrade an HTTP request to a WebSocket connection.
   * @returns true if upgrade successful, false if failed
   */
  upgrade<T = undefined>(
    request: Request,
    options?: {
      headers?: Bun.HeadersInit;
      data?: T;
    },
  ): boolean;

  /**
   * Publish a message to all WebSocket clients subscribed to a topic.
   * @returns Bytes sent, 0 if dropped, -1 if backpressure applied
   */
  publish(
    topic: string,
    data: string | ArrayBufferView | ArrayBuffer | SharedArrayBuffer,
    compress?: boolean,
  ): ServerWebSocketSendStatus;

  /**
   * Get count of WebSocket clients subscribed to a topic.
   */
  subscriberCount(topic: string): number;

  /**
   * Get client IP address and port.
   * @returns null for closed requests or Unix sockets
   */
  requestIP(request: Request): SocketAddress | null;

  /**
   * Set custom idle timeout for a request.
   * @param seconds Timeout in seconds, 0 to disable
   */
  timeout(request: Request, seconds: number): void;

  /**
   * Keep process alive while server is running.
   */
  ref(): void;

  /**
   * Allow process to exit if server is only thing running.
   */
  unref(): void;

  /** Number of in-flight HTTP requests */
  readonly pendingRequests: number;

  /** Number of active WebSocket connections */
  readonly pendingWebSockets: number;

  /** Server URL including protocol, hostname and port */
  readonly url: URL;

  /** Port server is listening on */
  readonly port: number;

  /** Hostname server is bound to */
  readonly hostname: string;

  /** Whether server is in development mode */
  readonly development: boolean;

  /** Server instance identifier */
  readonly id: string;
}

interface WebSocketHandler<T = undefined> {
  /** Maximum WebSocket message size in bytes */
  maxPayloadLength?: number;

  /** Bytes of queued messages before applying backpressure */
  backpressureLimit?: number;

  /** Whether to close connection when backpressure limit hit */
  closeOnBackpressureLimit?: boolean;

  /** Called when backpressure is relieved */
  drain?(ws: ServerWebSocket<T>): void | Promise<void>;

  /** Seconds before idle timeout */
  idleTimeout?: number;

  /** Enable per-message deflate compression */
  perMessageDeflate?:
    | boolean
    | {
        compress?: WebSocketCompressor | boolean;
        decompress?: WebSocketCompressor | boolean;
      };

  /** Send ping frames to keep connection alive */
  sendPings?: boolean;

  /** Whether server receives its own published messages */
  publishToSelf?: boolean;

  /** Called when connection opened */
  open?(ws: ServerWebSocket<T>): void | Promise<void>;

  /** Called when message received */
  message(
    ws: ServerWebSocket<T>,
    message: string | Buffer,
  ): void | Promise<void>;

  /** Called when connection closed */
  close?(
    ws: ServerWebSocket<T>,
    code: number,
    reason: string,
  ): void | Promise<void>;

  /** Called when ping frame received */
  ping?(ws: ServerWebSocket<T>, data: Buffer): void | Promise<void>;

  /** Called when pong frame received */
  pong?(ws: ServerWebSocket<T>, data: Buffer): void | Promise<void>;
}

interface TLSOptions {
  /** Certificate authority chain */
  ca?: string | Buffer | BunFile | Array<string | Buffer | BunFile>;

  /** Server certificate */
  cert?: string | Buffer | BunFile | Array<string | Buffer | BunFile>;

  /** Path to DH parameters file */
  dhParamsFile?: string;

  /** Private key */
  key?: string | Buffer | BunFile | Array<string | Buffer | BunFile>;

  /** Reduce TLS memory usage */
  lowMemoryMode?: boolean;

  /** Private key passphrase */
  passphrase?: string;

  /** OpenSSL options flags */
  secureOptions?: number;

  /** Server name for SNI */
  serverName?: string;
}
```

</details>
