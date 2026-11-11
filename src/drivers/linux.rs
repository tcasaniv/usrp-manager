#[cfg(target_os = "linux")]
use anyhow::{bail, Result};
#[cfg(target_os = "linux")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use crate::executor::{run_visible, run_visible_in_dir};

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
    // Use /var/tmp or user cache on Linux instead of /tmp (which is tmpfs in RAM and limited)
    let tmp = if std::path::Path::new("/var/tmp").exists() {
        std::path::PathBuf::from("/var/tmp").join(format!("uhd-build-{}", tag))
    } else {
        dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("usrp-manager")
            .join(format!("uhd-build-{}", tag))
    };
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

    let r = run_visible_in_dir(
        "cmake",
        &[
            "..",
            &format!("-DCMAKE_INSTALL_PREFIX={}", prefix.display()),
            "-DCMAKE_BUILD_TYPE=Release",
            "-DENABLE_PYTHON_API=OFF",
        ],
        &build_dir,
    );
    if !r.success {
        bail!("cmake configure failed");
    }

    let nproc = num_cpus();
    let r = run_visible_in_dir("make", &["-j", &nproc], &build_dir);
    if !r.success {
        bail!("make failed");
    }

    let r = run_visible_in_dir("sudo", &["make", "install"], &build_dir);
    if !r.success {
        bail!("make install failed");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn num_cpus() -> String {
    let cores: usize = crate::executor::run_silent("nproc", &[])
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(4);

    // Read total RAM from /proc/meminfo in kB to avoid OOM crashes (each g++ compiler worker needs ~1.5 - 2.0 GB)
    let ram_gb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|meminfo| {
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return Some(kb / (1024 * 1024));
                        }
                    }
                }
            }
            None
        })
        .unwrap_or(8);

    // Allocate at most 1 worker per 2 GB of RAM, capped by available CPU cores and a minimum of 1
    let safe_by_ram = (ram_gb / 2).max(1);
    let safe_workers = cores.min(safe_by_ram);

    eprintln!(
        "  [36m● Parallel build jobs:[0m {} (CPU cores: {}, Available RAM: ~{} GB - limited to prevent OOM)",
        safe_workers, cores, ram_gb
    );

    safe_workers.to_string()
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

    // 1. Update dynamic linker cache (/etc/ld.so.conf.d/usrp-manager-uhd.conf + ldconfig)
    let lib_str = lib_dir.to_str().unwrap_or("");
    if !lib_str.is_empty() {
        let conf_content = format!("{}\n", lib_str);
        let write_ok = if crate::detection::system::is_admin() {
            std::fs::write("/etc/ld.so.conf.d/usrp-manager-uhd.conf", &conf_content).is_ok()
        } else {
            let r = run_visible("sudo", &["sh", "-c", &format!("echo '{}' > /etc/ld.so.conf.d/usrp-manager-uhd.conf", lib_str)]);
            r.success
        };
        if write_ok {
            let _ = run_visible("sudo", &["ldconfig"]);
        }
    }

    // 2. Update alternatives for binaries if installed in /usr/bin
    let tools = [
        "uhd_find_devices",
        "uhd_config_info",
        "uhd_image_loader",
        "uhd_usrp_probe",
    ];
    for tool in &tools {
        let path = bin_dir.join(tool);
        if path.exists() {
            let _ = run_visible(
                "sudo",
                &[
                    "update-alternatives",
                    "--set",
                    tool,
                    path.to_str().unwrap_or(""),
                ],
            );
        }
    }

    // 3. Update ~/.bashrc of current user AND SUDO_USER if invoked via sudo
    let mut target_bashrcs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        target_bashrcs.push(home.join(".bashrc"));
    }
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        let user_home = PathBuf::from(format!("/home/{}", sudo_user));
        let user_bashrc = user_home.join(".bashrc");
        if user_bashrc.exists() && !target_bashrcs.contains(&user_bashrc) {
            target_bashrcs.push(user_bashrc);
        }
    }

    let snippet = format!(
        "\n# usrp-manager: active UHD\nexport UHD_DIR=\"{prefix}\"\nexport PATH=\"{bin}:$PATH\"\nexport LD_LIBRARY_PATH=\"{lib}:$LD_LIBRARY_PATH\"\nexport PKG_CONFIG_PATH=\"{pkg}:$PKG_CONFIG_PATH\"\nexport PYTHONPATH=\"{py}:$PYTHONPATH\"\n",
        prefix = prefix.display(),
        bin = bin_dir.display(),
        lib = lib_dir.display(),
        pkg = pkg_dir.display(),
        py = python_dir.display(),
    );

    for bashrc in target_bashrcs {
        let existing = std::fs::read_to_string(&bashrc).unwrap_or_default();
        let cleaned: Vec<&str> = existing
            .lines()
            .filter(|l| !l.starts_with("# usrp-manager") && !l.contains("UHD_DIR") && !l.contains("active UHD"))
            .collect();
        let mut new_content = cleaned.join("\n");
        new_content.push_str(&snippet);
        let _ = std::fs::write(&bashrc, new_content);
    }

    Ok(())
}
