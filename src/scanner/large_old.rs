use super::ScanEntry;
use std::time::{Duration, SystemTime};

const MIN_SIZE: u64 = 400_000_000; // 400 MB
const MAX_AGE: Duration = Duration::from_secs(90 * 24 * 60 * 60); // 3 months
const MAX_DEPTH: usize = 4;

pub fn scan() -> Vec<ScanEntry> {
    let home = dirs::home_dir().unwrap_or_default();
    let mut results = Vec::new();
    let now = SystemTime::now();

    let search_dirs = [
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Desktop"),
        home.join("Movies"),
        home.join("Music"),
        home.join("Pictures"),
    ];

    for dir in &search_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = jwalk::WalkDir::new(dir)
            .max_depth(MAX_DEPTH)
            .skip_hidden(false)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            if !entry.file_type().is_file() {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = meta.len();
            if size < MIN_SIZE {
                continue;
            }

            // Check last modified time
            let modified = match meta.modified() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let age = match now.duration_since(modified) {
                Ok(d) => d,
                Err(_) => continue,
            };

            if age < MAX_AGE {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let icon = file_icon(&name);
            let age_str = format_age(age);

            results.push(ScanEntry::new_unselected(
                format!("{} ({})", name, age_str),
                entry.path().to_path_buf(),
                size,
                icon,
            ));
        }
    }

    results.sort_by(|a, b| b.size.cmp(&a.size));
    results
}

fn format_age(age: Duration) -> String {
    let days = age.as_secs() / 86400;
    if days > 365 {
        format!("{} year{} ago", days / 365, if days / 365 > 1 { "s" } else { "" })
    } else if days > 30 {
        format!("{} month{} ago", days / 30, if days / 30 > 1 { "s" } else { "" })
    } else {
        format!("{} day{} ago", days, if days > 1 { "s" } else { "" })
    }
}

fn file_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.ends_with(".mov") || lower.ends_with(".mp4") || lower.ends_with(".avi") || lower.ends_with(".mkv") {
        "󰎁"
    } else if lower.ends_with(".zip") || lower.ends_with(".tar") || lower.ends_with(".gz") || lower.ends_with(".rar") {
        "󰗀"
    } else if lower.ends_with(".dmg") || lower.ends_with(".iso") {
        "󰗮"
    } else if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "󰋩"
    } else {
        ""
    }
}
