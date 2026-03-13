pub fn scan() -> Vec<super::ScanEntry> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size: u64,
}
