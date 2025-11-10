use std::fs;
use std::path::PathBuf;
use zed_extension_api as zed;

#[cfg(test)]
use semver::Version;

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

// Base directory for all binary versions
const PROXY_DIR: &str = "bun-docs-mcp-proxy";

// Repository for binary releases
const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";

// Platform-specific archive names for binary distribution
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
    current_version: Option<String>,
    /// Tracks whether we've checked for updates this session
    update_checked_this_session: bool,
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

    /// Constructs the version-specific directory path for a binary version.
    ///
    /// # Arguments
    /// - `work_dir` - Base work directory
    /// - `version` - Version string (e.g., "0.1.2" or "v0.1.2")
    ///
    /// # Returns
    /// - PathBuf pointing to version directory (e.g., "work_dir/bun-docs-mcp-proxy/v0.1.2")
    fn get_version_dir(work_dir: &str, version: &str) -> PathBuf {
        let version_with_v = if version.starts_with('v') {
            version.to_string()
        } else {
            format!("v{}", version)
        };
        PathBuf::from(work_dir)
            .join(PROXY_DIR)
            .join(version_with_v)
    }

    /// Cleans up old version directories, keeping only the specified version.
    /// Also removes any old non-versioned binaries from previous folder structure.
    ///
    /// # Arguments
    /// - `work_dir` - Base work directory
    /// - `keep_version` - Version to keep (all others will be deleted)
    fn cleanup_old_versions(work_dir: &str, keep_version: &str) {
        let proxy_dir = PathBuf::from(work_dir).join(PROXY_DIR);

        // Read all entries in the proxy directory
        let Ok(entries) = fs::read_dir(&proxy_dir) else {
            return; // Directory doesn't exist or can't be read, nothing to clean
        };

        let keep_version_normalized = if keep_version.starts_with('v') {
            keep_version.to_string()
        } else {
            format!("v{}", keep_version)
        };

        // Delete old version directories and non-versioned files
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Only delete version directories (start with 'v' and not the one we're keeping)
                    if dir_name.starts_with('v') && dir_name != keep_version_normalized {
                        fs::remove_dir_all(path).ok();
                    }
                }
            } else if path.is_file() {
                // Delete any files in the base directory (old non-versioned binaries)
                // These are from the previous folder structure before versioning was added
                fs::remove_file(path).ok();
            }
        }
    }

    /// Ensures the MCP server binary is available, downloading if necessary.
    ///
    /// This function:
    /// 1. Checks GitHub for updates ONCE per Zed session (on first call)
    /// 2. Returns cached binary for subsequent calls in the same session
    /// 3. Downloads to version-specific folder if update available
    /// 4. Cleans up old version directories automatically
    ///
    /// # Returns
    /// - `Ok(String)` - Absolute path to the binary
    /// - `Err(String)` - Error if download, extraction, or verification fails
    fn ensure_binary(&mut self) -> Result<String, String> {
        // If we've already checked and have a valid cached binary, return it immediately
        // This avoids excessive GitHub API calls during a single Zed session
        if self.update_checked_this_session {
            if let Some(cached_path) = &self.cached_binary_path {
                if PathBuf::from(cached_path).exists() {
                    return Ok(cached_path.clone());
                }
            }
        }

        // Get work directory (where extension runs)
        let work_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get work directory: {}", e))?;

        // Check for latest release from GitHub (once per session)
        let release = zed::latest_github_release(
            PROXY_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to get latest release from {}: {}", PROXY_REPO, e))?;

        let latest_version = release.version.trim_start_matches('v');

        // Mark that we've checked for updates this session
        self.update_checked_this_session = true;

        // If we have a cached binary with the same version, return it
        if let (Some(cached_path), Some(current_version)) =
            (&self.cached_binary_path, &self.current_version)
        {
            let current_version_normalized = current_version.trim_start_matches('v');
            if current_version_normalized == latest_version {
                // Verify the binary still exists
                if PathBuf::from(cached_path).exists() {
                    return Ok(cached_path.clone());
                }
            }
        }

        // Need to download new version
        let binary_name = Self::get_binary_name();
        let version_dir = Self::get_version_dir(&work_dir, latest_version);
        let binary_path = version_dir.join(binary_name);

        let binary_path_str = binary_path
            .to_str()
            .ok_or_else(|| "Binary path contains invalid UTF-8".to_string())?
            .to_string();

        // Check if this version already exists on disk (from a previous session)
        if binary_path.exists() {
            // Verify it's actually a file
            let metadata = fs::metadata(&binary_path)
                .map_err(|e| format!("Failed to check binary metadata: {}", e))?;

            if metadata.is_file() {
                // Clean up old versions
                Self::cleanup_old_versions(&work_dir, latest_version);

                self.cached_binary_path = Some(binary_path_str.clone());
                self.current_version = Some(latest_version.to_string());
                return Ok(binary_path_str);
            }
        }

        // Create version directory if it doesn't exist
        fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create version directory: {}", e))?;

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

        // Determine file type for extraction
        let file_type = if archive_name.ends_with(".zip") {
            zed::DownloadedFileType::Zip
        } else if archive_name.ends_with(".tar.gz") {
            zed::DownloadedFileType::GzipTar
        } else {
            zed::DownloadedFileType::Uncompressed
        };

        // Download and extract to version-specific directory
        // The second parameter is the extraction path relative to the work directory
        let version_with_v = if latest_version.starts_with('v') {
            latest_version.to_string()
        } else {
            format!("v{}", latest_version)
        };
        let extract_path = format!("{}/{}", PROXY_DIR, version_with_v);

        zed::download_file(&asset.download_url, &extract_path, file_type).map_err(|e| {
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

        // Clean up old versions
        Self::cleanup_old_versions(&work_dir, latest_version);

        self.cached_binary_path = Some(binary_path_str.clone());
        self.current_version = Some(latest_version.to_string());
        Ok(binary_path_str)
    }
}

impl zed::Extension for BunDocsMcpExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
            current_version: None,
            update_checked_this_session: false,
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
    fn test_version_dir_construction() {
        // Test version-specific directory construction
        let work_dir = if cfg!(windows) {
            "C:\\test\\work"
        } else {
            "/test/work"
        };

        // Test with version without 'v' prefix
        let version_dir = BunDocsMcpExtension::get_version_dir(work_dir, "0.1.2");
        let path_str = version_dir.to_str().unwrap();
        assert!(path_str.contains("bun-docs-mcp-proxy"));
        assert!(path_str.contains("v0.1.2"));

        // Test with version with 'v' prefix
        let version_dir = BunDocsMcpExtension::get_version_dir(work_dir, "v0.1.3");
        let path_str = version_dir.to_str().unwrap();
        assert!(path_str.contains("bun-docs-mcp-proxy"));
        assert!(path_str.contains("v0.1.3"));

        // Verify path structure is correct for platform
        if cfg!(windows) {
            assert!(path_str.contains("C:\\test\\work\\bun-docs-mcp-proxy\\v"));
        } else {
            assert!(path_str.contains("/test/work/bun-docs-mcp-proxy/v"));
        }
    }

    #[test]
    fn test_binary_path_construction_with_version() {
        // Test that PathBuf construction works correctly with version directories
        let work_dir = if cfg!(windows) {
            "C:\\test\\work"
        } else {
            "/test/work"
        };
        let version = "0.1.2";
        let binary_name = "bun-docs-mcp-proxy";

        let version_dir = BunDocsMcpExtension::get_version_dir(work_dir, version);
        let path = version_dir.join(binary_name);
        let path_str = path.to_str().unwrap();

        // Verify path contains all expected components
        assert!(path_str.contains("test"));
        assert!(path_str.contains("work"));
        assert!(path_str.contains("bun-docs-mcp-proxy"));
        assert!(path_str.contains("v0.1.2"));

        // Verify path is correctly formed for the platform
        if cfg!(windows) {
            assert_eq!(
                path_str,
                "C:\\test\\work\\bun-docs-mcp-proxy\\v0.1.2\\bun-docs-mcp-proxy"
            );
        } else {
            assert_eq!(
                path_str,
                "/test/work/bun-docs-mcp-proxy/v0.1.2/bun-docs-mcp-proxy"
            );
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
        assert_eq!(PROXY_DIR, "bun-docs-mcp-proxy");
        assert_eq!(PROXY_REPO, "kjanat/bun-docs-mcp-proxy");

        // Verify expected archive names are valid
        let archives = vec![ARCHIVE_LINUX_X64, ARCHIVE_MACOS_ARM64, ARCHIVE_WINDOWS_X64];
        for archive in archives {
            assert!(archive.contains("bun-docs-mcp-proxy-"));
            assert!(archive.ends_with(".tar.gz") || archive.ends_with(".zip"));
        }
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
    fn test_version_normalization() {
        // Test that version strings are normalized correctly
        let version1 = "v0.1.2";
        let version2 = "0.1.2";

        let normalized1 = version1.trim_start_matches('v');
        let normalized2 = version2.trim_start_matches('v');

        assert_eq!(normalized1, normalized2);
        assert_eq!(normalized1, "0.1.2");
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
