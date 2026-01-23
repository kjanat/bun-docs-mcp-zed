## Bun Docs MCP Server

This extension provides an MCP (Model Context Protocol) server for searching Bun
documentation directly from Zed's AI assistant.

## IMPORTANT

If you encounter any issues with the MCP server, please use and switch to the official remote server directly.  
Zed started supporting remote MCP servers since [PR #39021], released in Zed [v0.214.5].

Maintaining a proxy is not something I feel like continuing to do, so add this
to your Zed settings file instead:

```jsonc
  "context_servers": {
    "bun-docs": {
      "url": "https://bun.com/docs/mcp",
      "enabled": true,
      "headers": {},
    },
  },
```

[v0.214.5]: https://github.com/zed-industries/zed/releases/tag/v0.214.5 "View Zed v0.214.5 release notes"
[PR #39021]: https://github.com/zed-industries/zed/pull/39021 "View PR #39021"

*It is unfortunately not yet (as of jan 2026) possible for extension authors to provide custom server URLs in the extension manifest.*

<!--markdownlint-disable-file-->
