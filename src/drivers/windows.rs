#[cfg(target_os = "windows")]
use anyhow::{bail, Result};
#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
use crate::executor::run_visible;

/// Get the current system PATH from the registry.
#[cfg(target_os = "windows")]
pub fn get_system_path() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let env = hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_READ,
    )?;
    let path: String = env.get_value("Path")?;
    Ok(path)
}

/// Set the system PATH in the registry (requires admin).
#[cfg(target_os = "windows")]
pub fn set_system_path(new_path: &str) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (env, _) = hklm.create_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_WRITE,
    )?;
    env.set_value("Path", &new_path.to_string())?;
    // Notify system of PATH change
    broadcast_settings_change();
    Ok(())
}

/// Broadcast WM_SETTINGCHANGE so Explorer/shells pick up the new PATH.
#[cfg(target_os = "windows")]
fn broadcast_settings_change() {
    // Use PowerShell to send HWND_BROADCAST
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"
            $signature = 'using System;using System.Runtime.InteropServices;public class WinMsg{[DllImport("user32.dll",SetLastError=true)]public static extern IntPtr SendMessageTimeout(IntPtr hWnd,uint Msg,UIntPtr wParam,string lParam,uint fuFlags,uint uTimeout,out UIntPtr lpdwResult);}';
            Add-Type -TypeDefinition $signature;
            [UIntPtr]$r = [UIntPtr]::Zero;
            [WinMsg]::SendMessageTimeout([IntPtr]0xffff,0x001A,[UIntPtr]::Zero,'Environment',2,5000,[ref]$r)
            "#,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Update system PATH to make a UHD bin dir the active version (Windows).
#[cfg(target_os = "windows")]
pub fn set_active_version(bin_dir: &Path) -> Result<()> {
    let current = get_system_path()?;
    // Remove any existing UHD entries
    let entries: Vec<&str> = current
        .split(';')
        .filter(|e| {
            let lower = e.to_lowercase();
            !lower.contains("uhd") && !lower.contains("ettus")
        })
        .collect();

    let new_path = format!("{};{}", bin_dir.display(), entries.join(";"));
    set_system_path(&new_path)?;
    Ok(())
}

/// Install UHD from a Windows .exe installer silently.
#[cfg(target_os = "windows")]
pub fn install_from_exe(installer: &Path, target_dir: &Path) -> Result<()> {
    let target_str = target_dir.display().to_string();
    // NSIS silent install: /S /D=path (no quotes around path, must be last arg)
    let result = run_visible(
        installer.to_str().unwrap_or("installer.exe"),
        &["/S", &format!("/D={}", target_str)],
    );
    if !result.success {
        bail!("Installer exited with code {}", result.exit_code);
    }
    Ok(())
}

/// Scan UHD_versions dir for installed UHD versions.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn detect_installed(versions_dir: &Path, active_bin: Option<&Path>) -> Vec<super::common::InstalledUhd> {
    crate::detection::uhd_versions::detect_installed_versions(versions_dir, active_bin)
        .into_iter()
        .map(|v| super::common::InstalledUhd {
            version: v.version,
            install_dir: v.install_dir,
            bin_dir: v.bin_dir,
            is_active: v.is_active,
        })
        .collect()
}
