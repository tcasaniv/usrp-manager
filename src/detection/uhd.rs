use crate::executor::run_silent;
use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone)]
pub struct ActiveUhd {
    pub version: String,
    pub bin_dir: PathBuf,
}

/// Detect the currently active UHD installation in PATH.
pub fn detect_active() -> Option<ActiveUhd> {
    // Try uhd_config_info first (most reliable for version)
    let uhd_config = which("uhd_config_info").ok()?;
    let version_out = run_silent(uhd_config.to_str()?, &["--version"])?;

    // Output is usually: "UHD 4.10.0.0-0-gabcdef"
    let version = parse_uhd_version(&version_out)?;
    let bin_dir = uhd_config.parent()?.to_path_buf();

    Some(ActiveUhd { version, bin_dir })
}

fn parse_uhd_version(raw: &str) -> Option<String> {
    // "UHD 4.10.0.0-0-g..." → "4.10.0.0"
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("UHD ") {
            // Strip git hash suffix
            let version = rest.split('-').next()?.to_string();
            return Some(version);
        }
        // Some versions just output the version number directly
        if line.starts_with(|c: char| c.is_ascii_digit()) {
            let version = line.split('-').next()?.to_string();
            return Some(version);
        }
    }
    None
}

/// Get UHD version string from a specific binary path.
pub fn version_from_binary(uhd_config_info: &PathBuf) -> Option<String> {
    let out = run_silent(uhd_config_info.to_str()?, &["--version"])?;
    parse_uhd_version(&out)
}
