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

    // 5. Select bitstream image to load
    let local_sets = list_local_images(&config.images_cache_dir);
    let chosen_bitfile = if local_sets.is_empty() {
        warn(&t!("jtag.no_cached_bits"));
        dim(&format!("  {}", t!("jtag.cache_hint")));
        match input(&t!("jtag.enter_bit_path"), None) {
            Ok(p) => p,
            Err(_) => return,
        }
    } else {
        let mut bits = Vec::new();
        for set in &local_sets {
            for b in &set.bitfiles {
                let fname = b.file_name().unwrap_or_default().to_string_lossy();
                if fname.ends_with(".bit") {
                    bits.push(b.display().to_string());
                }
            }
        }
        if bits.is_empty() {
            match input(&t!("jtag.enter_bit_path"), None) {
                Ok(p) => p,
                Err(_) => return,
            }
        } else {
            bits.push(t!("jtag.manual_bit_option").to_string());
            bits.push(t!("common.back").to_string());
            let s_idx = match select_str(&t!("jtag.select_bit_prompt"), &bits) {
                Ok(i) => i,
                Err(_) => return,
            };
            if s_idx == bits.len() - 1 {
                return;
            } else if s_idx == bits.len() - 2 {
                match input(&t!("jtag.enter_bit_path"), None) {
                    Ok(p) => p,
                    Err(_) => return,
                }
            } else {
                bits[s_idx].clone()
            }
        }
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
