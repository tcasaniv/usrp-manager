use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

/// Download a file from `url` to `dest`, showing a progress bar.
pub fn download_with_progress(url: &str, dest: &Path, name: &str) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("usrp-manager/0.1")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let mut resp = client.get(url).send().context("HTTP request failed")?;

    let total = resp.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .unwrap()
            .progress_chars("█░"),
    );
    pb.set_message(name.to_string());

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(dest).context("Cannot create output file")?;
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 8192];

    loop {
        use std::io::Read;
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();
    Ok(())
}

/// Verify SHA256 checksum. `expected_hex` is the hex string.
pub fn verify_sha256(file: &Path, expected_hex: &str) -> bool {
    let content = match std::fs::read(file) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    let actual = hex::encode(result);
    actual.eq_ignore_ascii_case(expected_hex.trim())
}

/// Extract a zip file to a destination directory.
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dest_dir)?;
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_path = dest_dir.join(entry.name());

        if entry.is_dir() {
            std::fs::create_dir_all(&entry_path)?;
        } else {
            if let Some(parent) = entry_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = std::fs::File::create(&entry_path)?;
            std::io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}
