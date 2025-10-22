use zed_extension_api as zed;
use std::fs;

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
        // Get the path to the proxy script
        let extension_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let proxy_script = extension_dir.join("proxy.js");

        // Verify proxy script exists
        if !proxy_script.exists() {
            return Err(format!("Proxy script not found at {:?}", proxy_script));
        }

        match context_server_id.as_ref() {
            "bun-docs" => {
                Ok(zed::Command {
                    command: "node".to_string(),
                    args: vec![proxy_script.to_string_lossy().to_string()],
                    env: vec![],
                })
            }
            id => Err(format!("Unknown context server: {}", id)),
        }
    }
}

zed::register_extension!(BunDocsMcpExtension);
