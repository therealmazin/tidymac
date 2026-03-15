use super::{dir_size, ScanEntry};

pub fn scan() -> Vec<ScanEntry> {
    let mut results = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();
    let trash_path = home.join(".Trash");

    if trash_path.exists() {
        let size = dir_size(&trash_path);
        if size > 0 {
            results.push(ScanEntry::new(
                "Trash Bin".to_string(),
                trash_path,
                size,
                "󰩹",
            ));
        }
    }

    results
}
