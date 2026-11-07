use crate::executor::run_visible;
use crate::ui::prompts::{confirm, input, select_str};
use crate::ui::styles::{dim, err, ok, press_enter, print_section, step, warn};
use console::style;
use rust_i18n::t;
use which::which;

pub fn run_probe_menu() {
    print_section(&t!("probe.title"));
    dim(&t!("probe.intro"));
    println!();

    let uhd_find = match which("uhd_find_devices") {
        Ok(p) => p,
        Err(_) => {
            err(&t!("probe.no_uhd"));
            press_enter();
            return;
        }
    };

    let uhd_probe = match which("uhd_usrp_probe") {
        Ok(p) => p,
        Err(_) => {
            err(&t!("probe.no_uhd"));
            press_enter();
            return;
        }
    };

    step(&t!("probe.scanning"));
    // Run uhd_find_devices visibly so the user sees the raw command and output
    let find_res = run_visible(uhd_find.to_str().unwrap_or("uhd_find_devices"), &[]);

    let mut device_args = Vec::new();
    for line in find_res.output.lines() {
        let trimmed = line.trim();
        // Catch lines like "addr: 192.168.10.2" or "type: b200"
        if trimmed.starts_with("addr:") || trimmed.starts_with("serial:") || trimmed.starts_with("type:") {
            if let Some((_, val)) = trimmed.split_once(':') {
                let v = val.trim();
                if !v.is_empty() && !device_args.contains(&format!("addr={}", v)) && v.contains('.') {
                    device_args.push(format!("addr={}", v));
                }
            }
        }
    }

    let chosen_args = if device_args.is_empty() {
        warn(&t!("diagnostic.usrp_not_found"));
        println!();
        if let Ok(true) = confirm(&t!("probe.manual_args_prompt"), true) {
            match input(&t!("probe.enter_args"), Some("addr=192.168.10.2")) {
                Ok(a) if !a.trim().is_empty() => a,
                _ => {
                    press_enter();
                    return;
                }
            }
        } else {
            press_enter();
            return;
        }
    } else {
        device_args.push(t!("common.back").to_string());
        match select_str(&t!("probe.select_device"), &device_args) {
            Ok(idx) if idx < device_args.len() - 1 => device_args[idx].clone(),
            _ => return,
        }
    };

    println!();
    step(&t!("probe.running_probe"));
    // Execute uhd_usrp_probe visibly
    let args_str = chosen_args.trim();
    let probe_res = run_visible(
        uhd_probe.to_str().unwrap_or("uhd_usrp_probe"),
        &["--args", args_str],
    );

    if !probe_res.success {
        err(&t!("probe.probe_failed"));
    } else {
        println!();
        ok(&t!("probe.summary_title"));
        // Parse and present clean summary
        for line in probe_res.output.lines() {
            let l = line.trim();
            if l.contains("Mboard:") || l.contains("Device:") || l.contains("product:") {
                println!("  {} {}", style("●").cyan(), l);
            } else if l.contains("Serial:") || l.contains("serial:") {
                println!("  {} {}", style("?").magenta(), l);
                println!("  {} {}", style("●").cyan(), l);
            } else if l.contains("FW Version:") || l.contains("FPGA Version:") {
                println!("  {} {}", style("●").green().bold(), l);
            } else if l.contains("Subdev:") || l.contains("Daughterboard") || l.contains("DB ") {
                println!("  {} {}", style("●").yellow(), l);
            }
        }
    }

    println!();
    if let Ok(true) = confirm(&t!("probe.show_full_log"), false) {
        println!("\n{}\n", style("─── Full Technical Output ───").dim());
        println!("{}", probe_res.output);
        println!("{}\n", style("─────────────────────────────").dim());
    }

    press_enter();
}
