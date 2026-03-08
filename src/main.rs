mod commands;
mod fs;
mod input;
mod state;
mod ui;

use anyhow::Result;
use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let _cleanup = TerminalCleanup;

    let mut app = state::AppState::new()?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        if event::poll(Duration::from_millis(200))? {
            let ev = event::read()?;
            if let event::Event::Resize(_, _) = ev {
                terminal.clear()?;
                continue;
            }
            let pane_at = |col: u16, row: u16| {
                app.pane_areas
                    .as_ref()
                    .and_then(|areas| areas.pane_at(col, row))
            };
            if let Some(action) =
                input::handle_event(ev, app.prompt.is_some(), app.help_open, pane_at)
            {
                match commands::apply_action(&mut app, action) {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(err) => {
                        app.status = err.to_string();
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
    }
}
