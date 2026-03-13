use sysinfo::{Disks, System};

pub struct SystemStats {
    sys: System,
    disks: Disks,
    pub cpu_history: Vec<f32>,
}

impl SystemStats {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            cpu_history: Vec::with_capacity(60),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh(true);

        let cpu = self.sys.global_cpu_usage();
        self.cpu_history.push(cpu);
        if self.cpu_history.len() > 60 {
            self.cpu_history.remove(0);
        }
    }

    pub fn cpu_usage(&self) -> f32 {
        self.sys.global_cpu_usage()
    }

    pub fn cpu_count(&self) -> usize {
        self.sys.cpus().len()
    }

    pub fn memory_used(&self) -> u64 {
        self.sys.used_memory()
    }

    pub fn memory_total(&self) -> u64 {
        self.sys.total_memory()
    }

    pub fn memory_percent(&self) -> f32 {
        if self.sys.total_memory() == 0 {
            return 0.0;
        }
        (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0
    }

    pub fn disk_usage(&self) -> Vec<DiskInfo> {
        self.disks
            .list()
            .iter()
            .filter(|d| {
                let mp = d.mount_point().to_string_lossy();
                mp == "/" || mp.starts_with("/Volumes") || mp == "/System/Volumes/Data"
            })
            .map(|d| DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total: d.total_space(),
                available: d.available_space(),
            })
            .collect()
    }

    /// Returns sparkline characters for CPU history
    pub fn cpu_sparkline(&self) -> String {
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        self.cpu_history
            .iter()
            .map(|&v| {
                let idx = ((v / 100.0) * 7.0).round() as usize;
                bars[idx.min(7)]
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub available: u64,
}

impl DiskInfo {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used() as f32 / self.total as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_stats_creation() {
        let stats = SystemStats::new();
        assert!(stats.memory_total() > 0);
        assert!(stats.cpu_count() > 0);
    }

    #[test]
    fn test_memory_percent_calculation() {
        let stats = SystemStats::new();
        let pct = stats.memory_percent();
        assert!(pct > 0.0 && pct <= 100.0);
    }

    #[test]
    fn test_disk_info_used() {
        let info = DiskInfo {
            name: "test".to_string(),
            mount_point: "/".to_string(),
            total: 500_000_000_000,
            available: 200_000_000_000,
        };
        assert_eq!(info.used(), 300_000_000_000);
        assert!((info.percent() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_cpu_sparkline_empty() {
        let stats = SystemStats::new();
        // No history yet, should be empty
        assert!(stats.cpu_sparkline().is_empty());
    }

    #[test]
    fn test_disk_usage_has_root() {
        let stats = SystemStats::new();
        let disks = stats.disk_usage();
        // macOS should always have a root disk
        assert!(!disks.is_empty());
    }
}
