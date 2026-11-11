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
pub fn check_build_prerequisites() -> Result<Vec<String>> {
    use which::which;
    let required = vec!["cmake", "make", "g++", "git"];
    let missing: Vec<String> = required
        .iter()
        .filter(|cmd| which(cmd).is_err())
        .map(|s| s.to_string())
        .collect();
    Ok(missing)
}

#[cfg(target_os = "linux")]
pub fn install_build_deps(pm: &PackageManager) -> Result<()> {
    match pm {
        PackageManager::Apt => {
            let r = run_visible("sudo", &[
                "apt", "install", "-y",
                "cmake", "build-essential", "g++",
                "libboost-all-dev", "libusb-1.0-0-dev",
                "python3-mako", "python3-numpy",
                "python3-requests", "python3-setuptools",
                "doxygen",
            ]);
            if !r.success {
                bail!("Failed to install build dependencies via apt");
            }
        }
        PackageManager::Dnf => {
            let r = run_visible("sudo", &[
                "dnf", "install", "-y",
                "cmake", "gcc-c++", "make",
                "boost-devel", "libusb1-devel",
                "python3-mako", "python3-numpy",
                "python3-requests", "python3-docutils",
                "doxygen",
            ]);
            if !r.success {
                bail!("Failed to install build dependencies via dnf");
            }
        }
        PackageManager::Pacman => {
            let r = run_visible("sudo", &[
                "pacman", "-Sy", "--noconfirm",
                "cmake", "base-devel", "boost",
                "libusb", "python-mako", "python-numpy",
                "python-requests", "doxygen",
            ]);
            if !r.success {
                bail!("Failed to install build dependencies via pacman");
            }
        }
        PackageManager::None => {
            bail!("No supported package manager found to install build dependencies");
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn build_from_source(tag: &str, prefix: &Path) -> Result<()> {
    // Check build prerequisites before doing anything
    let missing = check_build_prerequisites()?;
    if !missing.is_empty() {
        let missing_str = missing.join(", ");
        eprintln!();
        eprintln!("  \x1b[33m⚠  Missing build tools: {}\x1b[0m", missing_str);
        eprintln!();

        let pm = detect_package_manager();
        if pm != PackageManager::None {
            eprintln!("  The following dependencies will be installed automatically:");
            match &pm {
                PackageManager::Apt => eprintln!("  \x1b[33m● sudo apt install -y cmake build-essential g++ libboost-all-dev libusb-1.0-0-dev python3-mako ...\x1b[0m"),
                PackageManager::Dnf => eprintln!("  \x1b[33m● sudo dnf install -y cmake gcc-c++ make boost-devel libusb1-devel python3-mako ...\x1b[0m"),
                PackageManager::Pacman => eprintln!("  \x1b[33m● sudo pacman -Sy cmake base-devel boost libusb python-mako ...\x1b[0m"),
                _ => {}
            }
            eprintln!();

            let proceed = dialoguer::Confirm::new()
                .with_prompt("Install build dependencies now?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if proceed {
                install_build_deps(&pm)?;
                let still_missing = check_build_prerequisites()?;
                if !still_missing.is_empty() {
                    bail!("Still missing after install: {}. Please install them manually.", still_missing.join(", "));
                }
            } else {
                bail!("Cannot build without: {}. Please install them manually and retry.", missing_str);
            }
        } else {
            bail!("Missing build tools ({}) and no package manager found. Install them manually.", missing_str);
        }
    }
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
