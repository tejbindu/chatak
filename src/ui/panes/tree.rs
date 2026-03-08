use crate::state::AppState;
use crate::state::FocusPane;
use crate::state::ViewMode;
use crate::ui::theme::Theme;
use ratatui::prelude::Constraint;
use ratatui::prelude::Direction;
use ratatui::prelude::Frame;
use ratatui::prelude::Layout;
use ratatui::prelude::Rect;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;

fn view_mode_label(mode: ViewMode) -> &'static str {
    match mode {
        ViewMode::Tree => "Tree",
        ViewMode::List => "List",
        ViewMode::Columns => "Columns",
    }
}

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    match state.view_mode {
        ViewMode::Tree => render_tree_view(f, area, state, theme),
        ViewMode::List => render_list_view(f, area, state, theme),
        ViewMode::Columns => render_columns_view(f, area, state, theme),
    }
}

fn render_tree_view(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = format!(
        " {} · {} ",
        state.current_dir.display(),
        view_mode_label(state.view_mode)
    );
    let title = truncate_title(&title, area.width);
    let items: Vec<ListItem> = state
        .visible_tree
        .iter()
        .filter_map(|index| state.tree_entries.get(*index))
        .map(|entry| {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let indent = "  ".repeat(entry.depth);
            let marker = if entry.is_dir {
                if entry.is_expanded {
                    "[-] "
                } else {
                    "[+] "
                }
            } else {
                "    "
            };
            let icon = icon_for_entry(entry.is_dir, &entry.path);
            let label = if entry.is_dir {
                format!("{indent}{marker}{icon} {name}/")
            } else {
                format!("{indent}{marker}{icon} {name}")
            };
            let mut style = if entry.is_dir {
                Style::default().fg(theme.cyan)
            } else {
                Style::default().fg(theme.foreground)
            };
            if state.selected_items.contains(&entry.path) {
                style = style.bg(theme.selection).fg(theme.foreground);
            }
            ListItem::new(label).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    if !state.visible_tree.is_empty() {
        list_state.select(Some(state.selection.min(state.visible_tree.len() - 1)));
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let highlight = if state.focus == FocusPane::Middle {
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

fn render_list_view(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = format!(
        " {} · {} ",
        state.current_dir.display(),
        view_mode_label(state.view_mode)
    );
    let title = truncate_title(&title, area.width);
    let items: Vec<ListItem> = state
        .visible_entries
        .iter()
        .filter_map(|index| state.entries.get(*index))
        .map(|entry| {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let icon = icon_for_entry(entry.is_dir, &entry.path);
            let label = if entry.is_dir {
                format!("{icon} {name}/")
            } else {
                format!("{icon} {name}")
            };
            let mut style = if entry.is_dir {
                Style::default().fg(theme.cyan)
            } else {
                Style::default().fg(theme.foreground)
            };
            if state.selected_items.contains(&entry.path) {
                style = style.bg(theme.selection).fg(theme.foreground);
            }
            ListItem::new(label).style(style)
        })
        .collect();

    let mut list_state = ListState::default();
    if !state.visible_entries.is_empty() {
        list_state.select(Some(state.selection.min(state.visible_entries.len() - 1)));
    }

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let highlight = if state.focus == FocusPane::Middle {
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

fn render_columns_view(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = format!(
        " {} · {} ",
        state.current_dir.display(),
        view_mode_label(state.view_mode)
    );
    let title = truncate_title(&title, area.width);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let highlight = if state.focus == FocusPane::Middle {
        Style::default()
            .bg(theme.highlight)
            .fg(theme.foreground)
            .bold()
    } else {
        Style::default().fg(theme.foreground)
    };

    let left_items: Vec<ListItem> = state
        .visible_entries
        .iter()
        .filter_map(|index| state.entries.get(*index))
        .map(|entry| {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let icon = icon_for_entry(entry.is_dir, &entry.path);
            let label = if entry.is_dir {
                format!("{icon} {name}/")
            } else {
                format!("{icon} {name}")
            };
            let mut style = if entry.is_dir {
                Style::default().fg(theme.cyan)
            } else {
                Style::default().fg(theme.foreground)
            };
            if state.selected_items.contains(&entry.path) {
                style = style.bg(theme.selection).fg(theme.foreground);
            }
            ListItem::new(label).style(style)
        })
        .collect();

    let mut left_state = ListState::default();
    if !state.visible_entries.is_empty() {
        left_state.select(Some(state.selection.min(state.visible_entries.len() - 1)));
    }

    let left_list = List::new(left_items)
        .block(block.clone())
        .style(Style::default().fg(theme.foreground))
        .highlight_style(highlight);
    f.render_stateful_widget(left_list, columns[0], &mut left_state);

    let right_items: Vec<ListItem> = state
        .columns_right
        .iter()
        .map(|entry| {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let icon = icon_for_entry(entry.is_dir, &entry.path);
            let label = if entry.is_dir {
                format!("{icon} {name}/")
            } else {
                format!("{icon} {name}")
            };
            let mut style = if entry.is_dir {
                Style::default().fg(theme.cyan)
            } else {
                Style::default().fg(theme.foreground)
            };
            if state.selected_items.contains(&entry.path) {
                style = style.bg(theme.selection).fg(theme.foreground);
            }
            ListItem::new(label).style(style)
        })
        .collect();

    let right_block = Block::default()
        .title("Side")
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.purple));
    let right_list = List::new(right_items)
        .block(right_block)
        .style(Style::default().fg(theme.foreground));
    f.render_widget(right_list, columns[1]);
}

fn truncate_title(title: &str, width: u16) -> String {
    let max = width.saturating_sub(2) as usize;
    if max == 0 || title.chars().count() <= max {
        return title.to_string();
    }
    if max <= 3 {
        return ".".repeat(max);
    }
    let mut out = String::from("...");
    let tail_len = max - 3;
    let tail: String = title
        .chars()
        .rev()
        .take(tail_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    out.push_str(&tail);
    out
}

fn icon_for_entry(is_dir: bool, path: &std::path::Path) -> &'static str {
    if is_dir {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            match name {
                ".git" => "󰊢",
                "src" => "󰈙",
                "tests" | "test" => "󰙨",
                "target" | "dist" | "build" => "󰇮",
                ".config" | "config" => "󱁿",
                "docs" | "doc" => "󰈙",
                "scripts" | "script" => "󰆍",
                "assets" | "static" => "󰡨",
                _ => "",
            }
        } else {
            ""
        }
    } else {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        match name {
            "Cargo.toml" => "󰏗",
            "Cargo.lock" => "󰏗",
            "package.json" => "󰎙",
            "package-lock.json" => "󰎙",
            "pnpm-lock.yaml" => "󰎙",
            "yarn.lock" => "󰎙",
            "Makefile" => "󰆧",
            ".gitignore" => "󰊢",
            "README.md" => "󰈙",
            "LICENSE" | "LICENSE.md" => "󰌆",
            _ => {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                match ext.to_ascii_lowercase().as_str() {
                    "rs" => "",
                    "toml" => "󰏗",
                    "yaml" | "yml" => "󰈙",
                    "json" => "󰘦",
                    "md" => "󰈙",
                    "txt" => "󰈙",
                    "lock" => "󰌾",
                    "env" => "󰈡",
                    "ini" => "󰈡",
                    "cfg" | "conf" => "󰒓",
                    "sh" | "bash" | "zsh" | "fish" => "󰆍",
                    "ps1" => "󰨊",
                    "bat" | "cmd" => "󰆍",
                    "c" => "",
                    "h" => "",
                    "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => "",
                    "go" => "",
                    "py" => "",
                    "rb" => "",
                    "php" => "",
                    "js" => "",
                    "ts" => "",
                    "jsx" => "",
                    "tsx" => "",
                    "java" => "",
                    "kt" | "kts" => "󱈙",
                    "swift" => "",
                    "cs" => "󰌛",
                    "lua" => "",
                    "sql" => "",
                    "html" | "htm" => "",
                    "css" => "",
                    "scss" | "sass" => "",
                    "less" => "",
                    "xml" => "󰗀",
                    "svg" => "󰜡",
                    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" => "󰈟",
                    "mp3" | "wav" | "flac" | "ogg" => "󰎆",
                    "mp4" | "mkv" | "mov" | "avi" | "webm" => "󰈫",
                    "zip" | "tar" | "gz" | "xz" | "7z" | "rar" => "󰗄",
                    "pdf" => "󰈦",
                    "ttf" | "otf" | "woff" | "woff2" => "󰛖",
                    "exe" | "bin" | "app" => "󰊴",
                    "dll" | "so" | "dylib" => "󰌋",
                    "iso" => "󰋊",
                    _ => "󰈙",
                }
            }
        }
    }
}
