use super::{dir_size, ScanEntry};

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // User logs: ~/Library/Logs
    let user_logs = home.join("Library/Logs");
    if user_logs.exists() {
        let size = dir_size(&user_logs);
        if size > 0 {
            entries.push(ScanEntry::new(
                "User Log Files".to_string(),
                user_logs,
                size,
                "󰗀",
            ));
        }
    }

    // System logs: /var/log (read what we can)
    let sys_logs = std::path::PathBuf::from("/var/log");
    if sys_logs.exists() {
        let size = dir_size(&sys_logs);
        if size > 0 {
            entries.push(ScanEntry::new(
                "System Log Files".to_string(),
                sys_logs,
                size,
                "󰗀",
            ));
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
        // ~/Library/Logs should exist on macOS
        assert!(!entries.is_empty());
    }
}
