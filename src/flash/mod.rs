use crate::config::AppConfig;
use crate::detection::network::ping;
use crate::executor::run_visible;
use crate::images::local::list_local_images;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{dim, err, info, ok, press_enter, print_section, step, warn};
use console::style;
use rust_i18n::t;
use which::which;

pub fn run_flash_network(config: &AppConfig) {
    print_section(&t!("flash.title"));
    println!(
        "\n  {}\n  {}\n",
        style(t!("flash.what_title")).cyan().bold(),
        t!("flash.what_body")
    );
    warn(&t!("flash.caution_unplug"));
    println!();

    // 1. Check uhd_image_loader in PATH
    let uhd_loader = match which("uhd_image_loader") {
        Ok(p) => p,
        Err(_) => {
            err(&t!("flash.no_uhd"));
            dim(&format!("  {}", t!("flash.mismatch_hint")));
            press_enter();
            return;
        }
    };

    // 2. Select Connection Mode
    let mode_options = vec![
        t!("flash.mode_autodetect").to_string(),
        t!("flash.mode_manual_ip").to_string(),
        t!("common.back").to_string(),
    ];
    let mode_idx = match select_str(&t!("flash.select_conn_mode"), &mode_options) {
        Ok(i) => i,
        Err(_) => return,
    };

    let target_ip = match mode_idx {
        0 => {
            step(&t!("flash.scanning_network"));
            let find_bin = which("uhd_find_devices").unwrap_or_else(|_| std::path::PathBuf::from("uhd_find_devices"));
            let find_res = run_visible(find_bin.to_str().unwrap_or("uhd_find_devices"), &[]);
            let mut detected_ips = Vec::new();
            for line in find_res.output.lines() {
                let l = line.trim();
                if l.starts_with("addr:") {
                    if let Some((_, ip_val)) = l.split_once(':') {
                        let ip_str = ip_val.trim();
                        if !ip_str.is_empty() && !detected_ips.contains(&ip_str.to_string()) {
                            detected_ips.push(ip_str.to_string());
                        }
                    }
                }
            }
            if detected_ips.is_empty() {
                warn(&t!("flash.none_detected"));
                dim(&t!("flash.verify_sfp_tip"));
                println!();
                match input(&t!("flash.enter_ip"), Some("192.168.10.2")) {
                    Ok(s) => s,
                    Err(_) => return,
                }
            } else {
                detected_ips.push(t!("common.back").to_string());
                match select_str(&t!("flash.select_detected"), &detected_ips) {
                    Ok(idx) if idx < detected_ips.len() - 1 => detected_ips[idx].clone(),
                    _ => return,
                }
            }
        }
        1 => match input(&t!("flash.enter_ip"), Some("192.168.10.2")) {
            Ok(s) => s,
            Err(_) => return,
        },
        _ => return,
    };

    // 3. Test ping
    step(&t!("flash.pinging_host", ip = target_ip.as_str()));
    if ping(&target_ip) {
        ok(&t!("flash.ping_success", ip = target_ip.as_str()));
    } else {
        warn(&t!("flash.ping_timeout", ip = target_ip.as_str()));
        dim(&format!("  {}", t!("flash.bricked_guidance")));
        if let Ok(false) = confirm(&t!("flash.continue_anyway"), false) {
            press_enter();
            return;
        }
    }

    // 4. Select USRP Model with friendly guidance
    println!();
    println!(
        "  {}\n",
        style(t!("flash.select_model_title")).cyan().bold()
    );
    let models = vec![
        t!("flash.model_n200").to_string(),
        t!("flash.model_x300").to_string(),
        t!("flash.model_n300").to_string(),
        t!("flash.model_e310").to_string(),
        t!("common.back").to_string(),
    ];
    let model_idx = match select_str(&t!("flash.select_model_prompt"), &models) {
        Ok(i) => i,
        Err(_) => return,
    };
    if model_idx >= models.len() - 1 {
        return;
    }

    let usrp_type = match model_idx {
        0 => "n200",
        1 => "x300",
        2 => "n300",
        3 => "e310",
        _ => "usrp",
    };

    // 5. Select Image Version and Variant with explanation
    println!();
    println!(
        "  {}\n  {}",
        style(t!("flash.variant_explanation")).cyan().bold(),
        t!("flash.variant_explanation_body")
    );

    let local_sets = list_local_images(&config.images_cache_dir);
    let chosen_fpga_path = if local_sets.is_empty() {
        warn(&t!("flash.no_cached_images"));
        dim(&format!("  {}", t!("flash.cache_hint")));
        println!();
        match input(&t!("flash.enter_image_path"), None) {
            Ok(p) => p,
            Err(_) => return,
        }
    } else {
        let mut img_choices = Vec::new();
        for set in &local_sets {
            for bit in &set.bitfiles {
                let fname = bit.file_name().unwrap_or_default().to_string_lossy();
                // Filter matching files for selected model if possible
                if (usrp_type == "n200" && (fname.contains("n210") || fname.contains("n200")))
                    || (usrp_type == "x300" && fname.contains("x3"))
                    || (usrp_type == "n300" && fname.contains("n3"))
                    || usrp_type == "e310"
                    || model_idx > 1
                {
                    img_choices.push(bit.display().to_string());
                }
            }
        }

        // If no model-specific files found, offer all available
        if img_choices.is_empty() {
            for set in &local_sets {
                for bit in &set.bitfiles {
                    img_choices.push(bit.display().to_string());
                }
            }
        }

        img_choices.push(t!("flash.manual_file_option").to_string());
        img_choices.push(t!("common.back").to_string());

        let sel_idx = match select_str(&t!("flash.select_image_prompt"), &img_choices) {
            Ok(i) => i,
            Err(_) => return,
        };

        if sel_idx == img_choices.len() - 1 {
            return;
        } else if sel_idx == img_choices.len() - 2 {
            match input(&t!("flash.enter_image_path"), None) {
                Ok(p) => p,
                Err(_) => return,
            }
        } else {
            img_choices[sel_idx].clone()
        }
    };

    // 6. Build Command & Confirm
    let loader_str = uhd_loader.to_str().unwrap_or("uhd_image_loader");
    let args_spec = format!("type={},addr={}", usrp_type, target_ip);

    println!();
    println!("{}", style(format!("─── {} ───", t!("flash.summary_title"))).cyan());
    println!("  {}", t!("flash.summary_device", model = usrp_type.to_uppercase().as_str(), ip = target_ip.as_str()));
    println!("  {}", t!("flash.summary_file", path = chosen_fpga_path.as_str()));
    println!("  {}", t!("flash.summary_cmd"));
    println!(
        "  {}",
        style(format!("● {} --args \"{}\" --fpga-path \"{}\"", loader_str, args_spec, chosen_fpga_path))
            .yellow()
            .bold()
    );
    println!("{}", style("────────────────────────────────────────────────────────────────────────").cyan());
    println!();

    match confirm(&t!("flash.confirm_permanent"), true) {
        Ok(true) => {
            step(&t!("flash.executing_flash"));
            let res = run_visible(
                loader_str,
                &["--args", &args_spec, "--fpga-path", &chosen_fpga_path],
            );
            if res.success {
                ok(&t!("flash.flash_success"));
                println!();
                println!(
                    "  {}\n  {}",
                    style(t!("flash.reboot_required")).green().bold(),
                    t!("flash.reboot_instruction")
                );
            } else {
                err(&t!("flash.flash_error"));
                dim(&t!("flash.verify_uhd_tip"));
            }
        }
        _ => {
            info(&t!("flash.op_cancelled"));
        }
    }

    press_enter();
}
