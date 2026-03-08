use crate::state::AppState;
use crate::state::PromptKind;
use crate::ui::theme::Theme;
use ratatui::prelude::Frame;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let content = if let Some(prompt) = &state.prompt {
        let label = match prompt.kind {
            PromptKind::CreateFile => "New file: ",
            PromptKind::CreateDir => "New dir: ",
            PromptKind::Rename => "Rename to: ",
            PromptKind::Move => "Move to: ",
            PromptKind::Copy => "Copy to: ",
            PromptKind::Delete => "Delete? type y: ",
            PromptKind::DeleteSelection => "Delete selection? type y: ",
            PromptKind::Search => "Search: ",
        };
        format!("{label}{}", prompt.input)
    } else if state.status.is_empty() {
        "tab pane | j/k move | s select | y/x/p copy/cut/paste | D delete sel | / search"
            .to_string()
    } else {
        state.status.clone()
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(theme.comment).bg(theme.background));
    f.render_widget(paragraph, area);
}
