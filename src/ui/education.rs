use console::style;
use rust_i18n::t;

use super::styles::{press_enter, print_section, warn};

pub fn show_education_menu() {
    loop {
        print_section(&t!("education.title"));
        let topics = vec![
            t!("education.topic_what_usrp").to_string(),
            t!("education.topic_fpga").to_string(),
            t!("education.topic_antennas").to_string(),
            t!("education.topic_bricked").to_string(),
            t!("education.topic_versions").to_string(),
            t!("education.topic_models").to_string(),
            t!("education.topic_n210_guide").to_string(),
            t!("education.topic_back").to_string(),
        ];
        let idx = match super::prompts::select_str(&t!("education.select_topic"), &topics) {
            Ok(i) => i,
            Err(_) => break,
        };
        match idx {
            0 => show_what_usrp(),
            1 => show_fpga_explained(),
            2 => show_antenna_safety(),
            3 => show_bricked_explained(),
            4 => show_version_guide(),
            5 => show_models(),
            6 => show_n210_guide(),
            _ => break,
        }
    }
}

fn show_what_usrp() {
    print_section(&t!("education.edu_what_title"));
    println!(
        "\n  {}\n{}\n\n  {}\n{}\n\n  {}\n{}\n\n  {}\n{}\n",
        style(t!("education.edu_what_sec1")).cyan().bold(),
        t!("education.edu_what_body1"),
        style(t!("education.edu_what_sec2")).cyan().bold(),
        t!("education.edu_what_body2"),
        style(t!("education.edu_what_sec3")).cyan().bold(),
        t!("education.edu_what_body3"),
        style(t!("education.edu_what_sec4")).yellow().bold(),
        t!("education.edu_what_body4"),
    );
    press_enter();
}

fn show_fpga_explained() {
    print_section(&t!("education.edu_fpga_title"));
    println!(
        "\n  {}\n{}\n\n  {}\n{}\n\n  {}\n{}\n\n  {}\n{}\n",
        style(t!("education.edu_fpga_sec1")).cyan().bold(),
        t!("education.edu_fpga_body1"),
        style(t!("education.edu_fpga_sec2")).cyan().bold(),
        t!("education.edu_fpga_body2"),
        style(t!("education.edu_fpga_sec3")).yellow().bold(),
        t!("education.edu_fpga_body3"),
        style(t!("education.edu_fpga_sec4")).cyan().bold(),
        t!("education.edu_fpga_body4"),
    );
    press_enter();
}

fn show_antenna_safety() {
    print_section(&t!("education.edu_ant_title"));
    warn(&t!("education.edu_ant_warn"));
    println!(
        "\n  {}\n{}\n\n  {}\n{}\n",
        style(t!("education.edu_ant_sec1")).yellow().bold(),
        t!("education.edu_ant_body1"),
        style(t!("education.edu_ant_sec2")).cyan().bold(),
        t!("education.edu_ant_body2"),
    );
    press_enter();
}

fn show_bricked_explained() {
    print_section(&t!("education.edu_brick_title"));
    println!(
        "\n  {}\n{}\n\n  {}\n{}\n",
        style(t!("education.edu_brick_sec1")).yellow().bold(),
        t!("education.edu_brick_body1"),
        style(t!("education.edu_brick_sec2")).green().bold(),
        t!("education.edu_brick_body2"),
    );
    press_enter();
}

fn show_version_guide() {
    print_section(&t!("education.topic_versions"));
    println!("  {}", style("● UHD <-> FPGA Compatibility").cyan().bold());
    println!("  USRP devices verify FPGA compatibility using an internal register.");
    println!("  If versions differ, uhd_find_devices or uhd_usrp_probe will fail.\n");
    println!("  {}", style("● Quick Fix").yellow().bold());
    println!("  1. Flash matching FPGA image or switch active UHD driver.\n");
    press_enter();
}

fn show_models() {
    print_section(&t!("education.edu_models_title"));
    println!("\n{}\n", t!("education.edu_models_body"));
    press_enter();
}

fn show_n210_guide() {
    print_section(&t!("education.topic_n210_guide"));
    println!("  {}", style("● Steps to Flash USRP N200 / N210").cyan().bold());
    println!("  1. Connect Ethernet to SFP0, set PC IP to 192.168.10.1.");
    println!("  2. Run Flash FPGA via Network in usrp-manager.\n");
    println!("  {}", style("● In case of failure / bricking").yellow().bold());
    println!("  Connect Digilent USB-JTAG cable and run Unbrick via JTAG.\n");
    press_enter();
}
