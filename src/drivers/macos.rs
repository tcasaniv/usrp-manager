#[cfg(target_os = "macos")]
use anyhow::{bail, Result};
#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
use crate::executor::{run_visible, run_visible_in_dir};

#[cfg(target_os = "macos")]
pub fn install_via_brew() -> Result<()> {
    // Check brew exists
    if which::which("brew").is_err() {
        bail!("Homebrew not found. Install it from https://brew.sh");
    }
    let r = run_visible("brew", &["install", "uhd"]);
    if !r.success {
        bail!("brew install uhd failed");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn build_from_source(tag: &str, prefix: &Path) -> Result<()> {
    let tmp = std::env::temp_dir().join(format!("uhd-build-{}", tag));
    std::fs::create_dir_all(&tmp)?;
    let repo_dir = tmp.join("uhd-src");

    if !repo_dir.exists() {
        let r = run_visible(
            "git",
            &[
                "clone",
                "--depth=1",
                "--branch",
                tag,
                "https://github.com/EttusResearch/uhd.git",
                repo_dir.to_str().unwrap_or("uhd-src"),
            ],
        );
        if !r.success {
            bail!("git clone failed");
        }
    }

    let host_dir = repo_dir.join("host");
    let build_dir = host_dir.join("build");
    std::fs::create_dir_all(&build_dir)?;

    let r = run_visible_in_dir(
        "cmake",
        &[
            "..",
            &format!("-DCMAKE_INSTALL_PREFIX={}", prefix.display()),
            "-DCMAKE_BUILD_TYPE=Release",
        ],
        &build_dir,
    );
    if !r.success {
        bail!("cmake configure failed");
    }

    let r = run_visible_in_dir("make", &["-j", "4"], &build_dir);
    if !r.success {
        bail!("make failed");
    }

    let r = run_visible_in_dir("sudo", &["make", "install"], &build_dir);
    if !r.success {
        bail!("make install failed");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn set_active_version(prefix: &Path) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    // zsh is default on macOS since Catalina
    let zprofile = home.join(".zprofile");
    let bin_dir = prefix.join("bin");
    let lib_dir = prefix.join("lib");

    let snippet = format!(
        "\n# usrp-manager: active UHD\nexport UHD_DIR=\"{prefix}\"\nexport PATH=\"{bin}:$PATH\"\nexport DYLD_LIBRARY_PATH=\"{lib}:$DYLD_LIBRARY_PATH\"\n",
        prefix = prefix.display(),
        bin = bin_dir.display(),
        lib = lib_dir.display(),
    );

    let existing = std::fs::read_to_string(&zprofile).unwrap_or_default();
    // Remove old usrp-manager block
    let cleaned: String = {
        let mut skip = false;
        let mut lines = Vec::new();
        for line in existing.lines() {
            if line.starts_with("# usrp-manager:") {
                skip = true;
            }
            if !skip {
                lines.push(line);
            }
            if skip && line.is_empty() {
                skip = false;
            }
        }
        lines.join("\n")
    };

    std::fs::write(&zprofile, format!("{}\n{}", cleaned, snippet))?;
    Ok(())
}
