# Bun Docs MCP Extension for Zed

Repository: https://github.com/kjanat/bun-docs-mcp-zed

## Overview

Zed extension that provides MCP (Model Context Protocol) integration for searching Bun documentation.

**Implementation**: Pure Rust WASM extension that auto-downloads a native Rust proxy binary from GitHub Releases.

## Current Architecture

```
Zed (stdio) ← Extension WASM → Rust Binary (stdio) → HTTPS → bun.com/docs/mcp
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete technical details.

## Development

```sh
# Build extension WASM
cargo build --target wasm32-wasip2 --release

# Install as dev extension in Zed
# Cmd/Ctrl+Shift+P → "zed: install dev extension"
```

## Publishing Checklist

1. Update extension.toml metadata
2. Fork `https://github.com/zed-industries/extensions`
3. Add submodule: `git submodule add https://github.com/kjanat/bun-docs-mcp-zed extensions/bun-docs-mcp`
4. Update `extensions/extensions.toml`:
   ```toml
   [bun-docs-mcp]
   submodule = "extensions/bun-docs-mcp"
   version = "0.1.0-alpha"
   ```
5. Run `pnpm sort-extensions`
6. Open PR

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
version = "0.0.2"
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
```

## WebAssembly

Procedural parts of extensions are written in Rust and compiled to WebAssembly. To develop an extension that includes custom code, include a `Cargo.toml` like this:

```toml
[package]
name = "bun-docs-mcp-zed"
version = "0.1.0"
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
    version = "0.0.2"
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

If you need to download the MCP server from an external source—like GitHub Releases—you can also do that in this function.

## Available Extensions

Check out all the MCP servers that have already been exposed as extensions [on Zed's site](https://zed.dev/extensions?filter=context-servers).

We recommend taking a look at their repositories as a way to understand how they are generally created and structured.

## Testing

To test your new MCP server extension, you can [install it as a dev extension](./developing-extensions.md#developing-an-extension-locally).

</details>
