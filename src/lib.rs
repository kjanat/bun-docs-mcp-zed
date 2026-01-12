use std::fs;
use std::path::PathBuf;
use zed_extension_api as zed;

// ISSUE #8 - System Soft Lock Prevention
//
// This extension uses a simple "download-once" approach to prevent proxy binary
// crashes from causing system soft locks. The extension:
//
// 1. Downloads the latest proxy binary from GitHub Releases on first use
// 2. Reuses the downloaded binary on subsequent calls (no version checking)
// 3. Never executes the binary to check its version (prevents hanging/crashing)
//
// Users receive proxy updates when they update the extension through Zed's normal
// extension update mechanism. This avoids the complexity and risk of auto-updates.
//
// See: https://github.com/kjanat/bun-docs-mcp-zed/issues/8

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

// Platform-specific archive names for binary distribution
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
}

impl BunDocsMcpExtension {
    /// Returns the GitHub release archive name for the current platform.
    ///
    /// Uses `zed::current_platform()` to detect the host OS and architecture,
    /// which is critical when running as WASM in Zed's extension environment.
    ///
    /// # Returns
    /// - `Ok(&str)` - Archive filename for the current platform
    /// - `Err(String)` - Error if platform is not supported
    ///
    /// # Supported Platforms
    /// - Linux: x86_64, aarch64
    /// - macOS: x86_64, aarch64
    /// - Windows: x86_64, aarch64
    fn get_platform_archive_name() -> Result<&'static str, String> {
        // Use zed::current_platform() instead of std::env::consts
        // When running as WASM, std::env::consts::OS returns "wasm32" instead of host OS
        // current_platform() returns a tuple (Os, Architecture)
        let (os, arch) = zed::current_platform();

        match (os, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => Ok(ARCHIVE_LINUX_X64),
            (zed::Os::Linux, zed::Architecture::Aarch64) => Ok(ARCHIVE_LINUX_ARM64),
            (zed::Os::Mac, zed::Architecture::X8664) => Ok(ARCHIVE_MACOS_X64),
            (zed::Os::Mac, zed::Architecture::Aarch64) => Ok(ARCHIVE_MACOS_ARM64),
            (zed::Os::Windows, zed::Architecture::X8664) => Ok(ARCHIVE_WINDOWS_X64),
            (zed::Os::Windows, zed::Architecture::Aarch64) => Ok(ARCHIVE_WINDOWS_ARM64),
            _ => Err(format!(
                "Unsupported platform: {:?} {:?} - please file an issue at https://github.com/kjanat/bun-docs-mcp-zed/issues",
                os,
                arch
            )),
        }
    }

    /// Returns the binary filename for the current platform.
    ///
    /// The binary name differs between Windows (.exe extension) and Unix platforms.
    ///
    /// # Returns
    /// - `"bun-docs-mcp-proxy.exe"` on Windows
    /// - `"bun-docs-mcp-proxy"` on Unix (Linux/macOS)
    fn get_binary_name() -> &'static str {
        // Binary name after extraction
        // Use zed::current_platform() to get actual host OS when running as WASM
        let (os, _) = zed::current_platform();
        if os == zed::Os::Windows {
            "bun-docs-mcp-proxy.exe"
        } else {
            "bun-docs-mcp-proxy"
        }
    }

    /// Ensures the MCP server binary is available, downloading if necessary.
    ///
    /// This function uses a simple "download-once" approach:
    /// 1. Returns cached binary path if available
    /// 2. Checks if binary exists on disk
    /// 3. Downloads from GitHub Releases if binary doesn't exist
    /// 4. Extracts archive and makes binary executable (Unix)
    ///
    /// No version checking or auto-updates are performed to avoid executing
    /// the binary (which could crash or hang). Users receive updates when they
    /// update the extension through Zed.
    ///
    /// # Returns
    /// - `Ok(String)` - Absolute path to the binary
    /// - `Err(String)` - Error if download, extraction, or verification fails
    fn ensure_binary(&mut self) -> Result<String, String> {
        // Return cached path if we've already resolved it
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";
        const PROXY_DIR: &str = "bun-docs-mcp-proxy";

        // Get work directory (where extension runs)
        let work_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get work directory: {}", e))?;

        let binary_name = Self::get_binary_name();

        // Construct binary path using PathBuf for cross-platform compatibility
        let binary_path = PathBuf::from(&work_dir).join(PROXY_DIR).join(binary_name);

        let binary_path_str = binary_path
            .to_str()
            .ok_or_else(|| "Binary path contains invalid UTF-8".to_string())?
            .to_string();

        // Check if binary already exists and is executable
        match fs::metadata(&binary_path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    self.cached_binary_path = Some(binary_path_str.clone());
                    return Ok(binary_path_str);
                } else {
                    return Err(format!(
                        "Binary path exists but is not a file: {}",
                        binary_path_str
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Binary doesn't exist, proceed with download
            }
            Err(e) => {
                return Err(format!(
                    "Failed to check binary at {}: {}",
                    binary_path_str, e
                ));
            }
        }

        // Download from GitHub Releases
        let release = zed::latest_github_release(
            PROXY_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to get latest release from {}: {}", PROXY_REPO, e))?;

        // Find the asset for our platform
        let archive_name = Self::get_platform_archive_name()?;
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == archive_name)
            .ok_or_else(|| {
                format!(
                    "No {} asset found in release {} for {}",
                    archive_name, release.version, PROXY_REPO
                )
            })?;

        // Download and extract the archive
        let file_type = if archive_name.ends_with(".zip") {
            zed::DownloadedFileType::Zip
        } else if archive_name.ends_with(".tar.gz") {
            zed::DownloadedFileType::GzipTar
        } else {
            zed::DownloadedFileType::Uncompressed
        };

        // Download extracts to current directory
        zed::download_file(&asset.download_url, PROXY_DIR, file_type).map_err(|e| {
            format!(
                "Failed to download {} from {}: {}",
                archive_name, asset.download_url, e
            )
        })?;

        // Verify the binary was extracted correctly
        if !binary_path.exists() {
            return Err(format!(
                "Binary not found at expected path after extraction: {}",
                binary_path_str
            ));
        }

        // Make it executable (Unix platforms)
        // Use runtime platform detection since WASM is compiled for wasm32, not the host OS
        let (os, _) = zed::current_platform();
        if os != zed::Os::Windows {
            zed::make_file_executable(&binary_path_str)
                .map_err(|e| format!("Failed to make {} executable: {}", binary_path_str, e))?;
        }

        self.cached_binary_path = Some(binary_path_str.clone());
        Ok(binary_path_str)
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
            CONTEXT_SERVER_ID => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_archive_names() {
        // Platform detection uses zed::current_platform() which can't be mocked in unit tests
        // Platform-specific behavior must be validated by installing as dev extension in Zed

        // Test that expected archive names match supported platforms
        let expected_archives = vec![
            ARCHIVE_LINUX_X64,
            ARCHIVE_LINUX_ARM64,
            ARCHIVE_MACOS_X64,
            ARCHIVE_MACOS_ARM64,
            ARCHIVE_WINDOWS_X64,
            ARCHIVE_WINDOWS_ARM64,
        ];

        // Verify naming patterns are correct
        for archive in expected_archives {
            assert!(
                archive.contains("bun-docs-mcp-proxy-"),
                "Archive name should start with bun-docs-mcp-proxy-"
            );
            assert!(
                archive.ends_with(".tar.gz") || archive.ends_with(".zip"),
                "Archive should have valid extension"
            );
        }
    }

    #[test]
    fn test_binary_names() {
        // Binary name detection uses zed::current_platform() which only works in WASM runtime
        // Cannot be tested in native unit tests - must test via dev extension in Zed
        // This test documents the expected binary names for each platform
        let expected_unix = "bun-docs-mcp-proxy";
        let expected_windows = "bun-docs-mcp-proxy.exe";

        assert!(!expected_unix.is_empty());
        assert!(!expected_windows.is_empty());
        assert!(expected_windows.ends_with(".exe"));
    }

    #[test]
    fn test_binary_path_construction() {
        // Test that PathBuf construction works correctly cross-platform
        // Note: Unit tests run on host platform, but extension runs as WASM
        let work_dir = if cfg!(windows) {
            "C:\\test\\work"
        } else {
            "/test/work"
        };
        let binary_name = "bun-docs-mcp-proxy";

        let path = PathBuf::from(work_dir)
            .join("bun-docs-mcp-proxy")
            .join(binary_name);

        let path_str = path.to_str().unwrap();

        // Verify path contains all expected components
        assert!(path_str.contains("test"));
        assert!(path_str.contains("work"));
        assert!(path_str.contains("bun-docs-mcp-proxy"));

        // Verify path is correctly formed for the platform
        if cfg!(windows) {
            assert_eq!(
                path_str,
                "C:\\test\\work\\bun-docs-mcp-proxy\\bun-docs-mcp-proxy"
            );
        } else {
            assert_eq!(path_str, "/test/work/bun-docs-mcp-proxy/bun-docs-mcp-proxy");
        }
    }

    #[test]
    fn test_context_server_id_constant() {
        // Verify CONTEXT_SERVER_ID matches extension.toml
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
    }

}
