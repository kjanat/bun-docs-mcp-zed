use zed_extension_api as zed;

struct BunDocsMcpExtension;

impl zed::Extension for BunDocsMcpExtension {
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

impl zed::Extension for BunDocsMcpExtension {
  fn context_server_command(
    &mut self,
    context_server_id: &ContextServerId,
    project: &zed::Project,
  ) -> Result<zed::Command> {
    // Get the path to the proxy script
    let extension_dir =
      std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let proxy_script = extension_dir.join("proxy.ts");

    // Verify proxy script exists
    if !proxy_script.exists() {
      return Err(format!("Proxy script not found at {:?}", proxy_script));
    }

    match context_server_id.as_ref() {
      "bun-docs-mcp" => Ok(zed::Command {
        command: "bun".to_string(),
        args: vec![proxy_script.to_string_lossy().to_string()],
        env: vec![],
      }),
      id => Err(format!("Unknown context server: {}", id)),
    }
  }
}

// zed::register_extension!(BunDocsMcpExtension);
