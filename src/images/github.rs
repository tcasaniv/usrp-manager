use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[allow(dead_code)]
    pub name: String,
    pub published_at: String,
    pub assets: Vec<Asset>,
}

impl Release {
    /// Return the UHD version string from tag (e.g. "v4.10.0.0" → "4.10.0.0")
    pub fn version(&self) -> String {
        self.tag_name.trim_start_matches('v').to_string()
    }

    /// Find the images zip asset
    pub fn images_zip(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| {
            a.name.starts_with("uhd-images_") && a.name.ends_with(".zip")
        })
    }

    /// Find the sha256 checksum asset for images zip
    pub fn images_sha256(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| {
            a.name.starts_with("uhd-images_") && a.name.ends_with(".sha256")
        })
    }

    /// Find Windows installer asset (VS2019 or VS2022)
    pub fn windows_installer(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| {
            a.name.contains("Win64") && a.name.ends_with(".exe")
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

impl Asset {
    pub fn human_size(&self) -> String {
        let mb = self.size as f64 / 1_048_576.0;
        if mb < 1.0 {
            format!("{} KB", self.size / 1024)
        } else {
            format!("{:.1} MB", mb)
        }
    }
}

/// Fetch UHD releases from the GitHub API.
pub fn fetch_releases() -> Result<Vec<Release>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("usrp-manager/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let url = "https://api.github.com/repos/EttusResearch/uhd/releases?per_page=20";
    let resp = client.get(url).send()?;
    let releases: Vec<Release> = resp.json()?;
    Ok(releases)
}
