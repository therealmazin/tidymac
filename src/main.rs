mod app;
mod system;
mod ui;

use app::{App, Focus};
use system::SystemStats;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::{self, stdout};

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

        terminal.draw(|frame| ui::layout::draw(frame, app, &stats))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Tab => app.toggle_focus(),
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.focus == Focus::Sidebar {
                                app.prev_sidebar();
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.focus == Focus::Sidebar {
                                app.next_sidebar();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
