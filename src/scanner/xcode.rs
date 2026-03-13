use super::{dir_size, ScanEntry};

pub fn scan() -> Vec<ScanEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();

    // DerivedData
    let derived = home.join("Library/Developer/Xcode/DerivedData");
    if derived.exists() {
        let size = dir_size(&derived);
        if size > 0 {
            entries.push(ScanEntry::new(
                "Xcode DerivedData".to_string(),
                derived,
                size,
                "󰅐",
            ));
        }
    }

    // Archives
    let archives = home.join("Library/Developer/Xcode/Archives");
    if archives.exists() {
        let size = dir_size(&archives);
        if size > 0 {
            entries.push(ScanEntry::new(
                "Xcode Archives".to_string(),
                archives,
                size,
                "󰅐",
            ));
        }
    }

    // iOS Device Support
    let device_support = home.join("Library/Developer/Xcode/iOS DeviceSupport");
    if device_support.exists() {
        let size = dir_size(&device_support);
        if size > 0 {
            entries.push(ScanEntry::new(
                "iOS Device Support".to_string(),
                device_support,
                size,
                "󰅐",
            ));
        }
    }

    // CoreSimulator devices
    let simulators = home.join("Library/Developer/CoreSimulator/Devices");
    if simulators.exists() {
        let size = dir_size(&simulators);
        if size > 0 {
            entries.push(ScanEntry::new(
                "iOS Simulators".to_string(),
                simulators,
                size,
                "󰅐",
            ));
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcode_scan_no_panic() {
        let _entries = scan();
    }
}
