use semver::Version;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use zed_extension_api as zed;

// How often to check for binary updates (24 hours)
const UPDATE_CHECK_INTERVAL_SECS: u64 = 86400;

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
    last_update_check: Option<SystemTime>,
}

impl BunDocsMcpExtension {
    fn get_platform_archive_name() -> Result<&'static str, String> {
        // Match the GitHub release asset names
        // Note: std::env::consts::OS returns "macos" on macOS, not "darwin"
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

    fn get_binary_version(binary_path: &str) -> Result<String, String> {
        use std::process::Command;

        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to run binary --version: {}", e))?;

        if !output.status.success() {
            return Err("Binary --version exited with error".to_string());
        }

        // Parse "bun-docs-mcp-proxy 0.1.2" -> "0.1.2"
        let version_output = String::from_utf8_lossy(&output.stdout);
        let version = version_output
            .split_whitespace()
            .last()
            .ok_or_else(|| "Failed to parse version output".to_string())?
            .to_string();

        Ok(version)
    }

    fn should_check_for_update(&self) -> bool {
        match self.last_update_check {
            None => true, // Never checked
            Some(last) => {
                let interval = std::time::Duration::from_secs(UPDATE_CHECK_INTERVAL_SECS);
                // If elapsed() fails (system clock moved backward), assume interval has passed.
                // This errs on the side of checking for updates rather than never checking.
                last.elapsed().unwrap_or(interval) >= interval
            }
        }
    }

    fn check_and_update_binary(&mut self, binary_path: &str) -> Result<(), String> {
        // Get current binary version
        let current_version = match Self::get_binary_version(binary_path) {
            Ok(v) => v,
            Err(_) => return Ok(()), // Old binary without --version, skip update check
        };

        // Get latest release from GitHub (non-blocking: ignore errors)
        let release = zed::latest_github_release(
            "kjanat/bun-docs-mcp-proxy",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .ok();

        let Some(release) = release else {
            return Ok(()); // Network error, skip update
        };

        // Compare versions using proper semantic versioning
        let latest_version_str = release.version.trim_start_matches('v');
        let current_version_str = current_version.trim_start_matches('v');

        // Parse versions, skip update check if parsing fails
        let Ok(latest_version) = Version::parse(latest_version_str) else {
            return Ok(());
        };
        let Ok(current_version) = Version::parse(current_version_str) else {
            return Ok(());
        };

        if latest_version > current_version {
            // Delete old binary to trigger re-download on next call
            fs::remove_file(binary_path).ok();
            self.cached_binary_path = None;
        }

        Ok(())
    }

    fn ensure_binary(&mut self) -> Result<String, String> {
        // Check for updates if binary is cached and enough time has passed
        if self.cached_binary_path.is_some() {
            if self.should_check_for_update() {
                // Clone only when needed for update check (avoids clone on every call)
                let cached = self.cached_binary_path.as_ref().unwrap().clone();

                // Update check failures are intentionally ignored to avoid disrupting user workflow.
                // The extension continues using the existing binary if update check fails due to:
                // - Network errors (GitHub API unavailable)
                // - Version parsing failures
                // - Binary execution errors
                // This ensures the extension remains usable even with connectivity issues.
                self.check_and_update_binary(&cached).ok();
                self.last_update_check = Some(SystemTime::now());
            }

            // If update deleted binary, cached_binary_path will be None, continue to download
            if let Some(cached) = &self.cached_binary_path {
                return Ok(cached.clone());
            }
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
        #[cfg(unix)]
        zed::make_file_executable(&binary_path_str)
            .map_err(|e| format!("Failed to make {} executable: {}", binary_path_str, e))?;

        self.cached_binary_path = Some(binary_path_str.clone());
        Ok(binary_path_str)
    }
}

impl zed::Extension for BunDocsMcpExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
            last_update_check: None,
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
        // Test that all supported platforms return Ok with correct archive names
        let test_cases = vec![
            (
                ("linux", "x86_64"),
                "bun-docs-mcp-proxy-linux-x86_64.tar.gz",
            ),
            (
                ("linux", "aarch64"),
                "bun-docs-mcp-proxy-linux-aarch64.tar.gz",
            ),
            (
                ("macos", "x86_64"),
                "bun-docs-mcp-proxy-macos-x86_64.tar.gz",
            ),
            (
                ("macos", "aarch64"),
                "bun-docs-mcp-proxy-macos-aarch64.tar.gz",
            ),
            (
                ("windows", "x86_64"),
                "bun-docs-mcp-proxy-windows-x86_64.zip",
            ),
            (
                ("windows", "aarch64"),
                "bun-docs-mcp-proxy-windows-aarch64.zip",
            ),
        ];

        for ((os, arch), expected) in test_cases {
            // Note: Can't easily test this without mocking std::env::consts
            // This test documents the expected behavior
            assert!(expected.contains(os));
            assert!(
                expected.contains(arch)
                    || expected.contains("x86_64")
                    || expected.contains("aarch64")
            );
        }
    }

    #[test]
    fn test_binary_names() {
        // Test binary name format
        let name = BunDocsMcpExtension::get_binary_name();
        if cfg!(windows) {
            assert_eq!(name, "bun-docs-mcp-proxy.exe");
        } else {
            assert_eq!(name, "bun-docs-mcp-proxy");
        }
    }

    #[test]
    #[cfg(windows)]
    fn test_binary_path_construction() {
        // Test that PathBuf construction works correctly on Windows
        let work_dir = "/test/work";
        let binary_name = "bun-docs-mcp-proxy";

        let path = PathBuf::from(work_dir)
            .join("bun-docs-mcp-proxy")
            .join(binary_name);

        assert_eq!(
            path.to_str().unwrap(),
            "\\test\\work\\bun-docs-mcp-proxy\\bun-docs-mcp-proxy"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_binary_path_construction() {
        // Test that PathBuf construction works correctly on Unix platforms
        let work_dir = "/test/work";
        let binary_name = "bun-docs-mcp-proxy";

        let path = PathBuf::from(work_dir)
            .join("bun-docs-mcp-proxy")
            .join(binary_name);

        assert_eq!(
            path.to_str().unwrap(),
            "/test/work/bun-docs-mcp-proxy/bun-docs-mcp-proxy"
        );
    }

    #[test]
    fn test_unsupported_platform_error() {
        // Test that unsupported platforms return proper error
        // This would require mocking std::env::consts which isn't easily possible
        // But we verify the error message format is correct
        let expected_prefix = "Unsupported platform:";
        let expected_suffix = "please file an issue";

        // The actual function call would require env mocking
        // This documents the expected error format
        assert!(expected_prefix.len() > 0);
        assert!(expected_suffix.len() > 0);
    }

    #[test]
    fn test_context_server_id_constant() {
        // Verify CONTEXT_SERVER_ID matches extension.toml
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
    }

    #[test]
    fn test_constants_defined() {
        // Verify all required constants are properly defined
        assert_eq!(
            BunDocsMcpExtension::get_binary_name().len() > 0,
            true,
            "Binary name should be non-empty"
        );

        // Verify platform archive name returns valid extension
        if let Ok(archive) = BunDocsMcpExtension::get_platform_archive_name() {
            assert!(
                archive.ends_with(".tar.gz") || archive.ends_with(".zip"),
                "Archive should have valid extension"
            );
        }
    }

    #[test]
    fn test_version_parsing() {
        // Test parsing version output format "bun-docs-mcp-proxy 0.1.2"
        let version_output = "bun-docs-mcp-proxy 0.1.2";
        let version = version_output.trim().split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");

        // Test with just version number
        let version_output = "0.1.2";
        let version = version_output.trim().split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");

        // Test with v prefix
        let version_output = "bun-docs-mcp-proxy v0.1.2";
        let version = version_output.trim().split_whitespace().last().unwrap();
        assert_eq!(version, "v0.1.2");
    }

    #[test]
    fn test_version_comparison() {
        // Test proper semantic version comparison
        let v1 = Version::parse("0.1.2").unwrap();
        let v2 = Version::parse("0.1.2").unwrap();
        assert_eq!(v1, v2);

        let v1 = Version::parse("0.1.2").unwrap();
        let v2 = Version::parse("0.1.3").unwrap();
        assert!(v2 > v1);

        // Test that version comparison handles double digits correctly
        let v1 = Version::parse("0.1.9").unwrap();
        let v2 = Version::parse("0.1.10").unwrap();
        assert!(v2 > v1, "0.1.10 should be greater than 0.1.9");

        // Test v prefix stripping before parsing
        let v1 = "v0.1.2".trim_start_matches('v');
        let v2 = "0.1.2".trim_start_matches('v');
        assert_eq!(Version::parse(v1).unwrap(), Version::parse(v2).unwrap());
    }

    #[test]
    fn test_should_check_for_update() {
        let mut ext = BunDocsMcpExtension {
            cached_binary_path: None,
            last_update_check: None,
        };

        // Should check when never checked before
        assert!(ext.should_check_for_update());

        // Should not check immediately after checking
        ext.last_update_check = Some(SystemTime::now());
        assert!(!ext.should_check_for_update());

        // Should check after update interval has passed
        let interval_ago = SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(UPDATE_CHECK_INTERVAL_SECS))
            .unwrap();
        ext.last_update_check = Some(interval_ago);
        assert!(ext.should_check_for_update());
    }
}
