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
        if app.last_tick.elapsed() >= app.tick_rate {
            stats.refresh();
            app.last_tick = std::time::Instant::now();
        }

        // Check for scan messages (may receive multiple per tick for progress updates)
        let mut final_msg = None;
        if let Some(ref rx) = app.scan_receiver {
            loop {
                match rx.try_recv() {
                    Ok(ScanMessage::SmartScanProgress(step_name)) => {
                        for step in &mut app.scan_steps {
                            if step.name == step_name {
                                step.done = true;
                                break;
                            }
                        }
                    }
                    Ok(ScanMessage::CleanProgress { msg, size_freed }) => {
                        app.clean_progress += 1;
                        app.clean_size_freed += size_freed;
                        app.last_clean_results.push(msg);
                    }
                    Ok(ScanMessage::CleanDone) => {
                        final_msg = Some(ScanMessage::CleanDone);
                        break;
                    }
                    Ok(msg) => {
                        final_msg = Some(msg);
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        app.scanning = false;
                        app.scan_receiver = None;
                        app.scan_status = "Scan failed".to_string();
                        break;
                    }
                }
            }
        }
        if let Some(msg) = final_msg {
            match msg {
                ScanMessage::SmartScanProgress(_) => {}
                ScanMessage::CleanProgress { .. } => {}
                ScanMessage::CleanDone => {
                    app.confirm_kind = ConfirmKind::CleanDone;
                }
                ScanMessage::ScanResults(results) => {
                    app.scan_results = results;
                    if !app.scan_results.is_empty() {
                        app.scan_list_index = 0;
                        app.scan_list_state.select(Some(0));
                    }
                }
                ScanMessage::SmartScanResults(categories) => {
                    app.smart_scan_categories = categories;
                    app.smart_scan_index = 0;
                    if !app.smart_scan_categories.is_empty() {
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
                    app.app_list_index = 0;
                    if !app.orphan_results.is_empty() {
                        app.orphan_list_state.select(Some(0));
                    }
                }
                ScanMessage::SpaceTreeWithCache(tree, cache) => {
                    app.space_tree = tree;
                    app.space_size_cache = cache;
                    app.rebuild_space_visible();
                    app.space_list_index = 0;
                    if !app.space_visible.is_empty() {
                        app.space_list_state.select(Some(0));
                    }
                }
            }
            app.scanning = false;
            app.scan_receiver = None;
            app.scan_status.clear();
        }

        if app.scanning || app.space_expanding {
            app.tick_spinner();
        }

        terminal.draw(|frame| ui::layout::draw(frame, app, &stats))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.is_confirming() {
                        match app.confirm_kind {
                            ConfirmKind::Cleaning => {} // can't dismiss while cleaning
                            ConfirmKind::CleanDone => {
                                if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                                    app.confirm_kind = ConfirmKind::None;
                                    // Re-scan current screen
                                    match app.screen {
                                        app::Screen::SmartScan => app.run_smart_scan(),
                                        app::Screen::LargeOld => app.run_large_old_scan(),
                                        _ => {}
                                    }
                                }
                            }
                            _ => {
                                match key.code {
                                    KeyCode::Enter => {
                                        match app.confirm_kind {
                                            ConfirmKind::CleanScan => app.confirm_clean(),
                                            ConfirmKind::UninstallApp => app.confirm_uninstall(),
                                            ConfirmKind::KillPort => app.confirm_kill_port(),
                                            _ => {}
                                        }
                                    }
                                    KeyCode::Esc => app.cancel_confirm(),
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        // Ctrl+C always quits
                        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.quit();
                        } else {
                            match key.code {
                                KeyCode::Char('q') => app.quit(),
                                KeyCode::Tab => {
                                    if app.screen == app::Screen::Apps && app.focus == Focus::Main {
                                        app.cycle_app_view();
                                    } else {
                                        app.toggle_focus();
                                    }
                                }
                                KeyCode::Char('s') => {
                                    match app.screen {
                                        app::Screen::SmartScan => app.run_smart_scan(),
                                        app::Screen::Apps => app.scan_apps(),
                                        app::Screen::SpaceLens => app.run_space_scan(),
                                        app::Screen::LargeOld => app.run_large_old_scan(),
                                        _ => {}
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
                                        match app.screen {
                                            app::Screen::Config => {
                                                if app.config_index == 0 {
                                                    app.safe_mode = !app.safe_mode;
                                                }
                                            }
                                            app::Screen::SmartScan => app.toggle_smart_scan_item(),
                                            _ => app.toggle_selected(),
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    if app.focus == Focus::Main {
                                        match app.screen {
                                            app::Screen::SmartScan => app.toggle_smart_scan_expand(),
                                            app::Screen::SpaceLens => app.toggle_space_expand(),
                                            _ => {}
                                        }
                                    }
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.focus == Focus::Sidebar {
                                        app.prev_sidebar();
                                    } else {
                                        match app.screen {
                                            app::Screen::Home => app.prev_port(stats.listening_ports.len()),
                                            app::Screen::SmartScan => app.prev_smart_scan_item(),
                                            app::Screen::Apps => app.prev_app(),
                                            app::Screen::SpaceLens => app.prev_space_item(),
                                            app::Screen::Config => {
                                                if app.config_index > 0 { app.config_index -= 1; }
                                            }
                                            _ => app.prev_list_item(),
                                        }
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.focus == Focus::Sidebar {
                                        app.next_sidebar();
                                    } else {
                                        match app.screen {
                                            app::Screen::Home => app.next_port(stats.listening_ports.len()),
                                            app::Screen::SmartScan => app.next_smart_scan_item(),
                                            app::Screen::Apps => app.next_app(),
                                            app::Screen::SpaceLens => app.next_space_item(),
                                            app::Screen::Config => {
                                                if app.config_index < 1 { app.config_index += 1; }
                                            }
                                            _ => app.next_list_item(),
                                        }
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
