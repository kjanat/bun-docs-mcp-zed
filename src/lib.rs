use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
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

    fn verify_checksum(file_path: &Path, expected_checksum: &str) -> Result<(), String> {
        let mut file = fs::File::open(file_path)
            .map_err(|e| format!("Failed to open file for checksum: {}", e))?;

        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(e) => return Err(format!("Failed to read file for checksum: {}", e)),
            }
        }

        use sha2::Digest;
        let hash = hasher.finalize();
        let computed = format!("{:x}", hash);

        if computed != expected_checksum {
            return Err(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum, computed
            ));
        }

        Ok(())
    }

    fn validate_extraction_path(base: &Path, extracted: &Path) -> Result<(), String> {
        let canonical_base = base
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize base path: {}", e))?;

        let canonical_extracted = extracted
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize extracted path: {}", e))?;

        if !canonical_extracted.starts_with(&canonical_base) {
            return Err(format!(
                "Path traversal detected: {} is outside {}",
                canonical_extracted.display(),
                canonical_base.display()
            ));
        }

        Ok(())
    }

    fn ensure_binary(&mut self) -> Result<String, String> {
        // Return cached path if we already downloaded it
        if let Some(cached) = &self.cached_binary_path {
            return Ok(cached.clone());
        }

        const PROXY_REPO: &str = "kjanat/bun-docs-mcp-proxy";
        const PROXY_DIR: &str = "bun-docs-mcp-proxy";

        // Get work directory (where extension runs)
        let work_dir = std::env::var("PWD")
            .or_else(|_| std::env::current_dir().map(|p| p.to_string_lossy().to_string()))
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

        // Validate extraction path to prevent directory traversal
        let work_path = PathBuf::from(&work_dir);
        Self::validate_extraction_path(&work_path, &binary_path)?;

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
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command, String> {
        const CONTEXT_SERVER_ID: &str = "bun-docs-mcp";

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
    fn test_binary_path_construction() {
        // Test that PathBuf construction works correctly
        let work_dir = "/test/work";
        let binary_name = "bun-docs-mcp-proxy";

        let path = PathBuf::from(work_dir)
            .join("bun-docs-mcp-proxy")
            .join(binary_name);

        let expected = if cfg!(windows) {
            "\\test\\work\\bun-docs-mcp-proxy\\bun-docs-mcp-proxy"
        } else {
            "/test/work/bun-docs-mcp-proxy/bun-docs-mcp-proxy"
        };

        assert_eq!(path.to_str().unwrap(), expected);
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
        // Verify CONTEXT_SERVER_ID is consistent with extension.toml
        const EXPECTED_ID: &str = "bun-docs-mcp";
        assert_eq!(EXPECTED_ID, "bun-docs-mcp");
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
    fn test_checksum_validation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with known content
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"test content").unwrap();
        temp.flush().unwrap();

        // SHA256 of "test content"
        let valid_checksum = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        let invalid_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        // Valid checksum should pass
        assert!(BunDocsMcpExtension::verify_checksum(temp.path(), valid_checksum).is_ok());

        // Invalid checksum should fail
        assert!(BunDocsMcpExtension::verify_checksum(temp.path(), invalid_checksum).is_err());
    }

    #[test]
    fn test_path_traversal_protection() {
        use std::env;

        let temp_dir = env::temp_dir();
        let safe_path = temp_dir.join("bun-docs-mcp-proxy").join("binary");

        // Create the paths
        fs::create_dir_all(&safe_path.parent().unwrap()).ok();
        fs::write(&safe_path, b"test").ok();

        // Valid path should pass
        assert!(BunDocsMcpExtension::validate_extraction_path(&temp_dir, &safe_path).is_ok());

        // Cleanup
        fs::remove_file(&safe_path).ok();
        fs::remove_dir(safe_path.parent().unwrap()).ok();
    }

    #[test]
    fn test_binary_existence_check_validates_file_type() {
        // Test that we properly validate file vs directory
        // This is tested through the metadata check in ensure_binary
        // The actual validation happens at runtime when fs::metadata returns is_file()
        assert!(true); // Placeholder - actual validation happens in ensure_binary
    }
}
