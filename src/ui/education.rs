use console::style;
use rust_i18n::t;

use super::styles::{info, press_enter, print_section, warn};

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
    print_section("What is a USRP?");
    println!(
        r#"
  {}

  A USRP (Universal Software Radio Peripheral) is a hardware device
  made by Ettus Research (a National Instruments / Emerson company)
  that acts as a software-defined radio (SDR).

  {}

  Instead of fixed hardware circuits for each radio standard (Wi-Fi, LTE,
  FM radio, etc.), an SDR uses a general-purpose processor + FPGA to handle
  the signal processing in software. This makes the same hardware usable for:
    • FM/AM radio reception
    • GSM / LTE / 5G research
    • Radar simulation
    • Satellite signal analysis
    • Amateur radio (ham radio)
    • Spectrum monitoring

  {}

  The FPGA (Field Programmable Gate Array) in the USRP handles the
  real-time digital signal processing that is too fast for a regular CPU.
  The host computer runs UHD (USRP Hardware Driver) software to control
  the USRP over Ethernet or USB.

  {}

  The UHD software and FPGA image must be from compatible versions.
  If they are different, the USRP may not be detected or may malfunction.
"#,
        style("● What is SDR?").cyan().bold(),
        style("● How does it work?").cyan().bold(),
        style("● Role of the FPGA").cyan().bold(),
        style("● Why versions matter").yellow().bold(),
    );
    press_enter();
}

fn show_fpga_explained() {
    print_section("FPGA & Why Versions Matter");
    println!(
        r#"
  {}

  FPGA = Field-Programmable Gate Array.
  It's a chip that can be reconfigured to implement any digital circuit.
  In the USRP, it handles:
    • ADC/DAC sample routing
    • Decimation / interpolation filters
    • IQ correction
    • VITA49 packet framing

  {}

  The FPGA image is a binary file (.bit) that programs the FPGA.
  Every new UHD release may bring a new FPGA image with bug fixes or
  new features. The host UHD software communicates with the FPGA using
  an internal protocol — if versions differ, communication breaks.

  {}

  UHD version and FPGA image version must match EXACTLY (e.g., both 4.10).

  {}

  Run "System Diagnostic" in this tool — it will check if your installed
  UHD version matches any available FPGA image.
"#,
        style("● What is FPGA?").cyan().bold(),
        style("● The FPGA image (.bit file)").cyan().bold(),
        style("● ⚠  Critical: version matching").yellow().bold(),
        style("● How to check").cyan().bold(),
    );
    press_enter();
}

fn show_antenna_safety() {
    print_section("⚠  Antennas: RF Safety");
    warn("READ THIS BEFORE CONNECTING ANY ANTENNA OR TRANSMITTING");
    println!(
        r#"
  {}

  NEVER transmit without an antenna or into an open port.
  Transmitting into an open connector reflects the RF signal back
  into the USRP's power amplifier, which can permanently damage it.
  Always use a 50-ohm terminator or a proper antenna.

  {}

  Every antenna has a frequency range it can operate in.
  Using an antenna outside its rated frequency range causes:
    • Very poor signal quality
    • Potential for reflected power damage

  Common antenna ranges:
    • VERT900 (Ettus):  824 MHz – 960 MHz, 1710 MHz – 1990 MHz
    • VERT2450 (Ettus): 2.4 GHz – 2.5 GHz, 4.9 GHz – 5.9 GHz
    • SBX daughterboard: 400 MHz – 4.4 GHz
    • WBX daughterboard: 50 MHz – 2.2 GHz

  {}

  Transmitting on unauthorized frequencies is ILLEGAL in most countries.
  In many places you need a radio license (HAM license) to transmit.
  NEVER transmit on frequencies used by:
    • Emergency services (police, fire, ambulance)
    • Aviation (ATC)
    • Military frequencies
    • Commercial broadcast (FM, AM, TV)
  
  For research/lab use: Use a shielded RF chamber or very low power
  with a direct wired connection between TX and RX ports.

  {}

  For classroom USRP demos, the safest approach is:
    1. Use a loopback cable (SMA) between TX and RX
    2. Use very low TX gain (0 dB or minimum)
    3. Keep the frequency in an ISM band (2.4 GHz) if antenna is needed
"#,
        style("● ⚠  NEVER transmit without antenna or terminator").red().bold(),
        style("● Antenna frequency range").cyan().bold(),
        style("● ⚠  Legal requirements").red().bold(),
        style("● Safe classroom use").cyan().bold(),
    );
    press_enter();
}

fn show_bricked_explained() {
    print_section("Is my USRP bricked?");
    println!(
        r#"
  {}

    • The USRP powers on (LEDs light up) but UHD cannot detect it
    • The USRP is on the network but uhd_find_devices gives no output
    • A previous FPGA flash failed midway
    • The FPGA image version does not match UHD drivers installed

  {}

    • No LED activity when powered on → possible power supply issue
    • USRP detected but RF performance is very poor → may be hardware damage

  {}

  For X300/X310:
    1. Use "Unbrick via JTAG" in this tool (requires JTAG cable)
    2. After JTAG, the USRP is temporarily recovered (RAM only)
    3. Then use "Flash FPGA via Network" to make it permanent

  {}

  Bricking happens when:
    • A flash operation (via network) is interrupted (power cut, cable pull)
    • Wrong FPGA image is flashed (mismatched hardware)
    • USRP firmware updated with incompatible version

  Prevention:
    • Never interrupt a flash operation
    • Always verify UHD version matches FPGA image before flashing
    • Use "System Diagnostic" before any flash operation
"#,
        style("● Signs of a bricked USRP").cyan().bold(),
        style("● Signs of hardware damage (NOT software)").yellow().bold(),
        style("● Recovery: X300/X310").cyan().bold(),
        style("● How bricking happens & prevention").cyan().bold(),
    );
    press_enter();
}

fn show_version_guide() {
    print_section("Which UHD version do I need?");
    println!(
        r#"
  {}

  The FPGA image (the .bit file inside the uhd-images archive) and the
  UHD software on your host must have MATCHING major.minor versions.

  Example:
    • UHD 4.10.x.x  →  use uhd-images_4.10.x.x  (.bit files inside)
    • UHD 4.8.x.x   →  use uhd-images_4.8.x.x

  {}

  If you are using Ubuntu 24 LTS with the default apt packages, you get
  UHD 4.6.x. But many lab setups need UHD 4.8 or 4.10 or 4.11.
  Use "Manage UHD Drivers" → "Install a new version" to get the right one.

  {}

  Run "System Diagnostic" in this tool. It will show:
    • Active UHD version
    • Available FPGA images in cache
    • Whether the USRP is reachable on the network
    • Whether there is a version mismatch
"#,
        style("● Rule: host UHD version = FPGA image version").cyan().bold(),
        style("● ⚠  Linux package manager versions may be old").yellow().bold(),
        style("● Easiest way to check").cyan().bold(),
    );
    press_enter();
}

fn show_models() {
    print_section("USRP Models");
    println!(
        r#"
  {}   USRP X300 & X310
  ─────────────────────────────────────────────────
  • High-performance, desktop/lab form factor
  • FPGA: Xilinx Kintex-7 (X300: XC7K325T, X310: XC7K410T)
  • 2× daughterboard slots (SBX, UBX, TwinRX, etc.)
  • Network: 2× SFP ports (1G or 10G Ethernet)
  • USB3 for direct connection (slower)
  • Variants: HG (1G+10G), XG (10G+10G)
  • Fully supported by this tool (JTAG + network flash)

  {}   USRP N210 / N200
  ─────────────────────────────────────────────────
  • Mid-range, GigE-only
  • FPGA: Xilinx Spartan-3A DSP 3400 (N210) / 1800 (N200)
  • 1× daughterboard slot
  • Recovery: different process (Ethernet-based, no JTAG needed)
  • See: "N210 — Flashing Guide" in this menu

  {}   USRP B200 / B205mini / B210
  ─────────────────────────────────────────────────
  • USB 3.0 connected, portable
  • Integrated RF chip (AD9361 or AD9364)
  • No external JTAG needed — reflash via USB with UHD tools
  • Popular for portable / field use

  {}   USRP E3xx / E320
  ─────────────────────────────────────────────────
  • Embedded USRP (ARM CPU onboard)
  • Can run UHD locally on the device
  • Ethernet + PCIe connectivity options
"#,
        style("★").yellow().bold(),
        style("★").cyan().bold(),
        style("★").cyan().bold(),
        style("★").cyan().bold(),
    );
    press_enter();
}

fn show_n210_guide() {
    print_section("N210 — Flashing Guide");
    info("Note: N210 recovery is network-based, no JTAG cable required.");
    println!(
        r#"
  The USRP N210 uses a different recovery process from the X300/X310.
  It has a built-in network bootloader that allows re-flashing via Ethernet.

  {}

    1. Connect the N210 via Ethernet (1GbE port)
    2. Set host IP to 192.168.10.1 / mask 255.255.255.0
    3. Power on the N210 while holding the reset button for 10 seconds
       (until front LED flashes rapidly) — this enters the bootloader
    4. Find the correct .bin image for your N210 from UHD images
    5. Run:
       uhd_image_loader --args="type=usrp1,addr=192.168.10.2" --fpga-path=<path/to/n210.bin>
       (Adjust 'type' and image path accordingly)

  {}

  The bootloader mode allows flashing even if the main firmware is broken.
  This is why N210 does NOT need a JTAG cable.

  {}

  For detailed official instructions:
  https://kb.ettus.com/USRP_N200/N210_Getting_Started_Guide
"#,
        style("● Steps to flash N210").cyan().bold(),
        style("● Why N210 doesn't need JTAG").cyan().bold(),
        style("● Official documentation").dim(),
    );
    press_enter();
}
