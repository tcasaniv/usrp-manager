use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct InstalledUhd {
    pub version: String,
    pub install_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub is_active: bool,
}
