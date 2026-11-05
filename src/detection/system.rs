/// Returns true if the current process has admin/root privileges.
pub fn is_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Try to query a protected registry key
        let status = Command::new("net")
            .args(["session"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        matches!(status, Ok(s) if s.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe { libc::geteuid() == 0 }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OsType {
    Windows,
    Linux,
    MacOs,
    Other,
}

pub fn get_os() -> OsType {
    #[cfg(target_os = "windows")]
    return OsType::Windows;
    #[cfg(target_os = "linux")]
    return OsType::Linux;
    #[cfg(target_os = "macos")]
    return OsType::MacOs;
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return OsType::Other;
}
