use sysinfo::{Disks, Networks, Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub memory: u64,
    pub cpu_usage: f32,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub download_speed: u64,
    pub upload_speed: u64,
    pub download_top: u64,
    pub upload_top: u64,
    pub download_total: u64,
    pub upload_total: u64,
}

pub struct SystemStats {
    sys: System,
    disks: Disks,
    networks: Networks,
    pub cpu_history: Vec<f32>,
    pub per_core_usage: Vec<f32>,
    pub listening_ports: Vec<PortInfo>,
    pub network_stats: NetworkStats,
    slow_tick: u32, // counter to throttle expensive ops
}

impl SystemStats {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let per_core = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let mut stats = Self {
            sys,
            disks,
            networks,
            cpu_history: Vec::with_capacity(120),
            per_core_usage: per_core,
            listening_ports: Vec::new(),
            network_stats: NetworkStats::default(),
            slow_tick: 0,
        };
        stats.sys.refresh_processes(ProcessesToUpdate::All, false);
        stats.refresh_ports();
        stats
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh(true);

        let cpu = self.sys.global_cpu_usage();
        self.cpu_history.push(cpu);
        if self.cpu_history.len() > 120 {
            self.cpu_history.remove(0);
        }

        self.per_core_usage = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        self.networks.refresh(true);
        self.refresh_network();

        // Expensive ops (lsof + process refresh) every 5 ticks instead of every tick
        self.slow_tick += 1;
        if self.slow_tick >= 5 {
            self.slow_tick = 0;
            self.sys.refresh_processes(ProcessesToUpdate::All, false);
            self.refresh_ports();
        }
    }

    fn refresh_network(&mut self) {
        let mut down: u64 = 0;
        let mut up: u64 = 0;

        for (_name, data) in self.networks.list() {
            down += data.received();
            up += data.transmitted();
        }

        // sysinfo's received()/transmitted() return bytes since last refresh
        self.network_stats.download_speed = down;
        self.network_stats.upload_speed = up;

        self.network_stats.download_total += down;
        self.network_stats.upload_total += up;

        if down > self.network_stats.download_top {
            self.network_stats.download_top = down;
        }
        if up > self.network_stats.upload_top {
            self.network_stats.upload_top = up;
        }
    }

    fn refresh_ports(&mut self) {
        self.listening_ports.clear();
        let output = std::process::Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => return,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut seen_ports = std::collections::HashSet::new();

        for line in stdout.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 9 {
                continue;
            }

            let process_name = fields[0].replace("\\x20", " ");
            let pid: u32 = match fields[1].parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let name_field = fields[fields.len() - 2];
            let port: u16 = match name_field.rsplit(':').next().and_then(|p| p.parse().ok()) {
                Some(p) => p,
                None => continue,
            };

            if seen_ports.insert(port) {
                let proc = self.sys.process(Pid::from(pid as usize));
                let memory = proc.map(|p| p.memory()).unwrap_or(0);
                let cpu_usage = proc.map(|p| p.cpu_usage()).unwrap_or(0.0);

                self.listening_ports.push(PortInfo {
                    port,
                    pid,
                    process_name,
                    memory,
                    cpu_usage,
                });
            }
        }

        self.listening_ports.sort_by_key(|p| p.port);
    }

    pub fn cpu_usage(&self) -> f32 {
        self.sys.global_cpu_usage()
    }

    pub fn cpu_count(&self) -> usize {
        self.sys.cpus().len()
    }

    pub fn per_core(&self) -> &[f32] {
        &self.per_core_usage
    }

    pub fn memory_used(&self) -> u64 {
        self.sys.used_memory()
    }

    pub fn memory_total(&self) -> u64 {
        self.sys.total_memory()
    }

    pub fn memory_available(&self) -> u64 {
        self.sys.available_memory()
    }

    pub fn memory_free(&self) -> u64 {
        self.sys.free_memory()
    }

    pub fn swap_used(&self) -> u64 {
        self.sys.used_swap()
    }

    pub fn swap_total(&self) -> u64 {
        self.sys.total_swap()
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

    pub fn cpu_history_u64(&self) -> Vec<u64> {
        self.cpu_history
            .iter()
            .map(|&v| (v * 100.0) as u64)
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
        assert!(stats.cpu_sparkline().is_empty());
    }

    #[test]
    fn test_disk_usage_has_root() {
        let stats = SystemStats::new();
        let disks = stats.disk_usage();
        assert!(!disks.is_empty());
    }

    #[test]
    fn test_per_core_populated() {
        let stats = SystemStats::new();
        assert!(!stats.per_core().is_empty());
    }
}
