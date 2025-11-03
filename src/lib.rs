use std::fs;
use zed_extension_api as zed;

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
}

impl BunDocsMcpExtension {
    fn get_platform_archive_name() -> Result<&'static str, String> {
        // Match the GitHub release asset names
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Ok("bun-docs-mcp-proxy-linux-x86_64.tar.gz"),
            ("linux", "aarch64") => Ok("bun-docs-mcp-proxy-linux-aarch64.tar.gz"),
            ("macos", "x86_64") => Ok("bun-docs-mcp-proxy-macos-x86_64.tar.gz"),
            ("macos", "aarch64") => Ok("bun-docs-mcp-proxy-macos-aarch64.tar.gz"),
            ("windows", "x86_64") => Ok("bun-docs-mcp-proxy-windows-x86_64.zip"),
            ("windows", "aarch64") => Ok("bun-docs-mcp-proxy-windows-aarch64.zip"),
            _ => Err(format!(
                "Unsupported platform: {} {} - please file an issue at https://github.com/kjanat/bun-docs-mcp-zed/issues",
                std::env::consts::OS,
                std::env::consts::ARCH
            )),
        }
    }

    fn get_binary_name() -> &'static str {
        // Binary name after extraction
        if cfg!(windows) {
            "bun-docs-mcp-proxy.exe"
        } else {
            "bun-docs-mcp-proxy"
        }
    }

    fn ensure_binary(&mut self) -> Result<String, String> {
        // Return cached path if we already downloaded it
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        // Get work directory (where extension runs)
        let work_dir = std::env::var("PWD")
            .or_else(|_| std::env::current_dir().map(|p| p.to_string_lossy().to_string()))
            .map_err(|e| format!("Failed to get work directory: {}", e))?;

        let binary_name = Self::get_binary_name();

        // After extraction, the binary is in a subdirectory
        // The tar.gz extracts to: bun-docs-mcp-proxy/bun-docs-mcp-proxy
        let binary_path = format!("{}/bun-docs-mcp-proxy/{}", work_dir, binary_name);

        // Check if binary already exists
        if fs::metadata(&binary_path).is_ok() {
            self.cached_binary_path = Some(binary_path.clone());
            return Ok(binary_path);
        }

        // Download from GitHub Releases
        // The proxy is in a separate repo: kjanat/bun-docs-mcp-proxy
        let repo = "kjanat/bun-docs-mcp-proxy";

        let release = zed::latest_github_release(
            repo,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to get latest release: {}", e))?;

        // Find the asset for our platform
        let archive_name = Self::get_platform_archive_name()?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == archive_name)
            .ok_or_else(|| format!("No archive found for platform: {}", archive_name))?;

        // Download and extract the archive
        let file_type = if archive_name.ends_with(".zip") {
            zed::DownloadedFileType::Zip
        } else if archive_name.ends_with(".tar.gz") {
            zed::DownloadedFileType::GzipTar
        } else {
            zed::DownloadedFileType::Uncompressed
        };

        zed::download_file(&asset.download_url, binary_name, file_type)
            .map_err(|e| format!("Failed to download binary: {}", e))?;

        // Make it executable (Unix platforms)
        #[cfg(unix)]
        zed::make_file_executable(&binary_path)
            .map_err(|e| format!("Failed to make {} executable: {}", binary_path, e))?;

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for BunDocsMcpExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command, String> {
        match context_server_id.as_ref() {
            "bun-docs-mcp" => {
                let binary_path = self.ensure_binary()?;

                Ok(zed::Command {
                    command: binary_path,
                    args: vec![],
                    env: vec![],
                })
            }
            id => Err(format!("Unknown context server: {}", id)),
        }
    }
}

zed::register_extension!(BunDocsMcpExtension);
