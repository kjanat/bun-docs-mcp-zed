use schemars::JsonSchema;
use serde::Deserialize;
use std::fs;
use zed_extension_api::{
    self as zed, Command, ContextServerConfiguration, ContextServerId, Project, Result, serde_json,
    settings::ContextServerSettings,
};

const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";
const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";
const PROXY_DIR: &str = "bun-docs-mcp-proxy";
const PROXY_VERSION: &str = "v0.3.0";
const ARCHIVE_LINUX_X64: &str = "bun-docs-mcp-proxy-linux-x86_64.tar.gz";
const ARCHIVE_LINUX_ARM64: &str = "bun-docs-mcp-proxy-linux-aarch64.tar.gz";
const ARCHIVE_MACOS_X64: &str = "bun-docs-mcp-proxy-macos-x86_64.tar.gz";
const ARCHIVE_MACOS_ARM64: &str = "bun-docs-mcp-proxy-macos-aarch64.tar.gz";
const ARCHIVE_WINDOWS_X64: &str = "bun-docs-mcp-proxy-windows-x86_64.zip";
const ARCHIVE_WINDOWS_ARM64: &str = "bun-docs-mcp-proxy-windows-aarch64.zip";
const BINARY_NAME_UNIX: &str = "bun-docs-mcp-proxy";
const BINARY_NAME_WINDOWS: &str = "bun-docs-mcp-proxy.exe";

#[derive(Debug, Deserialize, JsonSchema, Default)]
struct BunDocsMcpSettings {
    path: Option<String>,
}

struct BunDocsMcpExtension {
    cached_binary_path: Option<String>,
    did_legacy_cleanup: bool,
}

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

fn binary_name_for(os: zed::Os) -> &'static str {
    if os == zed::Os::Windows {
        BINARY_NAME_WINDOWS
    } else {
        BINARY_NAME_UNIX
    }
}

fn binary_rel_path(version: &str, os: zed::Os) -> String {
    let binary_name = binary_name_for(os);
    format!("{PROXY_DIR}/{version}/{binary_name}")
}

fn extraction_dir(version: &str) -> String {
    format!("{PROXY_DIR}/{version}")
}

fn expand_tilde(path: &str) -> String {
    #[cfg(unix)]
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return format!("{home}/{rest}");
    }
    path.to_string()
}

/// Validates a user-provided binary path by executing it with `--version`.
///
/// Checks that:
/// 1. The binary can be executed
/// 2. It exits successfully (code 0)
/// 3. The output contains "bun-docs-mcp-proxy" (verifies it's our binary)
fn validate_user_binary(path: &str) -> Result<()> {
    let output = zed::process::Command::new(path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute custom binary at {path}: {e}"))?;

    match output.status {
        Some(0) => {
            // Verify it's actually our binary by checking the output
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains("bun-docs-mcp-proxy") {
                return Err(format!(
                    "Binary at {path} is not bun-docs-mcp-proxy (output: {stdout})"
                ));
            }
            Ok(())
        }
        Some(code) => Err(format!(
            "Custom binary at {path} exited with code {code}. stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )),
        None => Err(format!("Custom binary at {path} was terminated by signal")),
    }
}

impl BunDocsMcpExtension {
    fn get_platform_archive_name() -> Result<&'static str> {
        let (os, arch) = zed::current_platform();
        archive_name_for(os, arch)
    }

    fn get_binary_rel_path() -> String {
        let (os, _) = zed::current_platform();
        binary_rel_path(PROXY_VERSION, os)
    }

    fn ensure_binary(&mut self) -> Result<String> {
        if !self.did_legacy_cleanup {
            self.did_legacy_cleanup = true;
            let (os, _) = zed::current_platform();
            let legacy_binary = format!("{PROXY_DIR}/{}", binary_name_for(os));
            if fs::metadata(&legacy_binary)
                .map(|m| m.is_file())
                .unwrap_or(false)
            {
                let _ = fs::remove_file(&legacy_binary);
            }
        }

        // Re-validate cached path in case user deleted the binary while Zed was running
        if let Some(cached) = &self.cached_binary_path {
            if fs::metadata(cached).is_ok_and(|m| m.is_file() && m.len() > 0) {
                return Ok(cached.clone());
            }
            self.cached_binary_path = None;
        }

        let binary_path = Self::get_binary_rel_path();

        match fs::metadata(&binary_path) {
            Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {
                self.cached_binary_path = Some(binary_path.clone());
                return Ok(binary_path);
            }
            Ok(meta) => {
                if meta.is_dir() {
                    let _ = fs::remove_dir_all(&binary_path);
                } else {
                    let _ = fs::remove_file(&binary_path);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("Failed to check binary at {binary_path}: {e}")),
        }

        let release = zed::github_release_by_tag_name(PROXY_REPO, PROXY_VERSION)
            .map_err(|e| format!("Failed to get release {PROXY_VERSION} from {PROXY_REPO}: {e}"))?;

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

        let extract_dir = extraction_dir(PROXY_VERSION);
        zed::download_file(&asset.download_url, &extract_dir, file_type).map_err(|e| {
            format!(
                "Failed to download {} from {}: {}",
                archive_name, asset.download_url, e
            )
        })?;

        match fs::metadata(&binary_path) {
            Ok(metadata) if metadata.is_file() && metadata.len() > 0 => {}
            Ok(_) => {
                return Err(format!(
                    "Extracted binary is invalid (not a file or empty): {binary_path}"
                ));
            }
            Err(_) => {
                return Err(format!("Binary not found after extraction: {binary_path}"));
            }
        }

        #[cfg(unix)]
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
            did_legacy_cleanup: false,
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        match context_server_id.as_ref() {
            CONTEXT_SERVER_ID => {
                let settings = ContextServerSettings::for_project(CONTEXT_SERVER_ID, project)
                    .map_err(|e| format!("Failed to load context server settings: {e}"))?;

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

                let binary_path = match custom_settings.as_ref().and_then(|s| s.path.as_ref()) {
                    Some(path) => {
                        let expanded = expand_tilde(path);
                        if expanded.trim().is_empty() {
                            return Err(
                                "Custom binary path is empty - remove 'path' setting or provide a valid path".to_string()
                            );
                        }
                        validate_user_binary(&expanded)?;
                        expanded
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
    #[cfg(unix)]
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_archive_name_for() {
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

    #[test]
    fn test_expand_tilde() {
        // Non-tilde paths pass through unchanged
        assert_eq!(expand_tilde("/usr/bin/foo"), "/usr/bin/foo");
        assert_eq!(expand_tilde("relative/path"), "relative/path");
        assert_eq!(expand_tilde(""), "");

        // Tilde without slash is not expanded (not a home dir reference)
        assert_eq!(expand_tilde("~foo"), "~foo");
        assert_eq!(expand_tilde("~"), "~");

        // Tilde expansion only on Unix with HOME set
        #[cfg(unix)]
        {
            // SAFETY: set_var/remove_var are unsafe since Rust 1.84 due to potential data
            // races in multi-threaded contexts. We serialize access via ENV_LOCK.
            let _guard = ENV_LOCK.lock().unwrap();
            let original_home = std::env::var("HOME").ok();

            unsafe { std::env::set_var("HOME", "/home/testuser") };
            assert_eq!(expand_tilde("~/bin/proxy"), "/home/testuser/bin/proxy");
            assert_eq!(expand_tilde("~/.config/app"), "/home/testuser/.config/app");

            match original_home {
                Some(h) => unsafe { std::env::set_var("HOME", h) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }
}
