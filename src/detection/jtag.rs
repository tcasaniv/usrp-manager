use crate::executor::{run_silent, run_visible};
use std::path::PathBuf;
use which::which;

#[derive(Debug, Clone, PartialEq)]
pub enum FpgaModel {
    XC7K325T, // USRP X300
    XC7K410T, // USRP X310
    Unknown(String),
}

impl FpgaModel {
    pub fn usrp_model(&self) -> &str {
        match self {
            FpgaModel::XC7K325T => "X300",
            FpgaModel::XC7K410T => "X310",
            FpgaModel::Unknown(_) => "Unknown",
        }
    }

    pub fn fpga_name(&self) -> &str {
        match self {
            FpgaModel::XC7K325T => "XC7K325T",
            FpgaModel::XC7K410T => "XC7K410T",
            FpgaModel::Unknown(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JtagDevice {
    pub name: String,
    pub transport: String,
    pub product: String,
    pub serial: String,
}

/// Find the djtgcfg executable.
pub fn find_djtgcfg() -> Option<PathBuf> {
    // Check well-known Windows location first
    let win_default = PathBuf::from(
        r"C:\Program Files (x86)\Digilent\AdeptUtilities\djtgcfg.exe",
    );
    if win_default.exists() {
        return Some(win_default);
    }

    // Try PATH
    if let Ok(p) = which("djtgcfg") {
        return Some(p);
    }

    // Linux common path
    let linux_default = PathBuf::from("/usr/bin/djtgcfg");
    if linux_default.exists() {
        return Some(linux_default);
    }

    None
}

/// Enumerate connected JTAG devices.
pub fn enumerate(djtgcfg: &PathBuf) -> Vec<JtagDevice> {
    let result = run_visible(djtgcfg.to_str().unwrap_or("djtgcfg"), &["enum"]);
    if !result.success {
        return vec![];
    }
    parse_enum_output(&result.output)
}

fn parse_enum_output(output: &str) -> Vec<JtagDevice> {
    let mut devices = Vec::new();
    let mut current_name: Option<String> = None;
    let mut transport = String::new();
    let mut product = String::new();
    let mut serial = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix("Device: ") {
            if let Some(n) = current_name.take() {
                devices.push(JtagDevice {
                    name: n,
                    transport: transport.clone(),
                    product: product.clone(),
                    serial: serial.clone(),
                });
                transport.clear();
                product.clear();
                serial.clear();
            }
            current_name = Some(name.trim().to_string());
        } else if let Some(t) = trimmed.strip_prefix("Device Transport Type: ") {
            transport = t.trim().to_string();
        } else if let Some(p) = trimmed.strip_prefix("Product Name: ") {
            product = p.trim().to_string();
        } else if let Some(s) = trimmed.strip_prefix("Serial Number: ") {
            serial = s.trim().to_string();
        }
    }
    if let Some(n) = current_name {
        devices.push(JtagDevice {
            name: n,
            transport,
            product,
            serial,
        });
    }
    devices
}

/// Initialize JTAG scan chain and return detected FPGA model.
pub fn init_and_detect(djtgcfg: &PathBuf, device_name: &str) -> Option<FpgaModel> {
    let result = run_visible(
        djtgcfg.to_str().unwrap_or("djtgcfg"),
        &["init", "-d", device_name],
    );

    if !result.success {
        return None;
    }

    // Parse "Device 0: XC7K410T"
    for line in result.output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Device 0: ") {
            let chip = rest.trim();
            return Some(match chip {
                "XC7K325T" => FpgaModel::XC7K325T,
                "XC7K410T" => FpgaModel::XC7K410T,
                other => FpgaModel::Unknown(other.to_string()),
            });
        }
        // Also try "Device N: ..." for any index
        if trimmed.starts_with("Device ") {
            if let Some(colon) = trimmed.find(": ") {
                let chip = trimmed[colon + 2..].trim();
                return Some(match chip {
                    "XC7K325T" => FpgaModel::XC7K325T,
                    "XC7K410T" => FpgaModel::XC7K410T,
                    other => FpgaModel::Unknown(other.to_string()),
                });
            }
        }
    }

    None
}

/// Program the FPGA via JTAG.
pub fn program(djtgcfg: &PathBuf, device_name: &str, bitfile: &std::path::Path) -> bool {
    let bitfile_str = bitfile.to_str().unwrap_or("");
    let result = run_visible(
        djtgcfg.to_str().unwrap_or("djtgcfg"),
        &["prog", "-d", device_name, "-i", "0", "-f", bitfile_str],
    );
    result.success && result.output.contains("Programming succeeded")
}

/// Quick check: returns true if djtgcfg enum finds at least one device (silent).
pub fn has_jtag_device(djtgcfg: &PathBuf) -> bool {
    let out = run_silent(
        djtgcfg.to_str().unwrap_or("djtgcfg"),
        &["enum"],
    );
    out.map(|s| s.contains("Device:")).unwrap_or(false)
}
