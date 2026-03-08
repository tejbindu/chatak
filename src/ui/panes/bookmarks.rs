use crate::state::AppState;
use crate::state::FocusPane;
use crate::ui::theme::Theme;
use ratatui::prelude::Frame;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = "Bookmarks";
    let items: Vec<ListItem> = state
        .bookmarks
        .iter()
        .map(|path| {
            let name = path.to_string_lossy();
            ListItem::new(name.to_string())
        })
        .collect();

    let mut list_state = ListState::default();
    if !state.bookmarks.is_empty() {
        list_state.select(Some(state.bookmark_index.min(state.bookmarks.len() - 1)));
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let highlight = if state.focus == FocusPane::Bookmarks {
        Style::default()
            .bg(theme.highlight)
            .fg(theme.foreground)
            .bold()
    } else {
        Style::default().fg(theme.foreground)
    };
    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground))
        .highlight_style(highlight);
    f.render_stateful_widget(list, area, &mut list_state);
}
