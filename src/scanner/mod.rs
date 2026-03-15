pub mod cache;
pub mod logs;
pub mod brew;
pub mod xcode;
pub mod docker;
pub mod node;
pub mod cargo;
pub mod apps;
pub mod trash;
pub mod space;
pub mod large_old;

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ScanEntry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub icon: &'static str,
    pub selected: bool,
}

impl ScanEntry {
    pub fn new(name: String, path: PathBuf, size: u64, icon: &'static str) -> Self {
        Self {
            name,
            path,
            size,
            icon,
            selected: true,
        }
    }

    pub fn new_unselected(name: String, path: PathBuf, size: u64, icon: &'static str) -> Self {
        Self {
            name,
            path,
            size,
            icon,
            selected: false,
        }
    }
}

/// Compute total size of a directory recursively (parallel via jwalk)
pub fn dir_size(path: &std::path::Path) -> u64 {
    jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Protected paths that must never be deleted.
/// Only blocks deleting the directory itself, not files inside it.
pub fn is_protected(path: &std::path::Path) -> bool {
    let home = dirs::home_dir().unwrap_or_default();
    let protected_dirs = [
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Photos"),
        home.join("Pictures"),
        home.join("Movies"),
        home.join("Music"),
        home.join("Downloads"),
        home.join(".ssh"),
        home.join(".gnupg"),
        home.join(".config"),
        home.join("Library"),
        home.join("Library/Preferences"),
        home.join("Library/Application Support"),
        home.join("Library/Saved Application State"),
        home.join("Library/LaunchAgents"),
        home.join("Library/LaunchDaemons"),
        home.join("Library/Keychains"),
    ];
    protected_dirs.iter().any(|p| path == p.as_path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_protected() {
        let home = dirs::home_dir().unwrap();
        // Top-level protected dirs — can't delete these
        assert!(is_protected(&home.join("Documents")));
        assert!(is_protected(&home.join(".ssh")));
        assert!(is_protected(&home.join(".gnupg")));
        assert!(is_protected(&home.join("Library")));
        // Files INSIDE protected dirs — can delete these
        assert!(!is_protected(&home.join("Documents/movie.mp4")));
        assert!(!is_protected(&home.join("Library/Caches/foo")));
        // Random paths — not protected
        assert!(!is_protected(Path::new("/tmp/test")));
    }

    #[test]
    fn test_scan_entry_new() {
        let entry = ScanEntry::new(
            "Test".to_string(),
            PathBuf::from("/tmp/test"),
            1024,
            "󰃢",
        );
        assert_eq!(entry.name, "Test");
        assert_eq!(entry.size, 1024);
        assert!(entry.selected);
    }
}
