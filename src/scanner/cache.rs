use super::{dir_size, ScanEntry};
use std::fs;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // User caches: ~/Library/Caches
    let user_cache = home.join("Library/Caches");
    if user_cache.exists() {
        if let Ok(dirs) = fs::read_dir(&user_cache) {
            let total_size: u64 = dirs
                .filter_map(|e| e.ok())
                .map(|e| {
                    if e.path().is_dir() {
                        dir_size(&e.path())
                    } else {
                        e.metadata().map(|m| m.len()).unwrap_or(0)
                    }
                })
                .sum();

            if total_size > 0 {
                entries.push(ScanEntry::new(
                    "User Cache Files".to_string(),
                    user_cache,
                    total_size,
                    "󰃢",
                ));
            }
        }
    }

    // System caches: /Library/Caches
    let sys_cache = std::path::PathBuf::from("/Library/Caches");
    if sys_cache.exists() {
        let size = dir_size(&sys_cache);
        if size > 0 {
            entries.push(ScanEntry::new(
                "System Cache Files".to_string(),
                sys_cache,
                size,
                "",
            ));
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_scan_returns_entries() {
        let entries = scan();
        // Should find at least user caches on any macOS system
        assert!(!entries.is_empty(), "Expected at least one cache entry");
        for entry in &entries {
            assert!(entry.size > 0);
            assert!(!entry.name.is_empty());
        }
    }
}
