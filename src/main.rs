mod app;

use app::{App, Focus};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{self, stdout};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Run app
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while app.running {
        terminal.draw(|frame| draw(frame, app))?;

        if event::poll(app.tick_rate)? {
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

fn draw(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(frame.area());

    // Header
    let header = Paragraph::new("  tidymac")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(header, outer[0]);

    // Body: sidebar + main
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(outer[1]);

    // Sidebar
    let items: Vec<ListItem> = app::Screen::all()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.sidebar_index {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(format!(" {}", s.label())).style(style)
        })
        .collect();

    let sidebar_style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).border_style(sidebar_style));
    frame.render_widget(sidebar, body[0]);

    // Main panel placeholder
    let main_content = Paragraph::new(format!("  {} screen — coming soon", app.screen.label()))
        .style(Style::default().fg(Color::White));
    frame.render_widget(main_content, body[1]);
}
