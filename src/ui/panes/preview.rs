use crate::state::AppState;
use crate::state::FocusPane;
use crate::ui::theme::Theme;
use ratatui::prelude::Frame;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = "Preview";
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let content = if state.preview_text.is_empty() {
        "No selection".to_string()
    } else {
        state.preview_text.clone()
    };
    let max_width = area.width.saturating_sub(2) as usize;
    let content = if max_width == 0 {
        String::new()
    } else {
        content
            .lines()
            .map(|line| line.replace('\t', "    "))
            .map(|line| line.chars().take(max_width).collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    };

    let style = if state.focus == FocusPane::Preview {
        Style::default()
            .bg(theme.highlight)
            .fg(theme.foreground)
            .bold()
    } else {
        Style::default().fg(theme.foreground)
    };
    let paragraph = Paragraph::new(content)
        .block(block)
        .style(style)
        .scroll((state.preview_scroll, 0))
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}
