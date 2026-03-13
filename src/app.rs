use std::time::{Duration, Instant};

use crate::scanner::ScanEntry;

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

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub sidebar_index: usize,
    pub focus: Focus,
    pub last_tick: Instant,
    pub tick_rate: Duration,
    pub scan_results: Vec<ScanEntry>,
    pub scan_list_index: usize,
    pub scanning: bool,
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
            scanning: false,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_sidebar(&mut self) {
        let screens = Screen::all();
        self.sidebar_index = (self.sidebar_index + 1) % screens.len();
        self.screen = screens[self.sidebar_index];
    }

    pub fn prev_sidebar(&mut self) {
        let screens = Screen::all();
        if self.sidebar_index == 0 {
            self.sidebar_index = screens.len() - 1;
        } else {
            self.sidebar_index -= 1;
        }
        self.screen = screens[self.sidebar_index];
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
        }
    }

    pub fn prev_list_item(&mut self) {
        if !self.scan_results.is_empty() {
            if self.scan_list_index == 0 {
                self.scan_list_index = self.scan_results.len() - 1;
            } else {
                self.scan_list_index -= 1;
            }
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
        self.scanning = true;
        self.scan_results.clear();
        self.scan_list_index = 0;

        // Run all scanners
        self.scan_results.extend(crate::scanner::cache::scan());
        self.scan_results.extend(crate::scanner::logs::scan());
        self.scan_results.extend(crate::scanner::brew::scan());

        // Sort by size descending
        self.scan_results.sort_by(|a, b| b.size.cmp(&a.size));
        self.scanning = false;
    }

    pub fn run_dev_scan(&mut self) {
        self.scanning = true;
        self.scan_results.clear();
        self.scan_list_index = 0;

        self.scan_results.extend(crate::scanner::xcode::scan());
        self.scan_results.extend(crate::scanner::docker::scan());
        self.scan_results.extend(crate::scanner::node::scan());
        self.scan_results.extend(crate::scanner::cargo::scan());

        self.scan_results.sort_by(|a, b| b.size.cmp(&a.size));
        self.scanning = false;
    }
}
