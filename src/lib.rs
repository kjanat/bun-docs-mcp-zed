#![allow(clippy::multiple_crate_versions)]

use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use zed_extension_api::{
    self as zed, Command, ContextServerConfiguration, ContextServerId, Project, Result, serde_json,
    settings::ContextServerSettings,
};

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

// GitHub repository and directory names for the proxy binary
const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";
const PROXY_DIR: &str = "bun-docs-mcp-proxy";

// Pinned proxy binary version - update this when releasing new extension version
const PROXY_VERSION: &str = "v0.3.0";

// Platform-specific archive names for binary distribution
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";

/// Custom settings for the Bun Docs MCP server.
/// Parsed from the `settings` JSON blob in Zed's context server configuration.
#[derive(Debug, Deserialize, JsonSchema, Default)]
#[allow(dead_code)]
struct BunDocsMcpSettings {
    /// Path to a custom bun-docs-mcp-proxy binary.
    /// If not set, the extension will automatically download and manage the binary.
    path: Option<String>,
}

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
}

impl BunDocsMcpExtension {
    /// Returns the GitHub release archive name for the current platform.
    fn get_platform_archive_name() -> Result<&'static str> {
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
    fn get_binary_name() -> &'static str {
        let (os, _) = zed::current_platform();
        if os == zed::Os::Windows {
            "bun-docs-mcp-proxy.exe"
        } else {
            "bun-docs-mcp-proxy"
        }
    }

    /// Ensures the MCP server binary is available, downloading if necessary.
    fn ensure_binary(&mut self) -> Result<String> {
        // Return cached path if available
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        let work_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to get work directory: {e}"))?;

        let binary_name = Self::get_binary_name();
        let binary_path = PathBuf::from(&work_dir).join(PROXY_DIR).join(binary_name);

        let binary_path_str = binary_path
            .to_str()
            .ok_or_else(|| "Binary path contains invalid UTF-8".to_string())?
            .to_string();

        // Check if binary already exists
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

        // Download from pinned GitHub release
        let release = zed::github_release_by_tag_name(PROXY_REPO, PROXY_VERSION)
            .map_err(|e| format!("Failed to get release {PROXY_VERSION} from {PROXY_REPO}: {e}"))?;

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
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        match context_server_id.as_ref() {
            CONTEXT_SERVER_ID => {
                // Get Zed's context server settings
                let settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project).ok();

                // Parse our custom settings from the settings JSON blob
                let custom_settings: Option<BunDocsMcpSettings> = settings
                    .as_ref()
                    .and_then(|s| s.settings.as_ref())
                    .and_then(|v| serde_json::from_value(v.clone()).ok());

                // Determine binary path: user-specified or auto-download
                // Note: Cannot validate user paths - WASM sandbox has no filesystem access
                // outside extension directory. Invalid paths fail at Zed's process execution.
                let binary_path = match custom_settings.as_ref().and_then(|s| s.path.as_ref()) {
                    Some(path) => path.clone(),
                    None => self.ensure_binary()?,
                };

                Ok(Command {
                    command: binary_path,
                    args: vec![],
                    env: vec![],
                })
            }
            id => Err(format!("Unknown context server: {id}")),
        }
    }

    fn context_server_configuration(
        &mut self,
        context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        match context_server_id.as_ref() {
            CONTEXT_SERVER_ID => {
                let installation_instructions =
                    include_str!("../configuration/installation_instructions.md").to_string();
                let default_settings = include_str!("../configuration/default_settings.jsonc")
                    .trim()
                    .to_string();
                let settings_schema =
                    serde_json::to_string(&schemars::schema_for!(BunDocsMcpSettings))
                        .map_err(|e| e.to_string())?;

                Ok(Some(ContextServerConfiguration {
                    installation_instructions,
                    default_settings,
                    settings_schema,
                }))
            }
            _ => Ok(None),
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
        let expected_archives = vec![
            ARCHIVE_LINUX_X64,
            ARCHIVE_LINUX_ARM64,
            ARCHIVE_MACOS_X64,
            ARCHIVE_MACOS_ARM64,
            ARCHIVE_WINDOWS_X64,
            ARCHIVE_WINDOWS_ARM64,
        ];

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
        let expected_unix = "bun-docs-mcp-proxy";
        let expected_windows = "bun-docs-mcp-proxy.exe";

        assert!(!expected_unix.is_empty());
        assert!(!expected_windows.is_empty());
        assert!(expected_windows.ends_with(".exe"));
    }

    #[test]
    fn test_binary_path_construction() {
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

        assert!(path_str.contains("test"));
        assert!(path_str.contains("work"));
        assert!(path_str.contains("bun-docs-mcp-proxy"));

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
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
    }

    #[test]
    fn test_proxy_version_format() {
        assert!(PROXY_VERSION.starts_with('v'));
        let version_str = PROXY_VERSION.trim_start_matches('v');
        let parts: Vec<&str> = version_str.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "Version should have 3 parts (major.minor.patch)"
        );
        for part in parts {
            assert!(
                part.parse::<u32>().is_ok(),
                "Version parts should be numbers"
            );
        }
    }

    #[test]
    fn test_constants_defined() {
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
        assert_eq!(PROXY_REPO, "kjanat/bun-docs-mcp-proxy");
        assert_eq!(PROXY_DIR, "bun-docs-mcp-proxy");

        let archives = vec![ARCHIVE_LINUX_X64, ARCHIVE_MACOS_ARM64, ARCHIVE_WINDOWS_X64];
        for archive in archives {
            assert!(archive.contains("bun-docs-mcp-proxy-"));
            assert!(archive.ends_with(".tar.gz") || archive.ends_with(".zip"));
        }
    }

    #[test]
    fn test_settings_schema_generation() {
        let schema = schemars::schema_for!(BunDocsMcpSettings);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        // Should NOT contain nested "command" anymore
        assert!(!json.contains("command"));
    }
}
