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
}

/// Compute total size of a directory recursively
pub fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Protected paths that must never be deleted
pub fn is_protected(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    let protected = [
        "/Documents",
        "/Desktop",
        "/Photos",
        "/Pictures",
        "/Movies",
        "/Music",
        "/.ssh",
        "/.gnupg",
        "/.env",
        "/.config",
        "/.zshrc",
        "/.bashrc",
        "/Library/Preferences",
        "/Library/Application Support",
        "/Library/Saved Application State",
        "/Library/LaunchAgents",
        "/Library/LaunchDaemons",
        "/Library/Keychains",
        "/oh-my-posh",
        "/oh-my-zsh",
        "/powerlevel10k",
        "/starship",
    ];
    protected.iter().any(|p| path_str.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_protected() {
        assert!(is_protected(Path::new("/Users/mazin/Documents/foo")));
        assert!(is_protected(Path::new("/Users/mazin/.ssh/id_rsa")));
        assert!(is_protected(Path::new("/Users/mazin/.gnupg/keys")));
        assert!(!is_protected(Path::new("/Users/mazin/Library/Caches/foo")));
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
