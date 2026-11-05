use std::path::{Path, PathBuf};

use super::uhd::version_from_binary;

#[derive(Debug, Clone)]
pub struct InstalledUhd {
    pub version: String,
    pub install_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub is_active: bool,
}

/// Scan the versioned UHD installations directory for installed versions.
/// On Windows: C:\Program Files\UHD_versions\UHD-4.10\ etc.
/// On Linux: /opt/uhd/uhd-4.10/ etc.
pub fn detect_installed_versions(versions_dir: &Path, active_bin_dir: Option<&Path>) -> Vec<InstalledUhd> {
    let mut result = Vec::new();

    let entries = match std::fs::read_dir(versions_dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Try common bin directory locations
        let bin_dir = find_bin_dir(&path);
        let bin_dir = match bin_dir {
            Some(b) => b,
            None => continue,
        };

        // Try to get version from uhd_config_info binary
        let uhd_config = bin_dir.join(if cfg!(target_os = "windows") {
            "uhd_config_info.exe"
        } else {
            "uhd_config_info"
        });

        let version = if uhd_config.exists() {
            version_from_binary(&uhd_config).unwrap_or_else(|| guess_version_from_path(&path))
        } else {
            guess_version_from_path(&path)
        };

        if version.is_empty() {
            continue;
        }

        let is_active = active_bin_dir
            .map(|a| {
                // Compare canonical paths if possible
                a.canonicalize().ok() == bin_dir.canonicalize().ok()
            })
            .unwrap_or(false);

        result.push(InstalledUhd {
            version,
            install_dir: path,
            bin_dir,
            is_active,
        });
    }

    result.sort_by(|a, b| a.version.cmp(&b.version));
    result
}

fn find_bin_dir(install_dir: &Path) -> Option<PathBuf> {
    let candidates = ["bin", "lib/uhd/utils"];
    for c in &candidates {
        let p = install_dir.join(c);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

fn guess_version_from_path(path: &Path) -> String {
    // Extract version from folder name like "UHD-4.10" or "uhd-4.10.0.0"
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let lower = name.to_lowercase();
    // Remove common prefixes
    let stripped = lower
        .strip_prefix("uhd-")
        .or_else(|| lower.strip_prefix("uhd_"))
        .unwrap_or(&lower);
    stripped.to_string()
}
