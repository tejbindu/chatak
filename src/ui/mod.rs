pub mod panes;
pub mod status;
pub mod theme;
pub mod help;

use crate::state::AppState;
use ratatui::prelude::Constraint;
use ratatui::prelude::Direction;
use ratatui::prelude::Frame;
use ratatui::prelude::Layout;
use ratatui::widgets::Block;
use ratatui::widgets::Clear;

pub fn draw(f: &mut Frame, state: &mut AppState) {
    let theme = theme::Theme::dracula();
    let background = Block::default().style(ratatui::style::Style::default().bg(theme.background));
    f.render_widget(background, f.size());

    if state.help_open {
        help::render(f, state, &theme);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(rows[0]);

    state.pane_areas = Some(crate::state::PaneAreas {
        bookmarks: chunks[0],
        middle: chunks[1],
        preview: chunks[2],
        status: rows[1],
    });

    f.render_widget(Clear, chunks[0]);
    f.render_widget(Clear, chunks[1]);
    f.render_widget(Clear, chunks[2]);
    f.render_widget(Clear, rows[1]);
    panes::bookmarks::render(f, chunks[0], state, &theme);
    panes::tree::render(f, chunks[1], state, &theme);
    panes::preview::render(f, chunks[2], state, &theme);
    status::render(f, rows[1], state, &theme);
}
