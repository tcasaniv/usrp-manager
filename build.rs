use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=locales/en.yml");
    println!("cargo:rerun-if-changed=locales/es.yml");
    println!("cargo:rerun-if-changed=src");

    let en_yaml: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string("locales/en.yml").expect("Failed to read locales/en.yml"),
    )
    .expect("Invalid YAML in locales/en.yml");

    let es_yaml: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string("locales/es.yml").expect("Failed to read locales/es.yml"),
    )
    .expect("Invalid YAML in locales/es.yml");

    let mut used_keys = BTreeSet::new();
    collect_keys_from_dir(Path::new("src"), &mut used_keys);

    let mut missing_errors = Vec::new();

    for key in &used_keys {
        if !has_key(&en_yaml, key) {
            missing_errors.push(format!("Missing key in locales/en.yml: '{}'", key));
        }
        if !has_key(&es_yaml, key) {
            missing_errors.push(format!("Missing key in locales/es.yml: '{}'", key));
        }
    }

    if !missing_errors.is_empty() {
        panic!(
            "\n\n=======================================================\n\
             ❌ I18N COMPILE-TIME VERIFICATION FAILED:\n\
             =======================================================\n\
             {}\n\
             Total missing: {}\n\
             Please add the missing translation keys to both locales/en.yml and locales/es.yml.\n\
             =======================================================\n\n",
            missing_errors.join("\n"),
            missing_errors.len()
        );
    }
}

fn collect_keys_from_dir(dir: &Path, keys: &mut BTreeSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_keys_from_dir(&path, keys);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    extract_t_keys(&content, keys);
                }
            }
        }
    }
}

fn extract_t_keys(content: &str, keys: &mut BTreeSet<String>) {
    // Only match `t!("key.name"` or `t!( "key.name"` where key contains letters, numbers and underscores
    let mut rest = content;
    while let Some(pos) = rest.find("t!(") {
        let after = rest[pos + 3..].trim_start();
        if after.starts_with('"') {
            let inside = &after[1..];
            if let Some(end_quote) = inside.find('"') {
                let candidate = &inside[..end_quote];
                let is_valid_key = candidate.contains('.')
                    && candidate.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_');
                if is_valid_key {
                    keys.insert(candidate.to_string());
                }
            }
        }
        rest = &rest[pos + 3..];
    }
}

fn has_key(yaml: &serde_yaml::Value, key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();
    let mut curr = yaml;
    for part in parts {
        match curr.get(part) {
            Some(next) => curr = next,
            None => return false,
        }
    }
    true
}
