mod app;
mod cleaner;
mod system;
mod scanner;
mod ui;

use app::{App, ConfirmKind, Focus, ScanMessage};
use system::SystemStats;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{self, stdout};
use std::sync::mpsc;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut stats = SystemStats::new();

    while app.running {
        // Refresh stats on tick
        if app.last_tick.elapsed() >= app.tick_rate {
            stats.refresh();
            app.last_tick = std::time::Instant::now();
        }

        // Check for scan completion
        let mut received = None;
        if let Some(ref rx) = app.scan_receiver {
            match rx.try_recv() {
                Ok(msg) => received = Some(msg),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.scanning = false;
                    app.scan_receiver = None;
                    app.scan_status = "Scan failed".to_string();
                }
            }
        }
        if let Some(msg) = received {
            match msg {
                ScanMessage::ScanResults(results) => {
                    app.scan_results = results;
                    if !app.scan_results.is_empty() {
                        app.scan_list_index = 0;
                        app.scan_list_state.select(Some(0));
                    }
                }
                ScanMessage::AppList(apps) => {
                    app.app_list = apps;
                    app.app_list_index = 0;
                    if !app.app_list.is_empty() {
                        app.app_list_state.select(Some(0));
                    }
                }
                ScanMessage::OrphanResults(orphans) => {
                    app.orphan_results = orphans;
                    app.show_orphans = true;
                    app.app_list_index = 0;
                    if !app.orphan_results.is_empty() {
                        app.orphan_list_state.select(Some(0));
                    }
                }
            }
            app.scanning = false;
            app.scan_receiver = None;
            app.scan_status.clear();
        }

        // Animate spinner
        if app.scanning {
            app.tick_spinner();
        }

        terminal.draw(|frame| ui::layout::draw(frame, app, &stats))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.is_confirming() {
                        match key.code {
                            KeyCode::Enter => {
                                match app.confirm_kind {
                                    ConfirmKind::CleanScan => app.confirm_clean(),
                                    ConfirmKind::UninstallApp => app.confirm_uninstall(),
                                    ConfirmKind::KillPort => app.confirm_kill_port(),
                                    ConfirmKind::None => {}
                                }
                            }
                            KeyCode::Esc => app.cancel_confirm(),
                            _ => {}
                        }
                    } else {
                        // Ctrl+C always quits
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.quit();
                        } else {
                            match key.code {
                                KeyCode::Char('q') => app.quit(),
                                KeyCode::Tab => app.toggle_focus(),
                                KeyCode::Char('s') => {
                                    match app.screen {
                                        app::Screen::Scan => app.run_scan(),
                                        app::Screen::Dev => app.run_dev_scan(),
                                        app::Screen::Apps => app.scan_apps(),
                                        _ => {}
                                    }
                                }
                                KeyCode::Char('o') => {
                                    if app.screen == app::Screen::Apps {
                                        app.scan_orphan_apps();
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if app.screen == app::Screen::Apps && app.focus == Focus::Main {
                                        app.request_uninstall();
                                    }
                                }
                                KeyCode::Char('x') => {
                                    if app.screen == app::Screen::Home && app.focus == Focus::Main {
                                        app.request_kill_port(&stats);
                                    }
                                }
                                KeyCode::Char('c') => {
                                    if app.focus == Focus::Main {
                                        app.request_clean();
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    if app.focus == Focus::Main {
                                        if app.screen == app::Screen::Config {
                                            if app.config_index == 0 {
                                                app.safe_mode = !app.safe_mode;
                                            }
                                        } else {
                                            app.toggle_selected();
                                        }
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.focus == Focus::Sidebar {
                                        app.prev_sidebar();
                                    } else if app.screen == app::Screen::Home {
                                        app.prev_port(stats.listening_ports.len());
                                    } else if app.screen == app::Screen::Apps {
                                        app.prev_app();
                                    } else if app.screen == app::Screen::Config {
                                        if app.config_index > 0 {
                                            app.config_index -= 1;
                                        }
                                    } else {
                                        app.prev_list_item();
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.focus == Focus::Sidebar {
                                        app.next_sidebar();
                                    } else if app.screen == app::Screen::Home {
                                        app.next_port(stats.listening_ports.len());
                                    } else if app.screen == app::Screen::Apps {
                                        app.next_app();
                                    } else if app.screen == app::Screen::Config {
                                        if app.config_index < 1 {
                                            app.config_index += 1;
                                        }
                                    } else {
                                        app.next_list_item();
                                    }
                                }
                                KeyCode::Esc => {
                                    app.focus = Focus::Sidebar;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
