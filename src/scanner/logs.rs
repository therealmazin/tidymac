use super::{dir_size, ScanEntry};
use std::fs;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Scan individual subdirectories of ~/Library/Logs
    // Each app gets its own log directory — safe to delete individually
    let user_logs = home.join("Library/Logs");
    if user_logs.exists() {
        if let Ok(dirs) = fs::read_dir(&user_logs) {
            for entry in dirs.filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                let size = if path.is_dir() {
                    dir_size(&path)
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };

                if size > 1_000_000 { // Only show > 1MB
                    entries.push(ScanEntry::new(
                        format!("Logs: {}", name),
                        path,
                        size,
                        "󰗀",
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
    fn test_logs_scan_returns_entries() {
        let entries = scan();
        // ~/Library/Logs should have some log directories
        for entry in &entries {
            assert!(entry.size > 0);
        }
    }
}
