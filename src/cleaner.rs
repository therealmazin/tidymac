use std::path::Path;
use crate::scanner::is_protected;

#[derive(Debug)]
pub enum CleanError {
    ProtectedPath(String),
    TrashError(String),
}

impl std::fmt::Display for CleanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanError::ProtectedPath(p) => write!(f, "Refusing to delete protected path: {}", p),
            CleanError::TrashError(e) => write!(f, "Failed to move to trash: {}", e),
        }
    }
}

pub fn move_to_trash(path: &Path) -> Result<(), CleanError> {
    if is_protected(path) {
        return Err(CleanError::ProtectedPath(
            path.to_string_lossy().to_string(),
        ));
    }

    if !path.exists() {
        return Ok(()); // Already gone
    }

    trash::delete(path).map_err(|e| CleanError::TrashError(e.to_string()))
}

pub fn clean_selected(entries: &[crate::scanner::ScanEntry]) -> Vec<Result<String, CleanError>> {
    entries
        .iter()
        .filter(|e| e.selected)
        .map(|entry| {
            move_to_trash(&entry.path)?;
            Ok(format!("Moved to trash: {}", entry.name))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_protected_path_rejected() {
        let home = dirs::home_dir().unwrap();
        // Can't delete the Documents directory itself
        let result = move_to_trash(&home.join("Documents"));
        assert!(matches!(result, Err(CleanError::ProtectedPath(_))));
    }

    #[test]
    fn test_nonexistent_path_ok() {
        let result = move_to_trash(Path::new("/tmp/tidymac_nonexistent_test_path_12345"));
        assert!(result.is_ok());
    }
}
