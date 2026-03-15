use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use crate::scanner::ScanEntry;
use crate::scanner::apps::AppInfo;
use crate::scanner::space::{SpaceNode, SpaceVisibleItem};
use std::collections::HashMap;
use std::path::PathBuf;

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Parse mdls date format "2025-09-16 00:52:11 +0000" into SystemTime
fn parse_mdls_date(s: &str) -> Option<std::time::SystemTime> {
    // Format: "YYYY-MM-DD HH:MM:SS +0000"
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 2 { return None; }

    let date_parts: Vec<u64> = parts[0].split('-').filter_map(|p| p.parse().ok()).collect();
    let time_parts: Vec<u64> = parts[1].split(':').filter_map(|p| p.parse().ok()).collect();

    if date_parts.len() != 3 || time_parts.len() != 3 { return None; }

    let (year, month, day) = (date_parts[0], date_parts[1], date_parts[2]);
    let (hour, min, sec) = (time_parts[0], time_parts[1], time_parts[2]);

    // Approximate: convert to seconds since Unix epoch
    // Not perfectly accurate but good enough for "months ago" comparison
    let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4
        + match month {
            1 => 0, 2 => 31, 3 => 59, 4 => 90, 5 => 120, 6 => 151,
            7 => 181, 8 => 212, 9 => 243, 10 => 273, 11 => 304, 12 => 334,
            _ => 0,
        } + day - 1;

    let secs = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// Format last_used as human-readable "X months ago" or "Never opened"
pub fn format_last_used(last_used: &Option<String>) -> String {
    match last_used {
        None => "Never opened".to_string(),
        Some(date_str) => {
            match parse_mdls_date(date_str) {
                Some(last) => {
                    let age = std::time::SystemTime::now()
                        .duration_since(last)
                        .unwrap_or_default();
                    let days = age.as_secs() / 86400;
                    if days > 365 {
                        format!("{} year{} ago", days / 365, if days / 365 > 1 { "s" } else { "" })
                    } else if days > 30 {
                        format!("{} month{} ago", days / 30, if days / 30 > 1 { "s" } else { "" })
                    } else if days > 0 {
                        format!("{} day{} ago", days, if days > 1 { "s" } else { "" })
                    } else {
                        "Today".to_string()
                    }
                }
                None => "Unknown".to_string(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    SmartScan,
    Apps,
    SpaceLens,
    LargeOld,
    Config,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Home,
            Screen::SmartScan,
            Screen::Apps,
            Screen::SpaceLens,
            Screen::LargeOld,
            Screen::Config,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Screen::Home => "󰣇 Home",
            Screen::SmartScan => "󰃢 Smart Scan",
            Screen::Apps => "󰀲 Apps",
            Screen::SpaceLens => "󰋊 Space Lens",
            Screen::LargeOld => "󰱼 Large & Old",
            Screen::Config => " Cfg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Main,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    None,
    CleanScan,
    UninstallApp,
    KillPort,
    Cleaning,          // progress bar during clean
    CleanDone,         // finished summary
    DeleteSpaceItem,   // confirm delete from Space Lens
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppView {
    All,
    Unused,
    Leftovers,
}

#[derive(Debug, Clone)]
pub struct SmartScanCategory {
    pub name: String,
    pub icon: String,
    pub entries: Vec<ScanEntry>,
    pub total_size: u64,
    pub expanded: bool,
    pub selected: bool,
}

#[derive(Debug, Clone)]
pub struct ScanStep {
    pub name: String,
    pub done: bool,
}

pub enum ScanMessage {
    ScanResults(Vec<ScanEntry>),
    SmartScanResults(Vec<SmartScanCategory>),
    SmartScanProgress(String), // step name just completed
    CleanProgress { msg: String, size_freed: u64 },
    CleanDone,
    AppList(Vec<AppInfo>),
    OrphanResults(Vec<ScanEntry>),
    SpaceTreeWithCache(Vec<SpaceNode>, HashMap<PathBuf, u64>),
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub sidebar_index: usize,
    pub focus: Focus,
    pub last_tick: Instant,
    pub tick_rate: Duration,
    // Scan results (used by LargeOld)
    pub scan_results: Vec<ScanEntry>,
    pub scan_list_index: usize,
    pub scan_list_state: ListState,
    pub scanning: bool,
    // Smart Scan
    pub smart_scan_categories: Vec<SmartScanCategory>,
    pub smart_scan_index: usize,
    pub scan_steps: Vec<ScanStep>,
    // Apps
    pub app_list: Vec<AppInfo>,
    pub app_list_index: usize,
    pub app_list_state: ListState,
    pub app_view: AppView,
    pub show_orphans: bool,
    pub orphan_results: Vec<ScanEntry>,
    pub orphan_list_state: ListState,
    pub unused_apps: Vec<AppInfo>,
    pub unused_list_state: ListState,
    // Space Lens (tree)
    pub space_tree: Vec<SpaceNode>,
    pub space_visible: Vec<SpaceVisibleItem>,
    pub space_list_index: usize,
    pub space_list_state: ListState,
    pub space_expanding: bool,
    pub space_size_cache: HashMap<PathBuf, u64>,
    // Confirm dialog + cleaning progress
    pub confirm_kind: ConfirmKind,
    pub last_clean_results: Vec<String>,
    pub clean_progress: usize,     // items cleaned so far
    pub clean_total: usize,        // total items to clean
    pub clean_size_freed: u64,     // bytes freed so far
    // Settings
    pub safe_mode: bool,
    pub config_index: usize,
    // Ports (Home screen)
    pub port_list_index: usize,
    pub port_list_state: ListState,
    pub kill_port_info: Option<crate::system::PortInfo>,
    pub delete_space_info: Option<(String, PathBuf, u64, Vec<usize>)>, // (name, path, size, tree_path)
    // Async scan state
    pub scan_receiver: Option<mpsc::Receiver<ScanMessage>>,
    pub scan_status: String,
    pub spinner_frame: usize,
    pub spinner_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            screen: Screen::Home,
            sidebar_index: 0,
            focus: Focus::Sidebar,
            last_tick: Instant::now(),
            tick_rate: Duration::from_secs(1),
            scan_results: Vec::new(),
            scan_list_index: 0,
            scan_list_state: ListState::default(),
            scanning: false,
            smart_scan_categories: Vec::new(),
            smart_scan_index: 0,
            scan_steps: Vec::new(),
            app_list: Vec::new(),
            app_list_index: 0,
            app_list_state: ListState::default(),
            app_view: AppView::All,
            show_orphans: false,
            orphan_results: Vec::new(),
            orphan_list_state: ListState::default(),
            unused_apps: Vec::new(),
            unused_list_state: ListState::default(),
            space_tree: Vec::new(),
            space_visible: Vec::new(),
            space_list_index: 0,
            space_list_state: ListState::default(),
            space_expanding: false,
            space_size_cache: HashMap::new(),
            confirm_kind: ConfirmKind::None,
            last_clean_results: Vec::new(),
            clean_progress: 0,
            clean_total: 0,
            clean_size_freed: 0,
            safe_mode: true,
            config_index: 0,
            port_list_index: 0,
            port_list_state: ListState::default(),
            kill_port_info: None,
            delete_space_info: None,
            scan_receiver: None,
            scan_status: String::new(),
            spinner_frame: 0,
            spinner_tick: Instant::now(),
        }
    }

    pub fn is_confirming(&self) -> bool {
        self.confirm_kind != ConfirmKind::None
    }

    pub fn spinner_char(&self) -> char {
        SPINNER_CHARS[self.spinner_frame % SPINNER_CHARS.len()]
    }

    pub fn tick_spinner(&mut self) {
        if self.spinner_tick.elapsed() >= Duration::from_millis(80) {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_CHARS.len();
            self.spinner_tick = Instant::now();
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_sidebar(&mut self) {
        let screens = Screen::all();
        self.sidebar_index = (self.sidebar_index + 1) % screens.len();
        self.screen = screens[self.sidebar_index];
        self.scan_results.clear();
        self.scan_list_index = 0;
        self.scan_list_state.select(None);
    }

    pub fn prev_sidebar(&mut self) {
        let screens = Screen::all();
        if self.sidebar_index == 0 {
            self.sidebar_index = screens.len() - 1;
        } else {
            self.sidebar_index -= 1;
        }
        self.screen = screens[self.sidebar_index];
        self.scan_results.clear();
        self.scan_list_index = 0;
        self.scan_list_state.select(None);
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Main,
            Focus::Main => Focus::Sidebar,
        };
    }

    // Generic list navigation (used by LargeOld)
    pub fn next_list_item(&mut self) {
        if !self.scan_results.is_empty() {
            self.scan_list_index = (self.scan_list_index + 1) % self.scan_results.len();
            self.scan_list_state.select(Some(self.scan_list_index));
        }
    }

    pub fn prev_list_item(&mut self) {
        if !self.scan_results.is_empty() {
            if self.scan_list_index == 0 {
                self.scan_list_index = self.scan_results.len() - 1;
            } else {
                self.scan_list_index -= 1;
            }
            self.scan_list_state.select(Some(self.scan_list_index));
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(entry) = self.scan_results.get_mut(self.scan_list_index) {
            entry.selected = !entry.selected;
        }
    }

    pub fn selected_size(&self) -> u64 {
        self.scan_results
            .iter()
            .filter(|e| e.selected)
            .map(|e| e.size)
            .sum()
    }

    // Smart Scan
    pub fn run_smart_scan(&mut self) {
        if self.scanning { return; }
        self.scanning = true;
        self.smart_scan_categories.clear();
        self.smart_scan_index = 0;
        self.scan_list_state.select(None);
        self.scan_status = "Running smart scan...".to_string();

        self.scan_steps = vec![
            ScanStep { name: "Scanning caches...".to_string(), done: false },
            ScanStep { name: "Scanning logs...".to_string(), done: false },
            ScanStep { name: "Scanning brew leftovers...".to_string(), done: false },
            ScanStep { name: "Scanning Trash...".to_string(), done: false },
        ];

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let mut categories = Vec::new();

            // System Junk
            let mut junk = Vec::new();

            let cache = crate::scanner::cache::scan();
            junk.extend(cache);
            let _ = tx.send(ScanMessage::SmartScanProgress("Scanning caches...".to_string()));

            let logs = crate::scanner::logs::scan();
            junk.extend(logs);
            let _ = tx.send(ScanMessage::SmartScanProgress("Scanning logs...".to_string()));

            let brew = crate::scanner::brew::scan();
            junk.extend(brew);
            let _ = tx.send(ScanMessage::SmartScanProgress("Scanning brew leftovers...".to_string()));

            let junk_size: u64 = junk.iter().map(|e| e.size).sum();
            categories.push(SmartScanCategory {
                name: "System Junk".to_string(),
                icon: "󰃢".to_string(),
                entries: junk,
                total_size: junk_size,
                expanded: false,
                selected: true,
            });

            // Trash Bins
            let trash = crate::scanner::trash::scan();
            let _ = tx.send(ScanMessage::SmartScanProgress("Scanning Trash...".to_string()));

            let trash_size: u64 = trash.iter().map(|e| e.size).sum();
            categories.push(SmartScanCategory {
                name: "Trash Bins".to_string(),
                icon: "󰩹".to_string(),
                entries: trash,
                total_size: trash_size,
                expanded: false,
                selected: true,
            });

            let _ = tx.send(ScanMessage::SmartScanResults(categories));
        });
    }

    pub fn smart_scan_total_items(&self) -> usize {
        let mut count = 0;
        for cat in &self.smart_scan_categories {
            count += 1; // category header
            if cat.expanded {
                count += cat.entries.len();
            }
        }
        count
    }

    pub fn next_smart_scan_item(&mut self) {
        let total = self.smart_scan_total_items();
        if total > 0 {
            self.smart_scan_index = (self.smart_scan_index + 1) % total;
            self.scan_list_state.select(Some(self.smart_scan_index));
        }
    }

    pub fn prev_smart_scan_item(&mut self) {
        let total = self.smart_scan_total_items();
        if total > 0 {
            if self.smart_scan_index == 0 {
                self.smart_scan_index = total - 1;
            } else {
                self.smart_scan_index -= 1;
            }
            self.scan_list_state.select(Some(self.smart_scan_index));
        }
    }

    pub fn toggle_smart_scan_item(&mut self) {
        // Find which category/entry the current index points to
        let mut idx = 0;
        for cat in &mut self.smart_scan_categories {
            if idx == self.smart_scan_index {
                cat.selected = !cat.selected;
                // Toggle all entries too
                for entry in &mut cat.entries {
                    entry.selected = cat.selected;
                }
                return;
            }
            idx += 1;
            if cat.expanded {
                for entry in &mut cat.entries {
                    if idx == self.smart_scan_index {
                        entry.selected = !entry.selected;
                        return;
                    }
                    idx += 1;
                }
            }
        }
    }

    pub fn toggle_smart_scan_expand(&mut self) {
        let mut idx = 0;
        for cat in &mut self.smart_scan_categories {
            if idx == self.smart_scan_index {
                cat.expanded = !cat.expanded;
                return;
            }
            idx += 1;
            if cat.expanded {
                idx += cat.entries.len();
            }
        }
    }

    pub fn smart_scan_selected_size(&self) -> u64 {
        self.smart_scan_categories
            .iter()
            .flat_map(|c| &c.entries)
            .filter(|e| e.selected)
            .map(|e| e.size)
            .sum()
    }

    pub fn clean_smart_scan(&mut self) {
        let entries: Vec<ScanEntry> = self.smart_scan_categories
            .iter()
            .flat_map(|c| &c.entries)
            .filter(|e| e.selected)
            .cloned()
            .collect();

        self.clean_total = entries.len();
        self.clean_progress = 0;
        self.clean_size_freed = 0;
        self.last_clean_results.clear();
        self.confirm_kind = ConfirmKind::Cleaning;

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            for entry in &entries {
                let size = entry.size;
                let result = crate::cleaner::move_to_trash(&entry.path);
                let msg = match result {
                    Ok(()) => format!("✓ {} ({})", entry.name, bytesize::ByteSize(size)),
                    Err(e) => format!("✗ {}: {}", entry.name, e),
                };
                let _ = tx.send(ScanMessage::CleanProgress { msg, size_freed: size });
            }
            let _ = tx.send(ScanMessage::CleanDone);
        });
    }

    // Large & Old Files
    pub fn run_large_old_scan(&mut self) {
        if self.scanning { return; }
        self.scanning = true;
        self.scan_results.clear();
        self.scan_list_index = 0;
        self.scan_list_state.select(None);
        self.scan_status = "Searching for large, old files...".to_string();

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let results = crate::scanner::large_old::scan();
            let _ = tx.send(ScanMessage::ScanResults(results));
        });
    }

    // Space Lens
    pub fn run_space_scan(&mut self) {
        if self.scanning { return; }
        self.scanning = true;
        self.space_tree.clear();
        self.space_visible.clear();
        self.space_size_cache.clear();
        self.space_list_index = 0;
        self.space_list_state.select(None);
        self.scan_status = "Scanning disk usage...".to_string();

        self.scan_steps = vec![
            ScanStep { name: "Walking file system...".to_string(), done: false },
            ScanStep { name: "Building directory tree...".to_string(), done: false },
        ];

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let _ = tx.send(ScanMessage::SmartScanProgress("Walking file system...".to_string()));
            let (tree, cache) = crate::scanner::space::scan_home_tree_with_cache();
            let _ = tx.send(ScanMessage::SmartScanProgress("Building directory tree...".to_string()));
            let _ = tx.send(ScanMessage::SpaceTreeWithCache(tree, cache));
        });
    }

    pub fn rebuild_space_visible(&mut self) {
        self.space_visible = crate::scanner::space::flatten_tree(&self.space_tree);
    }

    pub fn toggle_space_expand(&mut self) {
        if let Some(item) = self.space_visible.get(self.space_list_index) {
            if !item.is_dir {
                return;
            }
            let tree_path = item.tree_path.clone();
            if let Some(node) = crate::scanner::space::get_node_mut(&mut self.space_tree, &tree_path) {
                if node.expanded {
                    node.expanded = false;
                } else {
                    // Load children from cache — instant, no disk IO
                    if !node.children_loaded {
                        node.load_children_from_cache(&self.space_size_cache);
                    }
                    node.expanded = true;
                }
            }
            self.rebuild_space_visible();
            if self.space_list_index >= self.space_visible.len() {
                self.space_list_index = self.space_visible.len().saturating_sub(1);
            }
            self.space_list_state.select(Some(self.space_list_index));
        }
    }

    pub fn next_space_item(&mut self) {
        if !self.space_visible.is_empty() {
            self.space_list_index = (self.space_list_index + 1) % self.space_visible.len();
            self.space_list_state.select(Some(self.space_list_index));
        }
    }

    pub fn prev_space_item(&mut self) {
        if !self.space_visible.is_empty() {
            if self.space_list_index == 0 {
                self.space_list_index = self.space_visible.len() - 1;
            } else {
                self.space_list_index -= 1;
            }
            self.space_list_state.select(Some(self.space_list_index));
        }
    }

    // Apps
    pub fn scan_apps(&mut self) {
        if self.scanning { return; }
        self.scanning = true;
        self.scan_status = "Scanning installed applications...".to_string();
        self.scan_steps = vec![
            ScanStep { name: "Reading /Applications...".to_string(), done: false },
            ScanStep { name: "Calculating app sizes...".to_string(), done: false },
            ScanStep { name: "Finding related files...".to_string(), done: false },
        ];

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let _ = tx.send(ScanMessage::SmartScanProgress("Reading /Applications...".to_string()));
            let apps = crate::scanner::apps::scan_installed();
            let _ = tx.send(ScanMessage::SmartScanProgress("Calculating app sizes...".to_string()));
            let _ = tx.send(ScanMessage::SmartScanProgress("Finding related files...".to_string()));
            let _ = tx.send(ScanMessage::AppList(apps));
        });
    }

    pub fn scan_orphan_apps(&mut self) {
        if self.scanning { return; }
        self.scanning = true;
        self.scan_status = "Scanning for orphaned files...".to_string();
        self.scan_steps = vec![
            ScanStep { name: "Loading installed apps...".to_string(), done: false },
            ScanStep { name: "Scanning Application Support...".to_string(), done: false },
            ScanStep { name: "Matching orphaned files...".to_string(), done: false },
        ];

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let _ = tx.send(ScanMessage::SmartScanProgress("Loading installed apps...".to_string()));
            let orphans = crate::scanner::apps::scan_orphans();
            let _ = tx.send(ScanMessage::SmartScanProgress("Scanning Application Support...".to_string()));
            let _ = tx.send(ScanMessage::SmartScanProgress("Matching orphaned files...".to_string()));
            let _ = tx.send(ScanMessage::OrphanResults(orphans));
        });
    }

    pub fn filter_unused_apps(&mut self) {
        use std::time::{SystemTime, Duration};

        let six_months = Duration::from_secs(180 * 24 * 60 * 60);
        let now = SystemTime::now();

        self.unused_apps = self.app_list.iter().filter(|app| {
            match &app.last_used {
                None => true, // Never opened
                Some(date_str) => {
                    // Parse "2025-09-16 00:52:11 +0000"
                    parse_mdls_date(date_str)
                        .map(|last| {
                            now.duration_since(last)
                                .map(|age| age > six_months)
                                .unwrap_or(false)
                        })
                        .unwrap_or(true) // If can't parse, consider unused
                }
            }
        }).cloned().collect();

        self.unused_apps.sort_by(|a, b| b.size.cmp(&a.size));
    }

    pub fn cycle_app_view(&mut self) {
        self.app_view = match self.app_view {
            AppView::All => AppView::Unused,
            AppView::Unused => AppView::Leftovers,
            AppView::Leftovers => AppView::All,
        };
        self.app_list_index = 0;
        self.app_list_state.select(None);
        self.orphan_list_state.select(None);
        self.unused_list_state.select(None);

        // Auto-scan when switching to unused or leftovers
        match self.app_view {
            AppView::Unused => {
                // Filter unused from app_list (last used > 6 months ago)
                // This is populated after scan_apps completes
            }
            AppView::Leftovers => {
                if self.orphan_results.is_empty() {
                    self.scan_orphan_apps();
                }
            }
            _ => {}
        }
    }

    pub fn next_app(&mut self) {
        match self.app_view {
            AppView::Leftovers => {
                if !self.orphan_results.is_empty() {
                    self.app_list_index = (self.app_list_index + 1) % self.orphan_results.len();
                    self.orphan_list_state.select(Some(self.app_list_index));
                }
            }
            AppView::Unused => {
                if !self.unused_apps.is_empty() {
                    self.app_list_index = (self.app_list_index + 1) % self.unused_apps.len();
                    self.unused_list_state.select(Some(self.app_list_index));
                }
            }
            AppView::All => {
                if !self.app_list.is_empty() {
                    self.app_list_index = (self.app_list_index + 1) % self.app_list.len();
                    self.app_list_state.select(Some(self.app_list_index));
                }
            }
        }
    }

    pub fn prev_app(&mut self) {
        let len = match self.app_view {
            AppView::Leftovers => self.orphan_results.len(),
            AppView::Unused => self.unused_apps.len(),
            AppView::All => self.app_list.len(),
        };
        if len > 0 {
            if self.app_list_index == 0 {
                self.app_list_index = len - 1;
            } else {
                self.app_list_index -= 1;
            }
            match self.app_view {
                AppView::Leftovers => self.orphan_list_state.select(Some(self.app_list_index)),
                AppView::Unused => self.unused_list_state.select(Some(self.app_list_index)),
                AppView::All => self.app_list_state.select(Some(self.app_list_index)),
            }
        }
    }

    // Clean
    pub fn request_clean(&mut self) {
        if self.screen == Screen::SmartScan {
            if self.smart_scan_selected_size() > 0 {
                self.confirm_kind = ConfirmKind::CleanScan;
            }
        } else if self.selected_size() > 0 {
            self.confirm_kind = ConfirmKind::CleanScan;
        }
    }

    pub fn confirm_clean(&mut self) {
        if self.screen == Screen::SmartScan {
            self.clean_smart_scan();
            return;
        }
        // LargeOld or other screens — also use async cleaning
        let entries: Vec<ScanEntry> = self.scan_results
            .iter()
            .filter(|e| e.selected)
            .cloned()
            .collect();

        self.clean_total = entries.len();
        self.clean_progress = 0;
        self.clean_size_freed = 0;
        self.last_clean_results.clear();
        self.confirm_kind = ConfirmKind::Cleaning;

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            for entry in &entries {
                let size = entry.size;
                let result = crate::cleaner::move_to_trash(&entry.path);
                let msg = match result {
                    Ok(()) => format!("✓ {} ({})", entry.name, bytesize::ByteSize(size)),
                    Err(e) => format!("✗ {}: {}", entry.name, e),
                };
                let _ = tx.send(ScanMessage::CleanProgress { msg, size_freed: size });
            }
            let _ = tx.send(ScanMessage::CleanDone);
        });

        // Note: re-scan happens when user dismisses the CleanDone screen
        if self.screen == Screen::LargeOld {
            // Will re-scan after done
        }
    }

    pub fn request_uninstall(&mut self) {
        if self.app_view == AppView::All && !self.app_list.is_empty() {
            self.confirm_kind = ConfirmKind::UninstallApp;
        }
    }

    pub fn confirm_uninstall(&mut self) {
        if let Some(app_info) = self.app_list.get(self.app_list_index) {
            let _ = crate::cleaner::move_to_trash(&app_info.path);
            for related in &app_info.related_files {
                let _ = crate::cleaner::move_to_trash(&related.path);
            }
        }
        self.confirm_kind = ConfirmKind::None;
        self.scan_apps();
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_kind = ConfirmKind::None;
        self.kill_port_info = None;
        self.delete_space_info = None;
    }

    // Space Lens delete
    pub fn request_delete_space_item(&mut self) {
        if let Some(item) = self.space_visible.get(self.space_list_index) {
            let tree_path = item.tree_path.clone();
            if let Some(node) = crate::scanner::space::get_node_mut(&mut self.space_tree, &tree_path) {
                self.delete_space_info = Some((
                    node.name.clone(),
                    node.path.clone(),
                    node.size,
                    tree_path,
                ));
                self.confirm_kind = ConfirmKind::DeleteSpaceItem;
            }
        }
    }

    pub fn confirm_delete_space_item(&mut self) {
        if let Some((_, ref path, size, ref tree_path)) = self.delete_space_info {
            let _ = crate::cleaner::move_to_trash(path);

            // Subtract size from all ancestor directories in cache
            let path_clone = path.clone();
            self.space_size_cache.remove(&path_clone);
            let mut current = path_clone.as_path();
            while let Some(parent) = current.parent() {
                if let Some(parent_size) = self.space_size_cache.get_mut(&parent.to_path_buf()) {
                    *parent_size = parent_size.saturating_sub(size);
                }
                current = parent;
            }

            // Remove node from tree
            let tp = tree_path.clone();
            if tp.len() == 1 {
                // Top-level node
                let idx = tp[0];
                if idx < self.space_tree.len() {
                    self.space_tree.remove(idx);
                }
            } else if tp.len() > 1 {
                // Find parent, remove child
                let parent_path = &tp[..tp.len() - 1];
                let child_idx = *tp.last().unwrap();
                if let Some(parent_node) = crate::scanner::space::get_node_mut(&mut self.space_tree, parent_path) {
                    if child_idx < parent_node.children.len() {
                        parent_node.children.remove(child_idx);
                        parent_node.size = parent_node.size.saturating_sub(size);
                    }
                }
            }
        }

        self.delete_space_info = None;
        self.confirm_kind = ConfirmKind::None;

        // Rebuild visible items — instant, no disk IO
        self.rebuild_space_visible();
        if self.space_list_index >= self.space_visible.len() {
            self.space_list_index = self.space_visible.len().saturating_sub(1);
        }
        if !self.space_visible.is_empty() {
            self.space_list_state.select(Some(self.space_list_index));
        }
    }

    // Port navigation
    pub fn next_port(&mut self, port_count: usize) {
        if port_count > 0 {
            self.port_list_index = (self.port_list_index + 1) % port_count;
            self.port_list_state.select(Some(self.port_list_index));
        }
    }

    pub fn prev_port(&mut self, port_count: usize) {
        if port_count > 0 {
            if self.port_list_index == 0 {
                self.port_list_index = port_count - 1;
            } else {
                self.port_list_index -= 1;
            }
            self.port_list_state.select(Some(self.port_list_index));
        }
    }

    pub fn request_kill_port(&mut self, stats: &crate::system::SystemStats) {
        if let Some(port_info) = stats.listening_ports.get(self.port_list_index) {
            self.kill_port_info = Some(port_info.clone());
            self.confirm_kind = ConfirmKind::KillPort;
        }
    }

    pub fn confirm_kill_port(&mut self) {
        if let Some(ref info) = self.kill_port_info {
            let _ = std::process::Command::new("kill")
                .arg(info.pid.to_string())
                .output();
        }
        self.kill_port_info = None;
        self.confirm_kind = ConfirmKind::None;
    }
}
