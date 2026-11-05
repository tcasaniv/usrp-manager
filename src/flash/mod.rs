use crate::config::AppConfig;
use crate::detection::network::{ping, probe_with_uhd};
use crate::executor::run_visible;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{err, info, ok, press_enter, print_section, step, warn};
use rust_i18n::t;
use which::which;

pub fn run_flash_network(config: &AppConfig) {
    print_section(&t!("flash.title"));
    println!("  {}", t!("flash.intro").replace("\\n", "\n  "));
    println!();

    // 1. Check uhd_image_loader in PATH
    let uhd_loader = match which("uhd_image_loader") {
        Ok(p) => p,
        Err(_) => {
            err(&t!("flash.no_uhd"));
            press_enter();
            return;
        }
    };

    // 2. Input IP
    let ip = match input(&t!("flash.enter_ip"), Some("192.168.10.2")) {
        Ok(s) => s,
        Err(_) => return,
    };

    // 3. Ping
    step(&t!("flash.pinging", ip = ip.as_str()));
    if ping(&ip) {
        ok(&t!("flash.ping_ok", ip = ip.as_str()));
    } else {
        err(&t!("flash.ping_fail", ip = ip.as_str()));
        println!("  {}", t!("flash.ping_fix1"));
        println!("  {}", t!("flash.ping_fix2"));
        println!("  {}", t!("flash.ping_fix3"));
        press_enter();
        return;
    }

    // 4. Find UHD find_devices
    if let Ok(uhd_find) = which("uhd_find_devices") {
        step(&t!("flash.finding"));
        let found = probe_with_uhd(&ip, &uhd_find);
        match found {
            Some(u) => {
                ok(&format!(
                    "USRP found: {} (serial: {})",
                    u.model.as_deref().unwrap_or("?"),
                    u.serial.as_deref().unwrap_or("?")
                ));
                if let Some(fpga_ver) = &u.fpga_version {
                    info(&format!("FPGA version: {}", fpga_ver));
                }
            }
            None => {
                warn(&t!("flash.find_fail").replace("\\n", "\n  "));
                info(&t!("flash.mismatch_hint"));
                // Don't abort — user might still want to try flashing
            }
        }
    }

    println!();

    // 5. Select variant
    let variants = vec![
        t!("jtag.variant_hg").to_string(),
        t!("jtag.variant_xg").to_string(),
    ];
    let variant_idx = match select_str(&t!("flash.select_variant"), &variants) {
        Ok(i) => i,
        Err(_) => return,
    };
    let variant = if variant_idx == 0 { "HG" } else { "XG" };

    // Need to know the USRP model (X300 or X310) to pick the right image name
    let model_choices = vec!["X300".to_string(), "X310".to_string()];
    let model_idx = match select_str("USRP model:", &model_choices) {
        Ok(i) => i,
        Err(_) => return,
    };
    let usrp_model = if model_idx == 0 { "X300" } else { "X310" };

    // 6. Find local bitstream
    let bitfile = crate::images::local::find_bitstream(&config.images_cache_dir, usrp_model, variant);

    // Also check Downloads
    let bitfile = match bitfile {
        Some(b) => b,
        None => {
            // Try Downloads
            let found_in_downloads = crate::jtag::find_in_downloads_pub(usrp_model, variant);
            match found_in_downloads.into_iter().next() {
                Some(b) => b,
                None => {
                    err(&t!("jtag.no_bitstream"));
                    info("Go to 'Manage FPGA Images' to download one.");
                    press_enter();
                    return;
                }
            }
        }
    };

    info(&format!(
        "Bitstream: {}",
        bitfile.file_name().unwrap_or_default().to_string_lossy()
    ));

    // 7. Confirm + flash
    println!();
    match confirm(&t!("flash.confirm_flash"), false) {
        Ok(true) => {}
        _ => return,
    }

    step(&t!("flash.flashing"));

    let addr_arg = format!("--args=addr={}", ip);
    let fpga_arg = format!(
        "--fpga-path={}",
        bitfile.display()
    );
    let result = run_visible(
        uhd_loader.to_str().unwrap_or("uhd_image_loader"),
        &[&addr_arg, &fpga_arg],
    );

    if result.success {
        ok(&t!("flash.success"));
        println!();
        warn(&t!("flash.reboot_reminder"));
    } else {
        err("Flash failed. See output above.");
    }

    press_enter();
}
