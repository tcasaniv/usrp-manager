use crate::config::AppConfig;
use crate::detection::jtag::{self, find_djtgcfg};
use crate::executor::run_visible;
use crate::images::local::list_local_images;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{dim, err, info, ok, press_enter, print_section, step, warn};
use console::style;
use rust_i18n::t;

pub fn run_jtag_unbrick(config: &AppConfig) {
    print_section(&t!("jtag.title"));
    println!(
        "\n  {}\n  {}\n",
        style(t!("jtag.what_is_bricked")).cyan().bold(),
        t!("jtag.what_is_bricked_body")
    );
    println!(
        "  {}\n  {}\n",
        style(t!("jtag.how_jtag_works")).cyan().bold(),
        t!("jtag.how_jtag_works_body")
    );

    // 1. Check Digilent Adept Utilities
    step(&t!("diagnostic.checking_jtag"));
    let djtgcfg = match find_djtgcfg() {
        Some(p) => {
            ok(&t!("jtag.digilent_found", path = p.display().to_string().as_str()));
            p
        }
        None => {
            err(&t!("jtag.no_djtgcfg"));
            println!();
            info(&t!("jtag.install_guide_title"));
            dim(&format!("  {}", t!("jtag.install_guide_win")));
            press_enter();
            return;
        }
    };

    // 2. Step-by-Step Model Guide with Physical Instructions
    println!();
    println!(
        "  {}\n  {}",
        style(t!("jtag.model_instructions_title")).cyan().bold(),
        t!("jtag.select_model_prompt")
    );
    let model_options = vec![
        t!("jtag.model_n210_label").to_string(),
        t!("jtag.model_x300_label").to_string(),
        t!("jtag.model_b200_label").to_string(),
        t!("common.back").to_string(),
    ];
    let model_idx = match select_str(&t!("jtag.select_model_prompt"), &model_options) {
        Ok(i) => i,
        Err(_) => return,
    };
    if model_idx >= model_options.len() - 1 {
        return;
    }

    match model_idx {
        0 => {
            println!();
            println!("{}", style(format!("─── {} ───", t!("jtag.proc_n210_title"))).yellow().bold());
            println!("  {}", t!("jtag.proc_n210_step1"));
            println!("  {}", t!("jtag.proc_n210_step2"));
            println!("  {}", t!("jtag.proc_n210_step3"));
            println!("  {}", t!("jtag.proc_n210_step4"));
            println!("{}", style("──────────────────────────────────────────────────").yellow());
            println!();
        }
        1 => {
            println!();
            println!("{}", style(format!("─── {} ───", t!("jtag.proc_x300_title"))).yellow().bold());
            println!("  {}", t!("jtag.proc_x300_step1"));
            println!("  {}", t!("jtag.proc_x300_step2"));
            println!("  {}", t!("jtag.proc_x300_step3"));
            println!("{}", style("────────────────────────────────────────────").yellow());
            println!();
        }
        2 => {
            println!();
            info(&t!("jtag.b200_info"));
            dim(&t!("jtag.b200_hint"));
            press_enter();
            return;
        }
        _ => return,
    }

    // 3. Enumerate JTAG devices
    step(&t!("jtag.scanning_jtag"));
    let devices = jtag::enumerate(&djtgcfg);
    let chosen_device = if devices.is_empty() {
        warn(&t!("jtag.no_cables"));
        dim(&t!("jtag.check_cable_tip"));
        println!();
        if let Ok(true) = confirm(&t!("jtag.manual_device_prompt"), false) {
            match input(&t!("jtag.device_name_input"), Some("JtagHs2")) {
                Ok(d) if !d.trim().is_empty() => d.trim().to_string(),
                _ => return,
            }
        } else {
            press_enter();
            return;
        }
    } else {
        ok(&t!("jtag.programmer_detected", name = devices[0].name.as_str()));
        devices[0].name.clone()
    };

    // 4. Initialize Scan Chain
    step(&t!("jtag.init_chain", device = chosen_device.as_str()));
    let chain = jtag::init_and_detect(&djtgcfg, &chosen_device);
    if let Some(fpga) = chain {
        ok(&t!("jtag.fpga_found_chain", name = format!("{:?}", fpga).as_str()));
    } else {
        warn(&t!("jtag.fpga_not_identified"));
    }

    // 5. Load image sets and bail early if none
    let local_sets = list_local_images(&config.images_cache_dir);
    if local_sets.is_empty() {
        warn(&t!("jtag.no_cached_bits"));
        dim(&format!("  {}", t!("jtag.cache_hint")));
        press_enter();
        return;
    }

    // 5a. Select UHD image version — cursor pre-placed on version matching active UHD
    let active_ver = crate::detection::uhd::detect_active().map(|u| u.version);
    let mut ver_opts: Vec<String> = local_sets
        .iter()
        .map(|s| {
            let marker = if active_ver.as_deref() == Some(s.version.as_str()) {
                "  ★ (active UHD)"
            } else {
                ""
            };
            format!("v{}  ({} .bit files){}", s.version, s.bitfiles.len(), marker)
        })
        .collect();
    ver_opts.push(t!("common.back").to_string());

    let ver_idx = match select_str(&t!("jtag.select_image_version"), &ver_opts) {
        Ok(i) if i < local_sets.len() => i,
        _ => return,
    };
    let chosen_set = &local_sets[ver_idx];

    // 5b. Filter .bit files for the chosen model and (if X300/X310) variant
    let chosen_bitfile: String = match model_idx {
        0 => {
            // N200 / N210: filenames contain "n210" or "n200"
            let mut bits: Vec<String> = chosen_set
                .bitfiles
                .iter()
                .filter(|p| {
                    let name = p.file_name().unwrap_or_default()
                        .to_string_lossy().to_lowercase();
                    name.ends_with(".bit")
                        && (name.contains("n210") || name.contains("n200"))
                })
                .map(|p| p.display().to_string())
                .collect();

            if bits.is_empty() {
                warn(&t!("jtag.no_bits_for_model"));
                match input(&t!("jtag.enter_bit_path"), None) {
                    Ok(p) => p,
                    Err(_) => return,
                }
            } else {
                bits.push(t!("jtag.manual_bit_option").to_string());
                bits.push(t!("common.back").to_string());
                let idx = match select_str(&t!("jtag.select_bit_prompt"), &bits) {
                    Ok(i) => i,
                    Err(_) => return,
                };
                if idx == bits.len() - 1 {
                    return;
                } else if idx == bits.len() - 2 {
                    match input(&t!("jtag.enter_bit_path"), None) {
                        Ok(p) => p,
                        Err(_) => return,
                    }
                } else {
                    bits[idx].clone()
                }
            }
        }
        1 => {
            // X300 / X310: ask HG vs XG variant then resolve exact .bit file
            let variant_opts = vec![
                t!("jtag.variant_hg").to_string(),
                t!("jtag.variant_xg").to_string(),
                t!("common.back").to_string(),
            ];
            let var_idx = match select_str(&t!("jtag.select_variant"), &variant_opts) {
                Ok(i) if i < 2 => i,
                _ => return,
            };
            let variant = if var_idx == 0 { "HG" } else { "XG" };

            // Try X300 first, then X310
            let found = crate::images::local::find_bitstream_in_set(chosen_set, "x300", variant)
                .or_else(|| crate::images::local::find_bitstream_in_set(chosen_set, "x310", variant));

            match found {
                Some(p) => p.display().to_string(),
                None => {
                    warn(&t!("jtag.no_bits_for_model"));
                    match input(&t!("jtag.enter_bit_path"), None) {
                        Ok(p) => p,
                        Err(_) => return,
                    }
                }
            }
        }
        _ => return,
    };

    // 6. Build command and show to user
    let djtg_str = djtgcfg.to_str().unwrap_or("djtgcfg");
    println!();
    println!("{}", style(format!("─── {} ───", t!("jtag.summary_title"))).cyan());
    println!("  {}", t!("jtag.summary_device", device = chosen_device.as_str()));
    println!("  {}", t!("jtag.summary_file", path = chosen_bitfile.as_str()));
    println!("  {}", t!("jtag.summary_cmd"));
    println!(
        "  {}",
        style(format!("● {} prog -d \"{}\" -i 0 -f \"{}\"", djtg_str, chosen_device, chosen_bitfile))
            .yellow()
            .bold()
    );
    println!("{}", style("────────────────────────────────────────────────────────────────────────────").cyan());
    println!();

    match confirm(&t!("jtag.confirm_program"), true) {
        Ok(true) => {
            step(&t!("jtag.programming_jtag"));
            let res = run_visible(
                djtg_str,
                &["prog", "-d", &chosen_device, "-i", "0", "-f", &chosen_bitfile],
            );
            if res.success {
                ok(&t!("jtag.prog_success"));
                println!();
                println!(
                    "  {}\n  {}\n",
                    style(t!("jtag.status_now")).green().bold(),
                    t!("jtag.ram_active_notice")
                );
                if let Ok(true) = confirm(&t!("jtag.offer_permanent_flash"), true) {
                    crate::flash::run_flash_network(config);
                    return;
                }
            } else {
                err(&t!("jtag.prog_failed"));
                dim(&t!("jtag.check_connection_tip"));
            }
        }
        _ => {
            info(&t!("jtag.op_cancelled"));
        }
    }

    press_enter();
}
