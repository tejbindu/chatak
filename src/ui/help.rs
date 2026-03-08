use crate::state::AppState;
use crate::ui::theme::Theme;
use ratatui::prelude::Alignment;
use ratatui::prelude::Frame;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

pub fn render(f: &mut Frame, _state: &AppState, theme: &Theme) {
    let area = f.size();
    let content = [
        "Help",
        "",
        "Navigation",
        "  Tab / Shift+Tab   Switch pane focus",
        "  j/k or arrows     Move selection",
        "  h / Backspace     Go parent",
        "  l / Enter         Open dir (or bookmark if focus is bookmarks)",
        "  o                 Open file with configured program",
        "  Space             Expand/collapse (tree view)",
        "",
        "Views",
        "  1 Tree  2 List  3 Columns  v Cycle",
        "",
        "Bookmarks",
        "  b Add bookmark from selection",
        "  B Remove selected bookmark",
        "  [ / ] Prev / Next bookmark",
        "  g Open selected bookmark",
        "",
        "Selection + Clipboard",
        "  s Toggle select",
        "  A Select all visible",
        "  u Clear selection",
        "  y Copy  x Cut  p Paste",
        "  D Delete selection",
        "",
        "File Ops",
        "  n New file  N New dir",
        "  r Rename  m Move  c Copy  d Delete (confirm with y)",
        "",
        "Search",
        "  / Search  X or Esc Clear search",
        "",
        "Help",
        "  ? Toggle help",
        "  q Quit",
    ]
    .join("\n");

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(ratatui::style::Style::default().fg(theme.purple));
    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .block(block)
        .style(ratatui::style::Style::default().fg(theme.foreground));

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}
