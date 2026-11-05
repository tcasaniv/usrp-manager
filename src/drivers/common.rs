#[allow(dead_code)]
pub struct InstalledUhd {
    pub version: String,
    pub install_dir: std::path::PathBuf,
    pub bin_dir: std::path::PathBuf,
    pub is_active: bool,
}
