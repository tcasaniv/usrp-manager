use crate::executor::run_silent;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FoundUsrp {
    #[allow(dead_code)]
    pub ip: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub fpga_version: Option<String>,
}

/// Ping an IP address (1 packet, 1 second timeout).
pub fn ping(ip: &str) -> bool {
    #[cfg(target_os = "windows")]
    let result = run_silent("ping", &["-n", "1", "-w", "1000", ip]);
    #[cfg(not(target_os = "windows"))]
    let result = run_silent("ping", &["-c", "1", "-W", "1", ip]);

    result.map(|out| {
        !out.contains("unreachable")
            && !out.contains("100% packet loss")
            && !out.contains("100% loss")
            && !out.contains("Request timeout")
            && !out.contains("Destination host unreachable")
    }).unwrap_or(false)
}

/// Scan default USRP IPs and return reachable ones.
pub fn scan_defaults() -> Vec<String> {
    let defaults = ["192.168.10.2", "192.168.40.2", "192.168.30.2"];
    defaults
        .iter()
        .filter(|ip| ping(ip))
        .map(|ip| ip.to_string())
        .collect()
}

/// Probe a USRP using uhd_find_devices.
pub fn probe_with_uhd(ip: &str, uhd_find: &Path) -> Option<FoundUsrp> {
    let addr_arg = format!("addr={}", ip);
    let result = run_silent(
        uhd_find.to_str().unwrap_or("uhd_find_devices"),
        &["--args", &addr_arg],
    )?;

    if result.trim().is_empty() || result.contains("No UHD Devices Found") {
        return None;
    }

    // Parse output for model, serial, fpga version
    let mut model = None;
    let mut serial = None;
    let mut fpga_version = None;

    for line in result.lines() {
        let trimmed = line.trim();
        if let Some(v) = trimmed.strip_prefix("type: ").or_else(|| trimmed.strip_prefix("Product: ")) {
            model = Some(v.trim().to_string());
        }
        if let Some(v) = trimmed.strip_prefix("serial: ") {
            serial = Some(v.trim().to_string());
        }
        if let Some(v) = trimmed.strip_prefix("fpga: ")
            .or_else(|| trimmed.strip_prefix("FPGA Version: "))
        {
            fpga_version = Some(v.trim().to_string());
        }
    }

    Some(FoundUsrp {
        ip: ip.to_string(),
        model,
        serial,
        fpga_version,
    })
}
