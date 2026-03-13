use super::ScanEntry;
use std::process::Command;

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // Docker Desktop data
    let docker_data = home.join("Library/Containers/com.docker.docker/Data");
    if docker_data.exists() {
        let size = super::dir_size(&docker_data);
        if size > 0 {
            entries.push(ScanEntry::new(
                "Docker Desktop Data".to_string(),
                docker_data,
                size,
                "",
            ));
        }
    }

    // Try docker system df for more info
    if let Ok(output) = Command::new("docker").args(["system", "df", "--format", "{{.Size}}"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                // Just note that docker is available; real size comes from the directory scan above
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_scan_no_panic() {
        let _entries = scan();
    }
}
