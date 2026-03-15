use super::{dir_size, ScanEntry};
use jwalk::WalkDir;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Global cargo registry cache
    let cargo_registry = home.join(".cargo/registry");
    if cargo_registry.exists() {
        let size = dir_size(&cargo_registry);
        if size > 0 {
            entries.push(ScanEntry::new(
                "Cargo Registry Cache".to_string(),
                cargo_registry,
                size,
                "",
            ));
        }
    }

    // Search for target/ directories in common project dirs
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
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "target" && entry.file_type().is_dir() {
                let entry_path = entry.path();
                // Verify it's a Cargo target dir by checking for parent Cargo.toml
                if let Some(p) = entry_path.parent() {
                    if p.join("Cargo.toml").exists() {
                        let size = dir_size(&entry_path);
                        if size > 1_000_000 {
                            let project_name = p
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            entries.push(ScanEntry::new(
                                format!("target/ ({})", project_name),
                                entry_path,
                                size,
                                "",
                            ));
                        }
                    }
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
    fn test_cargo_scan_no_panic() {
        let _entries = scan();
    }
}
