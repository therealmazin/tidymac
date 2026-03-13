use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::widgets::ListState;

use crate::scanner::ScanEntry;
use crate::scanner::apps::AppInfo;

const SPINNER_CHARS: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Scan,
    Dev,
    Apps,
    Config,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Home,
            Screen::Scan,
            Screen::Dev,
            Screen::Apps,
            Screen::Config,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Screen::Home => "󰣇 Home",
            Screen::Scan => "󰃢 Scan",
            Screen::Dev => "󰅐 Dev",
            Screen::Apps => "󰀲 Apps",
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
}

pub enum ScanMessage {
    ScanResults(Vec<ScanEntry>),
    AppList(Vec<AppInfo>),
    OrphanResults(Vec<ScanEntry>),
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub sidebar_index: usize,
    pub focus: Focus,
    pub last_tick: Instant,
    pub tick_rate: Duration,
    // Scan/Dev results
    pub scan_results: Vec<ScanEntry>,
    pub scan_list_index: usize,
    pub scan_list_state: ListState,
    pub scanning: bool,
    // Apps
    pub app_list: Vec<AppInfo>,
    pub app_list_index: usize,
    pub app_list_state: ListState,
    pub show_orphans: bool,
    pub orphan_results: Vec<ScanEntry>,
    pub orphan_list_state: ListState,
    // Confirm dialog
    pub confirm_kind: ConfirmKind,
    pub last_clean_results: Vec<String>,
    // Settings
    pub safe_mode: bool,
    pub config_index: usize,
    // Ports (Home screen)
    pub port_list_index: usize,
    pub port_list_state: ListState,
    pub kill_port_info: Option<crate::system::PortInfo>, // snapshot for confirm dialog
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
            app_list: Vec::new(),
            app_list_index: 0,
            app_list_state: ListState::default(),
            show_orphans: false,
            orphan_results: Vec::new(),
            orphan_list_state: ListState::default(),
            confirm_kind: ConfirmKind::None,
            last_clean_results: Vec::new(),
            safe_mode: true,
            config_index: 0,
            port_list_index: 0,
            port_list_state: ListState::default(),
            kill_port_info: None,
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

    pub fn run_scan(&mut self) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_results.clear();
        self.scan_list_index = 0;
        self.scan_list_state.select(None);
        self.scan_status = "Scanning caches, logs, brew...".to_string();

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let mut results = Vec::new();
            results.extend(crate::scanner::cache::scan());
            results.extend(crate::scanner::logs::scan());
            results.extend(crate::scanner::brew::scan());
            results.sort_by(|a, b| b.size.cmp(&a.size));
            let _ = tx.send(ScanMessage::ScanResults(results));
        });
    }

    pub fn run_dev_scan(&mut self) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_results.clear();
        self.scan_list_index = 0;
        self.scan_list_state.select(None);
        self.scan_status = "Scanning Xcode, Docker, node_modules, Cargo...".to_string();

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let mut results = Vec::new();
            results.extend(crate::scanner::xcode::scan());
            results.extend(crate::scanner::docker::scan());
            results.extend(crate::scanner::node::scan());
            results.extend(crate::scanner::cargo::scan());
            results.sort_by(|a, b| b.size.cmp(&a.size));
            let _ = tx.send(ScanMessage::ScanResults(results));
        });
    }

    pub fn scan_apps(&mut self) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_status = "Scanning installed applications...".to_string();

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let apps = crate::scanner::apps::scan_installed();
            let _ = tx.send(ScanMessage::AppList(apps));
        });
    }

    pub fn scan_orphan_apps(&mut self) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        self.scan_status = "Scanning for orphaned files...".to_string();

        let (tx, rx) = mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let orphans = crate::scanner::apps::scan_orphans();
            let _ = tx.send(ScanMessage::OrphanResults(orphans));
        });
    }

    pub fn next_app(&mut self) {
        if self.show_orphans {
            if !self.orphan_results.is_empty() {
                self.app_list_index = (self.app_list_index + 1) % self.orphan_results.len();
                self.orphan_list_state.select(Some(self.app_list_index));
            }
        } else if !self.app_list.is_empty() {
            self.app_list_index = (self.app_list_index + 1) % self.app_list.len();
            self.app_list_state.select(Some(self.app_list_index));
        }
    }

    pub fn prev_app(&mut self) {
        if self.show_orphans {
            if !self.orphan_results.is_empty() {
                if self.app_list_index == 0 {
                    self.app_list_index = self.orphan_results.len() - 1;
                } else {
                    self.app_list_index -= 1;
                }
                self.orphan_list_state.select(Some(self.app_list_index));
            }
        } else if !self.app_list.is_empty() {
            if self.app_list_index == 0 {
                self.app_list_index = self.app_list.len() - 1;
            } else {
                self.app_list_index -= 1;
            }
            self.app_list_state.select(Some(self.app_list_index));
        }
    }

    // Scan clean
    pub fn request_clean(&mut self) {
        if self.selected_size() > 0 {
            self.confirm_kind = ConfirmKind::CleanScan;
        }
    }

    pub fn confirm_clean(&mut self) {
        let results = crate::cleaner::clean_selected(&self.scan_results);
        self.last_clean_results = results
            .into_iter()
            .map(|r| match r {
                Ok(msg) => msg,
                Err(e) => format!("Error: {}", e),
            })
            .collect();
        self.confirm_kind = ConfirmKind::None;
        // Re-scan to update
        match self.screen {
            Screen::Scan => self.run_scan(),
            Screen::Dev => self.run_dev_scan(),
            _ => {}
        }
    }

    // App uninstall
    pub fn request_uninstall(&mut self) {
        if !self.show_orphans && !self.app_list.is_empty() {
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
    }

    // Port navigation (Home screen)
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
