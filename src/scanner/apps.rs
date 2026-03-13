use super::{dir_size, ScanEntry};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub bundle_id: Option<String>,
    pub related_files: Vec<ScanEntry>,
}

pub fn scan_installed() -> Vec<AppInfo> {
    let mut apps = Vec::new();
    let applications = PathBuf::from("/Applications");

    if let Ok(entries) = fs::read_dir(&applications) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                let name = path
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let size = dir_size(&path);
                let bundle_id = read_bundle_id(&path);
                let related = find_related_files(&name, bundle_id.as_deref());

                apps.push(AppInfo {
                    name,
                    path,
                    size,
                    bundle_id,
                    related_files: related,
                });
            }
        }
    }

    apps.sort_by(|a, b| b.size.cmp(&a.size));
    apps
}

pub fn scan_orphans() -> Vec<ScanEntry> {
    let mut orphans = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Installed app bundle IDs
    let installed: Vec<String> = scan_installed()
        .iter()
        .filter_map(|a| a.bundle_id.clone())
        .collect();

    // Check Application Support for orphaned dirs
    let app_support = home.join("Library/Application Support");
    if let Ok(entries) = fs::read_dir(&app_support) {
        for entry in entries.filter_map(|e| e.ok()) {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            // If no installed app matches this directory name or bundle ID
            if !has_matching_app(&dir_name, &installed) {
                let size = dir_size(&entry.path());
                if size > 1_000_000 {
                    orphans.push(ScanEntry::new(
                        format!("Orphan: {}", dir_name),
                        entry.path(),
                        size,
                        "󰀲",
                    ));
                }
            }
        }
    }

    orphans.sort_by(|a, b| b.size.cmp(&a.size));
    orphans
}

fn read_bundle_id(app_path: &std::path::Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");
    if !plist_path.exists() {
        return None;
    }
    // Use defaults read to extract CFBundleIdentifier
    let output = std::process::Command::new("defaults")
        .args(["read", &plist_path.to_string_lossy(), "CFBundleIdentifier"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn find_related_files(app_name: &str, bundle_id: Option<&str>) -> Vec<ScanEntry> {
    let mut related = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    let search_dirs = [
        ("Preferences", home.join("Library/Preferences")),
        ("Application Support", home.join("Library/Application Support")),
        ("Caches", home.join("Library/Caches")),
        ("Logs", home.join("Library/Logs")),
        ("Saved Application State", home.join("Library/Saved Application State")),
    ];

    for (category, dir) in &search_dirs {
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                let matches = name.to_lowercase().contains(&app_name.to_lowercase())
                    || bundle_id
                        .map(|bid| name.contains(bid))
                        .unwrap_or(false);

                if matches {
                    let size = if entry.path().is_dir() {
                        dir_size(&entry.path())
                    } else {
                        entry.metadata().map(|m| m.len()).unwrap_or(0)
                    };

                    if size > 0 {
                        related.push(ScanEntry::new(
                            format!("{} ({})", name, category),
                            entry.path(),
                            size,
                            "󰀲",
                        ));
                    }
                }
            }
        }
    }

    related
}

fn has_matching_app(dir_name: &str, installed_bundle_ids: &[String]) -> bool {
    // Check if any installed app's bundle ID contains this dir name
    let lower = dir_name.to_lowercase();
    installed_bundle_ids
        .iter()
        .any(|bid| bid.to_lowercase().contains(&lower) || lower.contains(&bid.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_installed_no_panic() {
        let apps = scan_installed();
        // /Applications should have at least some apps
        assert!(!apps.is_empty());
    }

    #[test]
    fn test_scan_orphans_no_panic() {
        let _orphans = scan_orphans();
    }
}
