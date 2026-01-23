use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use zed_extension_api::{
    self as zed, serde_json, settings::ContextServerSettings, Command, ContextServerConfiguration,
    ContextServerId, Project, Result,
};

// Context server identifier that must match extension.toml
const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

// GitHub repository for the proxy binary
const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";

// Base directory for proxy binaries (version subdirs go inside)
const PROXY_DIR: &str = "bun-docs-mcp-proxy";

// Pinned proxy binary version - update this when releasing new extension version
// Binaries are stored in versioned directories, so bumping this triggers re-download
const PROXY_VERSION: &str = "v0.3.0";

// Platform-specific archive names for binary distribution
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";

// Binary name (without path)
const BINARY_NAME_UNIX: &str = "bun-docs-mcp-proxy";
const BINARY_NAME_WINDOWS: &str = "bun-docs-mcp-proxy.exe";

/// Custom settings for the Bun Docs MCP server.
/// Parsed from the `settings` JSON blob in Zed's context server configuration.
#[derive(Debug, Deserialize, JsonSchema, Default)]
struct BunDocsMcpSettings {
    /// Path to a custom bun-docs-mcp-proxy binary.
    /// If not set, the extension will automatically download and manage the binary.
    path: Option<String>,
}

struct BunDocsMcpExtension {
    /// Cached relative path to the binary (within extension work directory)
    cached_binary_path: Option<String>,
}

/// Returns the archive name for a given OS and architecture.
/// This is a pure function that can be tested without Zed runtime.
fn archive_name_for(os: zed::Os, arch: zed::Architecture) -> Result<&'static str> {
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

/// Returns the binary filename for a given OS.
/// This is a pure function that can be tested without Zed runtime.
fn binary_name_for(os: zed::Os) -> &'static str {
    if os == zed::Os::Windows {
        BINARY_NAME_WINDOWS
    } else {
        BINARY_NAME_UNIX
    }
}

/// Constructs the relative path where the binary should be located.
/// Path format: `{PROXY_DIR}/{version}/{binary_name}`
/// This is relative to the extension's work directory.
fn binary_rel_path(version: &str, os: zed::Os) -> String {
    let binary_name = binary_name_for(os);
    format!("{PROXY_DIR}/{version}/{binary_name}")
}

/// Constructs the extraction directory path for downloads.
/// Path format: `{PROXY_DIR}/{version}`
fn extraction_dir(version: &str) -> String {
    format!("{PROXY_DIR}/{version}")
}

impl BunDocsMcpExtension {
    /// Returns the GitHub release archive name for the current platform.
    fn get_platform_archive_name() -> Result<&'static str> {
        let (os, arch) = zed::current_platform();
        archive_name_for(os, arch)
    }

    /// Returns the relative binary path for the current platform and pinned version.
    fn get_binary_rel_path() -> String {
        let (os, _) = zed::current_platform();
        binary_rel_path(PROXY_VERSION, os)
    }

    /// Ensures the MCP server binary is available, downloading if necessary.
    /// Returns a relative path within the extension work directory.
    fn ensure_binary(&mut self) -> Result<String> {
        // Return cached path if available
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        let binary_path = Self::get_binary_rel_path();

        // Check if binary already exists and is valid
        match fs::metadata(&binary_path) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(format!(
                        "Binary path exists but is not a file: {binary_path}"
                    ));
                }
                if metadata.len() == 0 {
                    return Err(format!("Binary file is empty: {binary_path}"));
                }
                // Binary exists and is valid
                self.cached_binary_path = Some(binary_path.clone());
                return Ok(binary_path);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Binary doesn't exist, proceed with download
            }
            Err(e) => {
                return Err(format!("Failed to check binary at {binary_path}: {e}"));
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

        // Determine file type for extraction
        let archive_path = std::path::Path::new(archive_name);
        let file_type = if archive_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            zed::DownloadedFileType::Zip
        } else if archive_name.ends_with(".tar.gz") {
            // .tar.gz is a compound extension, check as string
            zed::DownloadedFileType::GzipTar
        } else {
            zed::DownloadedFileType::Uncompressed
        };

        // Download and extract to versioned directory
        let extract_dir = extraction_dir(PROXY_VERSION);
        zed::download_file(&asset.download_url, &extract_dir, file_type).map_err(|e| {
            format!(
                "Failed to download {} from {}: {}",
                archive_name, asset.download_url, e
            )
        })?;

        // Verify the binary was extracted correctly
        match fs::metadata(&binary_path) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(format!("Extracted path is not a file: {binary_path}"));
                }
                if metadata.len() == 0 {
                    return Err(format!("Extracted binary is empty: {binary_path}"));
                }
            }
            Err(_) => {
                return Err(format!("Binary not found after extraction: {binary_path}"));
            }
        }

        // Make it executable (Unix platforms)
        zed::make_file_executable(&binary_path)
            .map_err(|e| format!("Failed to make {binary_path} executable: {e}"))?;

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
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        match context_server_id.as_ref() {
            CONTEXT_SERVER_ID => {
                // Get Zed's context server settings - surface errors instead of swallowing
                let settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project)
                    .map_err(|e| format!("Failed to load context server settings: {e}"))?;

                // Parse custom settings from the settings JSON blob
                // If settings exist but are invalid, return a hard error
                let custom_settings: Option<BunDocsMcpSettings> = if let Some(ref value) =
                    settings.settings
                {
                    Some(
                        serde_json::from_value(value.clone())
                            .map_err(|e| format!("Invalid {CONTEXT_SERVER_ID} settings: {e}"))?,
                    )
                } else {
                    None
                };

                // Determine binary path: user-specified or auto-download
                let binary_path = match custom_settings.as_ref().and_then(|s| s.path.as_ref()) {
                    Some(path) => {
                        // Basic validation for user-provided path
                        if path.trim().is_empty() {
                            return Err(
                                "Custom binary path is empty - remove 'path' setting or provide a valid path".to_string()
                            );
                        }
                        path.clone()
                    }
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
mod tests {
    use super::*;

    #[test]
    fn test_archive_name_for() {
        // Test all supported platforms
        assert_eq!(
            archive_name_for(zed::Os::Linux, zed::Architecture::X8664).unwrap(),
            ARCHIVE_LINUX_X64
        );
        assert_eq!(
            archive_name_for(zed::Os::Linux, zed::Architecture::Aarch64).unwrap(),
            ARCHIVE_LINUX_ARM64
        );
        assert_eq!(
            archive_name_for(zed::Os::Mac, zed::Architecture::X8664).unwrap(),
            ARCHIVE_MACOS_X64
        );
        assert_eq!(
            archive_name_for(zed::Os::Mac, zed::Architecture::Aarch64).unwrap(),
            ARCHIVE_MACOS_ARM64
        );
        assert_eq!(
            archive_name_for(zed::Os::Windows, zed::Architecture::X8664).unwrap(),
            ARCHIVE_WINDOWS_X64
        );
        assert_eq!(
            archive_name_for(zed::Os::Windows, zed::Architecture::Aarch64).unwrap(),
            ARCHIVE_WINDOWS_ARM64
        );
    }

    #[test]
    fn test_archive_name_for_unsupported() {
        // Test unsupported platform returns error
        let result = archive_name_for(zed::Os::Linux, zed::Architecture::X86);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported platform"));
    }

    #[test]
    fn test_binary_name_for() {
        assert_eq!(binary_name_for(zed::Os::Linux), BINARY_NAME_UNIX);
        assert_eq!(binary_name_for(zed::Os::Mac), BINARY_NAME_UNIX);
        assert_eq!(binary_name_for(zed::Os::Windows), BINARY_NAME_WINDOWS);
    }

    #[test]
    fn test_binary_rel_path() {
        assert_eq!(
            binary_rel_path("v0.3.0", zed::Os::Linux),
            "bun-docs-mcp-proxy/v0.3.0/bun-docs-mcp-proxy"
        );
        assert_eq!(
            binary_rel_path("v0.3.0", zed::Os::Mac),
            "bun-docs-mcp-proxy/v0.3.0/bun-docs-mcp-proxy"
        );
        assert_eq!(
            binary_rel_path("v0.3.0", zed::Os::Windows),
            "bun-docs-mcp-proxy/v0.3.0/bun-docs-mcp-proxy.exe"
        );
    }

    #[test]
    fn test_extraction_dir() {
        assert_eq!(extraction_dir("v0.3.0"), "bun-docs-mcp-proxy/v0.3.0");
        assert_eq!(extraction_dir("v1.0.0"), "bun-docs-mcp-proxy/v1.0.0");
    }

    #[test]
    fn test_archive_extensions_valid() {
        // All archives should have valid extensions
        let archives = [
            ARCHIVE_LINUX_X64,
            ARCHIVE_LINUX_ARM64,
            ARCHIVE_MACOS_X64,
            ARCHIVE_MACOS_ARM64,
            ARCHIVE_WINDOWS_X64,
            ARCHIVE_WINDOWS_ARM64,
        ];

        for archive in archives {
            assert!(
                archive.ends_with(".tar.gz") || archive.ends_with(".zip"),
                "Archive {archive} should have .tar.gz or .zip extension"
            );
        }

        // Windows uses .zip, others use .tar.gz
        assert!(ARCHIVE_WINDOWS_X64.ends_with(".zip"));
        assert!(ARCHIVE_WINDOWS_ARM64.ends_with(".zip"));
        assert!(ARCHIVE_LINUX_X64.ends_with(".tar.gz"));
        assert!(ARCHIVE_MACOS_ARM64.ends_with(".tar.gz"));
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
    fn test_constants_consistency() {
        // Verify constants are consistent
        assert_eq!(CONTEXT_SERVER_ID, "bun-docs-mcp");
        assert_eq!(PROXY_REPO, "kjanat/bun-docs-mcp-proxy");
        assert_eq!(PROXY_DIR, "bun-docs-mcp-proxy");
        assert_eq!(BINARY_NAME_UNIX, "bun-docs-mcp-proxy");
        assert_eq!(BINARY_NAME_WINDOWS, "bun-docs-mcp-proxy.exe");
    }

    #[test]
    fn test_settings_schema_generation() {
        let schema = schemars::schema_for!(BunDocsMcpSettings);
        let json = serde_json::to_string(&schema).unwrap();
        assert!(json.contains("path"));
        // Should NOT contain nested "command" anymore
        assert!(!json.contains("command"));
    }

    #[test]
    fn test_settings_deserialization() {
        // Valid settings
        let json = r#"{"path": "/custom/binary"}"#;
        let settings: BunDocsMcpSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.path, Some("/custom/binary".to_string()));

        // Empty settings (all optional)
        let json = r#"{}"#;
        let settings: BunDocsMcpSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.path, None);

        // Null path
        let json = r#"{"path": null}"#;
        let settings: BunDocsMcpSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.path, None);
    }

    #[test]
    fn test_settings_deserialization_invalid() {
        // Wrong type for path should fail
        let json = r#"{"path": 123}"#;
        let result: std::result::Result<BunDocsMcpSettings, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
