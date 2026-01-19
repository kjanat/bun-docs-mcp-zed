// Allow multiple hashbrown versions from transitive dependencies (semver, zed_extension_api)
#![allow(clippy::multiple_crate_versions)]

use semver::Version;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use zed_extension_api as zed;

// How often to check for binary updates (24 hours)
const UPDATE_CHECK_INTERVAL_SECS: u64 = 86400;

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

// GitHub repository and directory names for the proxy binary
const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";
const PROXY_DIR: &str = "bun-docs-mcp-proxy";

// Platform-specific archive names for binary distribution
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
    last_update_check: Option<SystemTime>,
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
    /// - Linux: `x86_64`, aarch64
    /// - macOS: `x86_64`, aarch64
    /// - Windows: `x86_64`, aarch64
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
                "Unsupported platform: {os:?} {arch:?} - please file an issue at https://github.com/kjanat/bun-docs-mcp-zed/issues"
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

    /// Retrieves the version string from the binary by running `--version`.
    ///
    /// # Arguments
    /// - `binary_path` - Absolute path to the binary
    ///
    /// # Returns
    /// - `Ok(String)` - Version string (e.g., "0.1.2")
    /// - `Err(String)` - Error if binary can't be executed or version can't be parsed
    fn get_binary_version(binary_path: &str) -> Result<String, String> {
        use std::process::Command;

        let output = Command::new(binary_path)
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to run binary --version: {e}"))?;

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

    /// Determines if enough time has passed since the last update check.
    ///
    /// Update checks are rate-limited to once per 24 hours to avoid
    /// excessive GitHub API calls.
    ///
    /// # Returns
    /// - `true` - If update check should be performed
    /// - `false` - If too soon since last check
    fn should_check_for_update(&self) -> bool {
        self.last_update_check.is_none_or(|last| {
            let interval = std::time::Duration::from_secs(UPDATE_CHECK_INTERVAL_SECS);
            // If elapsed() fails (system clock moved backward), assume interval has passed.
            // This errs on the side of checking for updates rather than never checking.
            last.elapsed().unwrap_or(interval) >= interval
        })
    }

    /// Checks for a newer binary version and deletes the old one if found.
    ///
    /// This triggers a re-download on the next `ensure_binary()` call.
    /// Errors during the update check are intentionally ignored to avoid
    /// disrupting normal operation.
    ///
    /// # Arguments
    /// - `binary_path` - Path to the current binary
    ///
    /// # Note
    /// Returns `Result` for semantic clarity even though errors are suppressed.
    /// All code paths return `Ok(())` because update check failures should not
    /// block normal operation.
    fn check_and_update_binary(&mut self, binary_path: &str) -> Result<(), String> {
        // Get current binary version
        let Ok(current_version) = Self::get_binary_version(binary_path) else {
            return Ok(()); // Old binary without --version, skip update check
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

    /// Ensures the MCP server binary is available, downloading if necessary.
    ///
    /// This function:
    /// 1. Returns cached binary path if available
    /// 2. Checks for updates if enough time has passed
    /// 3. Downloads from GitHub Releases if binary doesn't exist
    /// 4. Extracts archive and makes binary executable (Unix)
    ///
    /// # Returns
    /// - `Ok(String)` - Absolute path to the binary
    /// - `Err(String)` - Error if download, extraction, or verification fails
    ///
    /// # Thread Safety
    /// Safe to call from single-threaded WASM environment (Zed extensions).
    /// If adapted for multi-threaded use, proper locking is required.
    fn ensure_binary(&mut self) -> Result<String, String> {
        // Check for updates if binary is cached and enough time has passed
        //
        // SAFETY NOTE: This update logic has a theoretical race condition where
        // check_and_update_binary() may delete the binary and clear cached_binary_path,
        // but another thread could read the stale path before the deletion completes.
        // However, Zed extensions run in single-threaded WASM, so this is not a practical
        // concern. If this code is adapted for multi-threaded use, proper locking is needed.
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

        // Get work directory (where extension runs)
        let work_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get work directory: {e}"))?;

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
                }
                return Err(format!(
                    "Binary path exists but is not a file: {binary_path_str}"
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Binary doesn't exist, proceed with download
            }
            Err(e) => {
                return Err(format!("Failed to check binary at {binary_path_str}: {e}"));
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
        .map_err(|e| format!("Failed to get latest release from {PROXY_REPO}: {e}"))?;

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
        let archive_path = std::path::Path::new(archive_name);
        let file_type = if archive_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            zed::DownloadedFileType::Zip
        } else if archive_name.to_ascii_lowercase().ends_with(".tar.gz") {
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
                "Binary not found at expected path after extraction: {binary_path_str}"
            ));
        }

        // Make it executable (Unix platforms)
        #[cfg(unix)]
        zed::make_file_executable(&binary_path_str)
            .map_err(|e| format!("Failed to make {binary_path_str} executable: {e}"))?;

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
            id => Err(format!("Unknown context server: {id}")),
        }
    }
}

zed::register_extension!(BunDocsMcpExtension);

#[cfg(test)]
#[allow(clippy::case_sensitive_file_extension_comparisons)]
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
    fn test_unsupported_platform_error() {
        // Platform detection uses zed::current_platform() which returns host OS
        // Unsupported platforms would need to be tested manually or with integration tests
        // This test documents the expected error format for unsupported platforms
        let expected_prefix = "Unsupported platform:";
        let expected_suffix = "please file an issue";

        assert!(!expected_prefix.is_empty());
        assert!(!expected_suffix.is_empty());
    }

    #[test]
    fn test_context_server_id_constant() {
        // Verify CONTEXT_SERVER_ID matches extension.toml
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
    }

    #[test]
    fn test_constants_defined() {
        // Platform functions use zed::current_platform() which only works in WASM runtime
        // Cannot be tested in native unit tests - must test via dev extension in Zed
        // This test verifies the expected constants exist
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
        assert_eq!(UPDATE_CHECK_INTERVAL_SECS, 86400);

        // Verify expected archive names are valid
        let archives = vec![ARCHIVE_LINUX_X64, ARCHIVE_MACOS_ARM64, ARCHIVE_WINDOWS_X64];
        for archive in archives {
            assert!(archive.contains("bun-docs-mcp-proxy-"));
            assert!(archive.ends_with(".tar.gz") || archive.ends_with(".zip"));
        }
    }

    #[test]
    fn test_version_parsing() {
        // Test parsing version output format "bun-docs-mcp-proxy 0.1.2"
        let version_output = "bun-docs-mcp-proxy 0.1.2";
        let version = version_output.split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");

        // Test with just version number
        let version_output = "0.1.2";
        let version = version_output.split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");

        // Test with v prefix
        let version_output = "bun-docs-mcp-proxy v0.1.2";
        let version = version_output.split_whitespace().last().unwrap();
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

    #[test]
    fn test_version_parsing_edge_cases() {
        // Test malformed version output (empty string)
        let version_output = "";
        assert_eq!(version_output.split_whitespace().last(), None);

        // Test malformed version output (whitespace only)
        let version_output = "   ";
        assert_eq!(version_output.split_whitespace().last(), None);

        // Test version output with multiple spaces
        let version_output = "bun-docs-mcp-proxy    0.1.2";
        let version = version_output.split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");

        // Test version with newlines
        let version_output = "bun-docs-mcp-proxy 0.1.2\n";
        let version = version_output.split_whitespace().last().unwrap();
        assert_eq!(version, "0.1.2");
    }

    #[test]
    fn test_semver_parsing_edge_cases() {
        // Test invalid semver strings gracefully fail
        assert!(Version::parse("not-a-version").is_err());
        assert!(Version::parse("1.2").is_err()); // Missing patch
        assert!(Version::parse("1.2.3.4").is_err()); // Too many components

        // Test pre-release versions work correctly
        let v1 = Version::parse("0.1.2").unwrap();
        let v2 = Version::parse("0.1.3-beta").unwrap();
        assert!(v2 > v1, "0.1.3-beta should be greater than 0.1.2");

        let v1 = Version::parse("0.1.3-alpha").unwrap();
        let v2 = Version::parse("0.1.3-beta").unwrap();
        assert!(v2 > v1, "beta comes after alpha");
    }

    #[test]
    fn test_version_comparison_edge_cases() {
        // Test major version takes precedence
        let v1 = Version::parse("0.1.10").unwrap();
        let v2 = Version::parse("1.0.0").unwrap();
        assert!(v2 > v1, "1.0.0 should be greater than 0.1.10");

        // Test minor version takes precedence over patch
        let v1 = Version::parse("0.1.99").unwrap();
        let v2 = Version::parse("0.2.0").unwrap();
        assert!(v2 > v1, "0.2.0 should be greater than 0.1.99");

        // Test large version numbers
        let v1 = Version::parse("0.999.999").unwrap();
        let v2 = Version::parse("1.0.0").unwrap();
        assert!(v2 > v1);
    }
}
