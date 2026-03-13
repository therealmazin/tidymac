use super::{dir_size, ScanEntry};
use std::path::PathBuf;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();

    // Homebrew cache: ~/Library/Caches/Homebrew
    let home = dirs::home_dir().unwrap_or_default();
    let brew_cache = home.join("Library/Caches/Homebrew");
    if brew_cache.exists() {
        let size = dir_size(&brew_cache);
        if size > 0 {
            entries.push(ScanEntry::new(
                "Homebrew Cache".to_string(),
                brew_cache,
                size,
                "󰃢",
            ));
        }
    }

    // Homebrew old versions: /usr/local/Cellar or /opt/homebrew/Cellar
    for cellar_path in &["/usr/local/Cellar", "/opt/homebrew/Cellar"] {
        let cellar = PathBuf::from(cellar_path);
        if cellar.exists() {
            // Count old versions (dirs with more than 1 version subdir)
            let mut old_size: u64 = 0;
            if let Ok(packages) = std::fs::read_dir(&cellar) {
                for pkg in packages.filter_map(|e| e.ok()) {
                    if let Ok(versions) = std::fs::read_dir(pkg.path()) {
                        let mut version_dirs: Vec<_> =
                            versions.filter_map(|e| e.ok()).collect();
                        if version_dirs.len() > 1 {
                            // Sort by name (version), keep newest, sum old ones
                            version_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
                            for old in &version_dirs[1..] {
                                old_size += dir_size(&old.path());
                            }
                        }
                    }
                }
            }
            if old_size > 0 {
                entries.push(ScanEntry::new(
                    "Brew Old Versions".to_string(),
                    cellar,
                    old_size,
                    "󰃢",
                ));
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brew_scan_no_panic() {
        // Should not panic even if brew is not installed
        let _entries = scan();
    }
}
