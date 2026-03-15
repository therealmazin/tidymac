use super::{dir_size, ScanEntry};
use jwalk::WalkDir;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Search common project directories for node_modules
    let search_dirs = [
        home.join("Documents"),
        home.join("Projects"),
        home.join("Developer"),
        home.join("Code"),
        home.join("src"),
    ];

    for search_dir in &search_dirs {
        if !search_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(search_dir)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "node_modules" && entry.file_type().is_dir() {
                let entry_path = entry.path();
                let size = dir_size(&entry_path);
                if size > 1_000_000 {
                    let parent_name = entry_path
                        .parent()
                        .and_then(|p| p.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    entries.push(ScanEntry::new(
                        format!("node_modules ({})", parent_name),
                        entry_path,
                        size,
                        "󰌠",
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
    fn test_node_scan_no_panic() {
        let _entries = scan();
    }
}
