use anyhow::Result;
use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub locale: String,
    pub images_cache_dir: PathBuf,
    pub uhd_versions_dir: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            images_cache_dir: default_images_dir(),
            uhd_versions_dir: default_uhd_versions_dir(),
        }
    }
}

fn default_images_dir() -> PathBuf {
    if let Some(d) = data_dir() {
        d.join("usrp-manager").join("fpga-images")
    } else {
        PathBuf::from("fpga-images")
    }
}

fn default_uhd_versions_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Program Files\UHD_versions")
    }
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/opt/uhd")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/opt/uhd")
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        PathBuf::from("uhd_versions")
    }
}

fn config_path() -> PathBuf {
    if let Some(d) = data_dir() {
        d.join("usrp-manager").join("config.json")
    } else {
        PathBuf::from("usrp-manager-config.json")
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn ensure_dirs(&self) {
        let _ = std::fs::create_dir_all(&self.images_cache_dir);
        let _ = std::fs::create_dir_all(&self.uhd_versions_dir);
    }
}
