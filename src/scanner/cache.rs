use super::{dir_size, ScanEntry};
use std::fs;

/// Known-safe cache directories that can be deleted without breaking apps.
/// Apps will regenerate these on next launch.
const SAFE_CACHE_PATTERNS: &[&str] = &[
    // Browsers
    "com.apple.Safari",
    "com.google.Chrome",
    "com.brave.Browser",
    "com.microsoft.Edge",
    "org.mozilla.firefox",
    "org.chromium.Chromium",
    // Apple apps
    "com.apple.Music",
    "com.apple.podcasts",
    "com.apple.Safari.SafeBrowsing",
    "com.apple.appstoreagent",
    "com.apple.nsurlsessiond",
    "com.apple.appstore",
    "com.apple.iLifeMediaBrowser",
    "com.apple.mediaanalysisd",
    // Media & streaming
    "com.spotify.client",
    "com.apple.tv",
    // System caches (safe to clear)
    "CloudKit",
    "storeassetsd",
    "FamilyCircle",
    "com.apple.cache_delete",
    "com.apple.bird",
];

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Scan individual subdirectories of ~/Library/Caches
    // Only include entries matching safe patterns
    let user_cache = home.join("Library/Caches");
    if user_cache.exists() {
        if let Ok(dirs) = fs::read_dir(&user_cache) {
            for entry in dirs.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();

                // Only include if it matches a known-safe pattern
                let is_safe = SAFE_CACHE_PATTERNS.iter().any(|pattern| {
                    name.contains(pattern)
                });

                if !is_safe {
                    continue;
                }

                let path = entry.path();
                let size = if path.is_dir() {
                    dir_size(&path)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };

                if size > 100_000 { // Only show > 100KB
                    entries.push(ScanEntry::new(
                        format!("Cache: {}", name),
                        path,
                        size,
                        "󰃢",
                    ));
                }
            }
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
        // May or may not find safe caches depending on the system
        for entry in &entries {
            assert!(entry.size > 0);
            assert!(!entry.name.is_empty());
            // Verify no dangerous paths
            let path_str = entry.path.to_string_lossy();
            assert!(!path_str.contains("oh-my-posh"));
            assert!(!path_str.contains("oh-my-zsh"));
            assert!(!path_str.contains("Homebrew"));
        }
    }
}
