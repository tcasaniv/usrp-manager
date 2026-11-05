use crate::config::AppConfig;
use crate::detection::jtag::{self, FpgaModel};
use crate::detection::system::OsType;
use crate::images::local::{find_bitstream_in_set, list_local_images};
use crate::ui::prompts::{confirm, select_str};
use crate::ui::styles::{err, info, ok, press_enter, print_section, step, warn};
use rust_i18n::t;

pub fn run_jtag_unbrick(config: &AppConfig) {
    print_section(&t!("jtag.title"));
    println!("  {}", t!("jtag.intro").replace("\\n", "\n  "));
    println!();

    // 1. Find djtgcfg
    step(&t!("diagnostic.checking_jtag"));
    let djtgcfg = match jtag::find_djtgcfg() {
        Some(p) => {
            ok(&format!("{} ({})", t!("jtag_found"), p.display()));
            p
        }
        None => {
            err(&t!("jtag.no_djtgcfg"));
            println!();
            info(&t!("jtag.install_guide_title"));
            let os = crate::detection::system::get_os();
            match os {
                OsType::Windows => {
                    println!();
                    for line in t!("jtag.install_guide_win").lines() {
                        println!("    {}", line);
                    }
                }
                OsType::Linux => {
                    println!();
                    for line in t!("jtag.install_guide_linux").lines() {
                        println!("    {}", line);
                    }
                }
                OsType::MacOs => {
                    println!();
                    for line in t!("jtag.install_guide_mac").lines() {
                        println!("    {}", line);
                    }
                }
                OsType::Other => {}
            }
            press_enter();
            return;
        }
    };

    // 2. Enumerate JTAG devices
    step(&t!("jtag.enumerating"));
    let devices = jtag::enumerate(&djtgcfg);

    if devices.is_empty() {
        err(&t!("jtag.no_devices").replace("\\n", "\n  "));
        press_enter();
        return;
    }

    // 3. Select device (if multiple)
    let device_name = if devices.len() == 1 {
        devices[0].name.clone()
    } else {
        let items: Vec<String> = devices
            .iter()
            .map(|d| format!("{} — {} [{}]", d.name, d.product, d.serial))
            .collect();
        let idx = match select_str(&t!("jtag.select_device"), &items) {
            Ok(i) => i,
            Err(_) => return,
        };
        devices[idx].name.clone()
    };

    // 4. Init + detect FPGA model
    step(&t!("jtag.initializing"));
    let fpga_model = match jtag::init_and_detect(&djtgcfg, &device_name) {
        Some(m) => {
            match &m {
                FpgaModel::Unknown(id) => {
                    warn(&t!("jtag.fpga_unknown", id = id));
                    press_enter();
                    return;
                }
                _ => {
                    ok(&t!(
                        "jtag.fpga_detected",
                        fpga = m.fpga_name(),
                        usrp = m.usrp_model()
                    ));
                }
            }
            m
        }
        None => {
            err("Failed to initialize JTAG scan chain.");
            warn(&t!("jtag.already_bricked_tip"));
            press_enter();
            return;
        }
    };

    let usrp_model = fpga_model.usrp_model().to_string();

    // 5. Select FPGA variant
    let variants = vec![
        t!("jtag.variant_hg").to_string(),
        t!("jtag.variant_xg").to_string(),
    ];
    let variant_idx = match select_str(&t!("jtag.select_variant"), &variants) {
        Ok(i) => i,
        Err(_) => return,
    };
    let variant = if variant_idx == 0 { "HG" } else { "XG" };

    // 6. Find bitstream
    let local_images = list_local_images(&config.images_cache_dir);

    // Also check the well-known Downloads folder
    let downloads_images = find_in_downloads(&usrp_model, variant);

    let bitfile = select_bitstream(config, &local_images, &usrp_model, variant, &downloads_images);

    let bitfile = match bitfile {
        Some(b) => b,
        None => {
            err(&t!("jtag.no_bitstream"));
            info("Go to 'Manage FPGA Images' to download one.");
            press_enter();
            return;
        }
    };

    // 7. Confirm + program
    info(&format!(
        "Bitstream: {}",
        bitfile.file_name().unwrap_or_default().to_string_lossy()
    ));
    println!();
    match confirm(&t!("jtag.programming"), true) {
        Ok(true) => {}
        _ => return,
    }

    step(&t!("jtag.programming"));
    let success = jtag::program(&djtgcfg, &device_name, &bitfile);

    if success {
        ok(&t!("jtag.success"));
        println!();
        info(&t!("jtag.success_detail").replace("\\n", "\n  "));
        println!();

        // 8. Offer permanent flash
        if let Ok(true) = confirm(&t!("jtag.offer_permanent"), true) {
            crate::flash::run_flash_network(config);
        }
    } else {
        err("JTAG programming failed.");
        warn(&t!("jtag.already_bricked_tip"));
        press_enter();
    }
}

pub fn find_in_downloads_pub(usrp_model: &str, variant: &str) -> Vec<std::path::PathBuf> {
    find_in_downloads(usrp_model, variant)
}

fn find_in_downloads(usrp_model: &str, variant: &str) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    if let Some(home) = dirs::home_dir() {
        let downloads = home.join("Downloads");
        scan_for_bit(&downloads, usrp_model, variant, &mut found);
    }
    found
}

fn scan_for_bit(
    dir: &std::path::Path,
    usrp_model: &str,
    variant: &str,
    result: &mut Vec<std::path::PathBuf>,
) {
    let expected = format!(
        "usrp_{}_fpga_{}.bit",
        usrp_model.to_lowercase(),
        variant.to_uppercase()
    );
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                scan_for_bit(&p, usrp_model, variant, result);
            } else if p
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase() == expected)
                .unwrap_or(false)
            {
                result.push(p);
            }
        }
    }
}

fn select_bitstream(
    _config: &AppConfig,
    local_images: &[crate::images::local::LocalImageSet],
    usrp_model: &str,
    variant: &str,
    extra_paths: &[std::path::PathBuf],
) -> Option<std::path::PathBuf> {
    let mut options: Vec<String> = Vec::new();
    let mut paths: Vec<std::path::PathBuf> = Vec::new();

    // From managed cache
    for img_set in local_images {
        if let Some(bf) = find_bitstream_in_set(img_set, usrp_model, variant) {
            options.push(format!(
                "[cache v{}]  {}",
                img_set.version,
                bf.file_name().unwrap_or_default().to_string_lossy()
            ));
            paths.push(bf);
        }
    }

    // From Downloads / auto-found paths
    for p in extra_paths {
        let display = format!("[found]  {}", p.display());
        if !paths.contains(p) {
            options.push(display);
            paths.push(p.clone());
        }
    }

    if options.is_empty() {
        return None;
    }

    options.push(t!("common.back").to_string());

    let idx = match select_str(&t!("jtag.select_bitstream"), &options) {
        Ok(i) => i,
        Err(_) => return None,
    };

    if idx == options.len() - 1 {
        return None;
    }

    paths.get(idx).cloned()
}
