use crate::config::AppConfig;
use crate::detection::{
    jtag as jtag_det, network, system,
    uhd,
    uhd_versions,
};
use crate::images::{
    download::{download_with_progress, extract_zip, verify_sha256},
    github::{self, Release},
    local,
};
use crate::ui::prompts::{confirm, fuzzy_select_str, input, select_str};
use crate::ui::styles::{
    dim, err, info, ok, press_enter, print_section, step, warn,
};
use console::style;
use rust_i18n::t;
use which::which;

// ─────────────────────────────────────────────────────────────────────────────
//  MAIN MENU
// ─────────────────────────────────────────────────────────────────────────────

pub fn main_menu(config: &mut AppConfig) {
    loop {
        crate::ui::styles::print_header();

        let items = vec![
            t!("menu.diagnostic").to_string(),
            t!("menu.probe").to_string(),
            t!("menu.jtag_unbrick").to_string(),
            t!("menu.flash_network").to_string(),
            t!("menu.manage_images").to_string(),
            t!("menu.manage_drivers").to_string(),
            t!("menu.education").to_string(),
            t!("menu.settings").to_string(),
            t!("menu.exit").to_string(),
        ];

        let idx = match select_str(&t!("menu.main_prompt"), &items) {
            Ok(i) => i,
            Err(_) => break,
        };

        match idx {
            0 => diagnostic_menu(config),
            1 => crate::ui::probe::run_probe_menu(),
            2 => crate::jtag::run_jtag_unbrick(config),
            3 => crate::flash::run_flash_network(config),
            4 => images_menu(config),
            5 => drivers_menu(config),
            6 => crate::ui::education::show_education_menu(),
            7 => {
                settings_menu(config);
            }
            _ => {
                println!("\n  {} Goodbye!\n", style("👋").bold());
                break;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  DIAGNOSTIC
// ─────────────────────────────────────────────────────────────────────────────

pub fn diagnostic_menu(config: &AppConfig) {
    print_section(&t!("diagnostic.title"));

    // UHD
    step(&t!("diagnostic.checking_uhd"));
    let active_uhd = uhd::detect_active();
    match &active_uhd {
        Some(u) => ok(&t!("diagnostic.uhd_found", version = u.version.as_str())),
        None => err(&t!("diagnostic.uhd_not_found")),
    }

    // JTAG
    step(&t!("diagnostic.checking_jtag"));
    match jtag_det::find_djtgcfg() {
        Some(p) => ok(&format!(
            "{} ({})",
            t!("diagnostic.jtag_found"),
            p.display()
        )),
        None => warn(&t!("diagnostic.jtag_not_found")),
    }

    // Network scan
    step(&t!("diagnostic.checking_network"));
    let reachable = network::scan_defaults();
    if reachable.is_empty() {
        warn(&t!("diagnostic.usrp_not_found"));
        dim("  (Make sure Ethernet is connected, host IP is 192.168.10.1)");
    } else {
        for ip in &reachable {
            ok(&t!("diagnostic.usrp_found_at", ip = ip.as_str()));
            // Try UHD probe
            if let Some(u) = &active_uhd {
                if let Ok(uhd_find) = which("uhd_find_devices") {
                    if let Some(found) = network::probe_with_uhd(ip, &uhd_find) {
                        if let Some(model) = &found.model {
                            info(&format!("  Model: {}", model));
                        }
                        if let Some(ser) = &found.serial {
                            info(&format!("  Serial: {}", ser));
                        }
                        if let Some(fpga_ver) = &found.fpga_version {
                            info(&format!("  FPGA image version: {}", fpga_ver));
                            // Check mismatch
                            let uhd_major_minor = u.version.split('.').take(2).collect::<Vec<_>>().join(".");
                            let fpga_major_minor = fpga_ver.split('.').take(2).collect::<Vec<_>>().join(".");
                            if !uhd_major_minor.is_empty() && uhd_major_minor != fpga_major_minor {
                                warn(&t!(
                                    "diagnostic.version_mismatch",
                                    uhd_ver = u.version.as_str(),
                                    fpga_ver = fpga_ver.as_str()
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Local images summary
    println!();
    let local_imgs = local::list_local_images(&config.images_cache_dir);
    if local_imgs.is_empty() {
        dim(&format!(
            "No FPGA images in cache ({})",
            config.images_cache_dir.display()
        ));
        dim("  → Go to 'Manage FPGA Images' to download one.");
    } else {
        info(&format!(
            "FPGA image cache: {} version(s) available",
            local_imgs.len()
        ));
        for img in &local_imgs {
            dim(&format!("  v{}  ({} .bit files)", img.version, img.bitfiles.len()));
        }
    }

    // Admin status
    println!();
    if system::is_admin() {
        ok("Running with administrator / root privileges");
    } else {
        warn("Not running as administrator. Some operations (PATH update, install) may require re-running as admin.");
    }

    println!();
    if active_uhd.is_some() && !reachable.is_empty() {
        ok(&t!("diagnostic.all_ok"));
    }

    press_enter();
}

// ─────────────────────────────────────────────────────────────────────────────
//  MANAGE FPGA IMAGES
// ─────────────────────────────────────────────────────────────────────────────

pub fn images_menu(config: &AppConfig) {
    loop {
        print_section(&t!("images.title"));

        let actions = vec![
            t!("images.action_download").to_string(),
            t!("images.action_view_local").to_string(),
            t!("images.action_back").to_string(),
        ];

        let idx = match select_str(&t!("images.select_action"), &actions) {
            Ok(i) => i,
            Err(_) => break,
        };

        match idx {
            0 => images_download(config),
            1 => images_view_local(config),
            _ => break,
        }
    }
}

fn images_view_local(config: &AppConfig) {
    print_section(&t!("images.local_title"));
    let images = local::list_local_images(&config.images_cache_dir);
    if images.is_empty() {
        warn(&t!("images.none_local"));
        dim(&format!("Cache dir: {}", config.images_cache_dir.display()));
    } else {
        for img in &images {
            info(&format!("v{}  — {} .bit files", img.version, img.bitfiles.len()));
            for bf in &img.bitfiles {
                dim(&format!(
                    "   {}",
                    bf.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }
    press_enter();
}

fn images_download(config: &AppConfig) {
    step(&t!("images.fetching"));
    let releases = match github::fetch_releases() {
        Ok(r) => r,
        Err(e) => {
            err(&t!("images.fetch_fail"));
            dim(&format!("Error: {}", e));
            press_enter();
            return;
        }
    };

    // Filter releases that have FPGA images
    let with_images: Vec<&Release> = releases
        .iter()
        .filter(|r| r.images_zip().is_some())
        .collect();

    if with_images.is_empty() {
        warn("No FPGA image releases found.");
        press_enter();
        return;
    }

    let local_versions: Vec<String> = local::list_local_images(&config.images_cache_dir)
        .into_iter()
        .map(|i| i.version)
        .collect();

    let items: Vec<String> = with_images
        .iter()
        .map(|r| {
            let ver = r.version();
            let size = r.images_zip().map(|a| a.human_size()).unwrap_or_default();
            let already = if local_versions.contains(&ver) {
                format!("  {}", t!("images.already_downloaded"))
            } else {
                String::new()
            };
            format!("v{}  ({})  {}", ver, size, already)
        })
        .collect();

    let mut items_with_back = items.clone();
    items_with_back.push(t!("common.back").to_string());

    let idx = match fuzzy_select_str(&t!("images.select_version"), &items_with_back) {
        Ok(i) => i,
        Err(_) => return,
    };

    if idx >= with_images.len() {
        return;
    }

    let release = with_images[idx];
    let asset = release.images_zip().unwrap();
    let sha_asset = release.images_sha256();

    // Confirm
    match confirm(
        &t!("images.download_confirm", name = asset.name.as_str(), size = asset.human_size().as_str()),
        true,
    ) {
        Ok(true) => {}
        _ => return,
    }

    // Download
    let zip_path = config
        .images_cache_dir
        .join(format!("{}.zip", release.version()));

    step(&t!("images.downloading", name = asset.name.as_str()));
    if let Err(e) = download_with_progress(&asset.browser_download_url, &zip_path, &asset.name) {
        err(&format!("Download failed: {}", e));
        press_enter();
        return;
    }

    // Verify SHA256
    if let Some(sha_asset) = sha_asset {
        step(&t!("images.verifying"));
        // Download the sha256 file
        let sha_path = zip_path.with_extension("sha256");
        if download_with_progress(&sha_asset.browser_download_url, &sha_path, &sha_asset.name).is_ok() {
            if let Ok(sha_content) = std::fs::read_to_string(&sha_path) {
                // sha256 file usually: "hexhash  filename"
                // Find the specific hash line matching the asset name (e.g. uhd-images_x.y.z.zip)
                let expected = sha_content
                    .lines()
                    .find(|line| line.contains(&asset.name))
                    .and_then(|line| line.split_whitespace().next())
                    .unwrap_or("")
                    .to_string();
                if verify_sha256(&zip_path, &expected) {
                    ok(&t!("images.verify_ok"));
                } else {
                    err(&t!("images.verify_fail"));
                    press_enter();
                    return;
                }
            }
        }
    }

    // Extract
    step(&t!("images.extracting"));
    let dest_dir = config
        .images_cache_dir
        .join(format!("uhd-images_{}", release.version()));
    if let Err(e) = extract_zip(&zip_path, &dest_dir) {
        err(&format!("Extraction failed: {}", e));
        press_enter();
        return;
    }

    // Remove zip to save space
    let _ = std::fs::remove_file(&zip_path);

    ok(&t!("images.done", path = dest_dir.display().to_string().as_str()));
    press_enter();
}

// ─────────────────────────────────────────────────────────────────────────────
//  MANAGE DRIVERS
// ─────────────────────────────────────────────────────────────────────────────

pub fn drivers_menu(config: &mut AppConfig) {
    loop {
        print_section(&t!("drivers.title"));

        // Show active UHD
        match uhd::detect_active() {
            Some(u) => ok(&t!("drivers.active", version = u.version.as_str())),
            None => warn(&t!("drivers.no_active")),
        }
        println!();

        let actions = vec![
            t!("drivers.action_install").to_string(),
            t!("drivers.action_activate").to_string(),
            t!("drivers.action_back").to_string(),
        ];

        let idx = match select_str(&t!("drivers.select_action"), &actions) {
            Ok(i) => i,
            Err(_) => break,
        };

        match idx {
            0 => drivers_install(config),
            1 => drivers_activate(config),
            _ => break,
        }
    }
}

fn drivers_install(config: &AppConfig) {
    print_section("Install UHD Driver");

    // Fetch available versions from GitHub
    step(&t!("images.fetching"));
    let releases = match github::fetch_releases() {
        Ok(r) => r,
        Err(e) => {
            err(&format!("{}: {}", t!("images.fetch_fail"), e));
            press_enter();
            return;
        }
    };

    let items: Vec<String> = releases
        .iter()
        .map(|r| format!("v{}  ({})", r.version(), r.published_at.get(..10).unwrap_or("")))
        .collect();

    let mut items_with_back = items.clone();
    items_with_back.push(t!("common.back").to_string());

    let idx = match fuzzy_select_str(&t!("drivers.select_version"), &items_with_back) {
        Ok(i) => i,
        Err(_) => return,
    };

    if idx >= releases.len() {
        return;
    }

    let release = &releases[idx];
    let version = release.version();

    #[cfg(target_os = "windows")]
    {
        drivers_install_windows(config, release, &version);
    }
    #[cfg(target_os = "linux")]
    {
        drivers_install_linux(config, &version);
    }
    #[cfg(target_os = "macos")]
    {
        drivers_install_macos(config, &version);
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        warn(&t!("common.not_implemented"));
        press_enter();
    }
}

#[cfg(target_os = "windows")]
fn drivers_install_windows(config: &AppConfig, release: &Release, version: &str) {
    let installer_asset = match release.windows_installer() {
        Some(a) => a.clone(),
        None => {
            warn("No Windows installer found for this release. Try building from source.");
            press_enter();
            return;
        }
    };

    let methods = vec![
        format!("Download installer from GitHub ({})", installer_asset.human_size()),
        t!("common.back").to_string(),
    ];
    let idx = match select_str(&t!("drivers.select_method"), &methods) {
        Ok(i) => i,
        Err(_) => return,
    };
    if idx != 0 {
        return;
    }

    let target_dir = config.uhd_versions_dir.join(format!("UHD-{}", version));
    info(&format!("{} {}", t!("drivers.install_dir"), target_dir.display()));

    let exe_path = config.images_cache_dir.join(&installer_asset.name);
    step(&t!("images.downloading", name = installer_asset.name.as_str()));
    if let Err(e) = crate::images::download::download_with_progress(
        &installer_asset.browser_download_url,
        &exe_path,
        &installer_asset.name,
    ) {
        err(&format!("Download failed: {}", e));
        press_enter();
        return;
    }

    step(&format!("Installing UHD {} to {}...", version, target_dir.display()));
    if let Err(e) = crate::drivers::windows::install_from_exe(&exe_path, &target_dir) {
        err(&format!("Install failed: {}", e));
        press_enter();
        return;
    }

    ok(&format!("UHD {} installed to {}", version, target_dir.display()));

    if let Ok(true) = confirm(&format!("Activate UHD {} now?", version), true) {
        let bin_dir = target_dir.join("bin");
        match crate::drivers::windows::set_active_version(&bin_dir) {
            Ok(_) => ok(&t!("drivers.activated", version = version)),
            Err(e) => err(&format!("Failed to update PATH: {}", e)),
        }
    }
    press_enter();
}

#[cfg(target_os = "linux")]
fn drivers_install_linux(config: &AppConfig, version: &str) {
    let pm = crate::drivers::linux::detect_package_manager();

    let mut methods = vec![];
    if pm != crate::drivers::linux::PackageManager::None {
        methods.push(format!("{} (Recomendado - Rápido, ~10s)", t!("drivers.method_package")));
    }
    methods.push(format!("{} (Lento, ~30-60 min, alto uso de RAM/CPU)", t!("drivers.method_build")));
    methods.push(t!("common.back").to_string());

    let idx = match select_str(&t!("drivers.select_method"), &methods) {
        Ok(i) => i,
        Err(_) => return,
    };

    if idx == methods.len() - 1 {
        return;
    }

    let use_package = pm != crate::drivers::linux::PackageManager::None && idx == 0;

    if use_package {
        warn(&t!("drivers.package_version_note"));
        if let Err(e) = crate::drivers::linux::install_from_package(&pm) {
            err(&format!("Package install failed: {}", e));
            press_enter();
            return;
        }
        ok("UHD installed via package manager.");
    } else {
        // Build from source
        let tag = format!("v{}", version);
        let prefix = config.uhd_versions_dir.join(format!("uhd-{}", version));
        step(&t!("drivers.building", version = version));

        if let Err(e) = crate::drivers::linux::build_from_source(&tag, &prefix) {
            err(&format!("Build failed: {}", e));
            press_enter();
            return;
        }

        // Register with update-alternatives
        if let Err(e) = crate::drivers::linux::register_with_update_alternatives(&prefix, version) {
            warn(&format!("update-alternatives registration failed: {}", e));
        }

        ok(&format!("UHD {} built and installed to {}", version, prefix.display()));

        if let Ok(true) = confirm(&format!("Activate UHD {} now?", version), true) {
            if let Err(e) = crate::drivers::linux::set_active_version(&prefix) {
                err(&format!("Failed to update ~/.bashrc: {}", e));
            } else {
                ok(&t!("drivers.activated", version = version));
            }
        }
    }
    press_enter();
}

#[cfg(target_os = "macos")]
fn drivers_install_macos(config: &AppConfig, version: &str) {
    let methods = vec![
        "Install via Homebrew (brew install uhd)".to_string(),
        t!("drivers.method_build").to_string(),
        t!("common.back").to_string(),
    ];
    let idx = match select_str(&t!("drivers.select_method"), &methods) {
        Ok(i) => i,
        Err(_) => return,
    };

    match idx {
        0 => {
            if let Err(e) = crate::drivers::macos::install_via_brew() {
                err(&format!("Homebrew install failed: {}", e));
            } else {
                ok("UHD installed via Homebrew.");
            }
        }
            2 => {
            let tag = format!("v{}", version);
            let prefix = config.uhd_versions_dir.join(format!("uhd-{}", version));
            step(&t!("drivers.building", version = version));
            if let Err(e) = crate::drivers::macos::build_from_source(&tag, &prefix) {
                err(&format!("Build failed: {}", e));
            } else {
                ok(&format!("UHD {} built to {}", version, prefix.display()));
                if let Ok(true) = confirm(&format!("Activate UHD {} now?", version), true) {
                    if let Err(e) = crate::drivers::macos::set_active_version(&prefix) {
                        err(&format!("Failed to update ~/.zprofile: {}", e));
                    } else {
                        ok(&t!("drivers.activated", version = version));
                    }
                }
            }
        }
        _ => return,
    }
    press_enter();
}

fn drivers_activate(config: &AppConfig) {
    print_section(&t!("drivers.installed_title"));

    let active = uhd::detect_active();
    let active_bin = active.as_ref().map(|a| a.bin_dir.as_path());

    let installed =
        uhd_versions::detect_installed_versions(&config.uhd_versions_dir, active_bin);

    if installed.is_empty() {
        warn(&t!("drivers.none_installed"));
        dim(&format!("Scan dir: {}", config.uhd_versions_dir.display()));
        press_enter();
        return;
    }

    let items: Vec<String> = installed
        .iter()
        .map(|v| {
            let marker = if v.is_active { " ★ [active]" } else { "" };
            format!("UHD {}  —  {}{}", v.version, v.install_dir.display(), marker)
        })
        .collect();

    let mut items_with_back = items.clone();
    items_with_back.push(t!("common.back").to_string());

    let idx = match fuzzy_select_str(&t!("drivers.action_activate"), &items_with_back) {
        Ok(i) => i,
        Err(_) => return,
    };

    if idx >= installed.len() {
        return;
    }

    let selected = &installed[idx];
    step(&t!("drivers.activating", version = selected.version.as_str()));

    #[cfg(target_os = "windows")]
    {
        step(&t!("drivers.activate_win"));
        match crate::drivers::windows::set_active_version(&selected.bin_dir) {
            Ok(_) => ok(&t!("drivers.activated", version = selected.version.as_str())),
            Err(e) => err(&format!("Failed: {}", e)),
        }
    }
    #[cfg(target_os = "linux")]
    {
        step(&t!("drivers.activate_linux"));
        match crate::drivers::linux::set_active_version(&selected.install_dir) {
            Ok(_) => ok(&t!("drivers.activated", version = selected.version.as_str())),
            Err(e) => err(&format!("Failed: {}", e)),
        }
    }
    #[cfg(target_os = "macos")]
    {
        match crate::drivers::macos::set_active_version(&selected.install_dir) {
            Ok(_) => ok(&t!("drivers.activated", version = selected.version.as_str())),
            Err(e) => err(&format!("Failed: {}", e)),
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        warn(&t!("common.not_implemented"));
    }

    press_enter();
}

// ─────────────────────────────────────────────────────────────────────────────
//  SETTINGS
// ─────────────────────────────────────────────────────────────────────────────

pub fn settings_menu(config: &mut AppConfig) {
    loop {
        print_section(&t!("settings.title"));

        let settings_items = vec![
            format!(
                "{}  ({})",
                t!("settings.language"),
                if config.locale == "es" { "Español" } else { "English" }
            ),
            format!(
                "{}  ({})",
                t!("settings.images_dir"),
                config.images_cache_dir.display()
            ),
            format!(
                "{}  ({})",
                t!("settings.uhd_versions_dir"),
                config.uhd_versions_dir.display()
            ),
            t!("common.back").to_string(),
        ];

        let idx = match select_str(&t!("settings.select"), &settings_items) {
            Ok(i) => i,
            Err(_) => break,
        };

        match idx {
            0 => {
                // Language
                let lang_items = vec![
                    t!("settings.lang_en").to_string(),
                    t!("settings.lang_es").to_string(),
                ];
                if let Ok(lang_idx) = select_str(&t!("settings.select_language"), &lang_items) {
                    let new_locale = if lang_idx == 0 { "en" } else { "es" };
                    config.locale = new_locale.to_string();
                    rust_i18n::set_locale(new_locale);
                    if let Err(e) = config.save() {
                        err(&format!("Could not save config: {}", e));
                    } else {
                        ok(&t!("settings.saved"));
                    }
                }
            }
            1 => {
                // Images dir
                if let Ok(new_path) =
                    input(&t!("settings.images_dir"), Some(&config.images_cache_dir.display().to_string()))
                {
                    config.images_cache_dir = std::path::PathBuf::from(new_path);
                    config.ensure_dirs();
                    let _ = config.save();
                    ok(&t!("settings.saved"));
                }
            }
            2 => {
                // UHD versions dir
                if let Ok(new_path) =
                    input(&t!("settings.uhd_versions_dir"), Some(&config.uhd_versions_dir.display().to_string()))
                {
                    config.uhd_versions_dir = std::path::PathBuf::from(new_path);
                    config.ensure_dirs();
                    let _ = config.save();
                    ok(&t!("settings.saved"));
                }
            }
            _ => break,
        }
    }
}
