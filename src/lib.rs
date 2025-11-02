use zed_extension_api as zed;

struct BunDocsMcpExtension;

impl zed::Extension for BunDocsMcpExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command, String> {
        // Get the path to the proxy scripts in the extension directory
        let extension_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get the current directory: {}", e))?;

        let proxy_ts = extension_dir.join("proxy.ts");
        let proxy_js = extension_dir.join("proxy.js");

        match context_server_id.as_ref() {
            "bun-docs-mcp" => {
                // Prefer `Node` + built `JS` if available (doesn't require `Bun` at runtime)
                if proxy_js.exists() {
                    Ok(zed::Command {
                        command: "node".to_string(),
                        args: vec![proxy_js.to_string_lossy().to_string()],
                        env: vec![],
                    })
                } else if proxy_ts.exists() {
                    // Fallback to `Bun` + `TypeScript` source
                    Ok(zed::Command {
                        command: "bun".to_string(),
                        args: vec![proxy_ts.to_string_lossy().to_string()],
                        env: vec![],
                    })
                } else {
                    Err(format!(
                        "Proxy wasn't found: expected {:?} or {:?}",
                        proxy_js, proxy_ts
                    ))
                }
            }
            id => Err(format!("Unknown context server: {}", id)),
        }
    }
}

zed::register_extension!(BunDocsMcpExtension);
