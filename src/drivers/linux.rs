#[cfg(target_os = "linux")]
use anyhow::{bail, Result};
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use crate::executor::run_visible;

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    None,
}

#[cfg(target_os = "linux")]
pub fn detect_package_manager() -> PackageManager {
    use which::which;
    if which("apt").is_ok() {
        PackageManager::Apt
    } else if which("dnf").is_ok() {
        PackageManager::Dnf
    } else if which("pacman").is_ok() {
        PackageManager::Pacman
    } else {
        PackageManager::None
    }
}

#[cfg(target_os = "linux")]
pub fn install_from_package(pm: &PackageManager) -> Result<()> {
    match pm {
        PackageManager::Apt => {
            run_visible("sudo", &["apt", "update"]);
            let r = run_visible("sudo", &["apt", "install", "-y", "libuhd-dev", "uhd-host"]);
            if !r.success {
                bail!("apt install failed");
            }
        }
        PackageManager::Dnf => {
            let r = run_visible("sudo", &["dnf", "install", "-y", "uhd", "uhd-devel"]);
            if !r.success {
                bail!("dnf install failed");
            }
        }
        PackageManager::Pacman => {
            let r = run_visible("sudo", &["pacman", "-Sy", "--noconfirm", "uhd"]);
            if !r.success {
                bail!("pacman install failed");
            }
        }
        PackageManager::None => bail!("No supported package manager found"),
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn build_from_source(tag: &str, prefix: &Path) -> Result<()> {
    let tmp = std::env::temp_dir().join(format!("uhd-build-{}", tag));
    std::fs::create_dir_all(&tmp)?;

    let repo_dir = tmp.join("uhd-src");

    // Clone or update
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

    let r = run_visible(
        "cmake",
        &[
            "..",
            &format!("-DCMAKE_INSTALL_PREFIX={}", prefix.display()),
            "-DCMAKE_BUILD_TYPE=Release",
            "-DENABLE_PYTHON_API=OFF",
        ],
    );
    if !r.success {
        bail!("cmake configure failed");
    }

    let nproc = num_cpus();
    let r = run_visible("make", &["-j", &nproc]);
    if !r.success {
        bail!("make failed");
    }

    let r = run_visible("sudo", &["make", "install"]);
    if !r.success {
        bail!("make install failed");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn num_cpus() -> String {
    crate::executor::run_silent("nproc", &[])
        .unwrap_or_else(|| "4".to_string())
        .trim()
        .to_string()
}

#[cfg(target_os = "linux")]
pub fn register_with_update_alternatives(prefix: &Path, ver: &str) -> Result<()> {
    let priority = version_to_priority(ver);
    let bin_dir = prefix.join("bin");
    let tools = [
        "uhd_find_devices",
        "uhd_config_info",
        "uhd_image_loader",
        "uhd_usrp_probe",
    ];
    for tool in &tools {
        let link = format!("/usr/bin/{}", tool);
        let path = bin_dir.join(tool);
        if path.exists() {
            run_visible(
                "sudo",
                &[
                    "update-alternatives",
                    "--install",
                    &link,
                    tool,
                    path.to_str().unwrap_or(""),
                    &priority.to_string(),
                ],
            );
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn version_to_priority(ver: &str) -> u32 {
    // "4.10.0.0" → 410
    let parts: Vec<u32> = ver.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() >= 2 {
        parts[0] * 100 + parts[1]
    } else {
        100
    }
}

#[cfg(target_os = "linux")]
pub fn set_active_version(prefix: &Path) -> Result<()> {
    let bin_dir = prefix.join("bin");
    let lib_dir = prefix.join("lib");
    let pkg_dir = lib_dir.join("pkgconfig");
    let python_dir = lib_dir.join("python3/dist-packages");

    // Write to ~/.bashrc
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    let bashrc = home.join(".bashrc");
    let snippet = format!(
        "\n# usrp-manager: active UHD\nexport UHD_DIR=\"{prefix}\"\nexport PATH=\"{bin}:$PATH\"\nexport LD_LIBRARY_PATH=\"{lib}:$LD_LIBRARY_PATH\"\nexport PKG_CONFIG_PATH=\"{pkg}:$PKG_CONFIG_PATH\"\nexport PYTHONPATH=\"{py}:$PYTHONPATH\"\n",
        prefix = prefix.display(),
        bin = bin_dir.display(),
        lib = lib_dir.display(),
        pkg = pkg_dir.display(),
        py = python_dir.display(),
    );

    // Remove old usrp-manager block if present
    let existing = std::fs::read_to_string(&bashrc).unwrap_or_default();
    let cleaned: String = existing
        .lines()
        .skip_while(|l| l.starts_with("# usrp-manager"))
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&bashrc, format!("{}\n{}", cleaned, snippet))?;
    Ok(())
}
