use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct LocalImageSet {
    /// e.g. "4.10.0.0"
    pub version: String,
    /// Directory containing the .bit files
    pub dir: PathBuf,
    pub bitfiles: Vec<PathBuf>,
}

/// List all locally cached image sets in the cache directory.
pub fn list_local_images(cache_dir: &Path) -> Vec<LocalImageSet> {
    let mut result = Vec::new();

    let entries = match std::fs::read_dir(cache_dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let bits = find_bit_files(&path);
        if bits.is_empty() {
            // Check one level deeper (zip extracts with a subfolder)
            if let Ok(sub_entries) = std::fs::read_dir(&path) {
                for sub in sub_entries.flatten() {
                    let sub_path = sub.path();
                    if sub_path.is_dir() {
                        let sub_bits = find_bit_files(&sub_path);
                        if !sub_bits.is_empty() {
                            let version = guess_version(&path);
                            result.push(LocalImageSet {
                                version,
                                dir: sub_path,
                                bitfiles: sub_bits,
                            });
                        }
                    }
                }
            }
            continue;
        }
        let version = guess_version(&path);
        result.push(LocalImageSet {
            version,
            dir: path,
            bitfiles: bits,
        });
    }

    result.sort_by(|a, b| a.version.cmp(&b.version));
    result
}

fn find_bit_files(dir: &Path) -> Vec<PathBuf> {
    let mut bits = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().map(|e| e == "bit").unwrap_or(false) {
                bits.push(p);
            }
        }
    }
    bits.sort();
    bits
}

fn guess_version(dir: &Path) -> String {
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    // "uhd-images_4.10.0.0" → "4.10.0.0"
    let stripped = name
        .strip_prefix("uhd-images_")
        .unwrap_or(&name)
        .to_string();
    stripped
}

/// Find the best bitstream for a given USRP model and variant in cache.
pub fn find_bitstream(
    cache_dir: &Path,
    usrp_model: &str, // "X300" or "X310"
    variant: &str,    // "HG" or "XG"
) -> Option<PathBuf> {
    let images = list_local_images(cache_dir);
    // Prefer the latest version
    let latest = images.last()?;
    find_bitstream_in_set(latest, usrp_model, variant)
}

pub fn find_bitstream_in_set(
    set: &LocalImageSet,
    usrp_model: &str,
    variant: &str,
) -> Option<PathBuf> {
    // Expected filename: usrp_x310_fpga_HG.bit or usrp_x300_fpga_XG.bit
    let expected = format!(
        "usrp_{}_fpga_{}.bit",
        usrp_model.to_lowercase(),
        variant.to_uppercase()
    );
    set.bitfiles
        .iter()
        .find(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_lowercase() == expected.to_lowercase())
                .unwrap_or(false)
        })
        .cloned()
}
