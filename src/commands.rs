use crate::state::AppState;
use crate::state::PaneKind;
use crate::state::PromptKind;
use crate::state::ViewMode;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    Enter,
    UpDir,
    ToggleExpand,
    ViewTree,
    ViewList,
    ViewColumns,
    CycleView,
    PromptCreateFile,
    PromptCreateDir,
    PromptRename,
    PromptMove,
    PromptCopy,
    PromptDelete,
    PromptChar(char),
    PromptBackspace,
    PromptConfirm,
    PromptCancel,
    PromptSearch,
    AddBookmark,
    RemoveBookmark,
    BookmarkNext,
    BookmarkPrev,
    BookmarkOpen,
    ClearSearch,
    FocusNext,
    FocusPrev,
    ToggleSelect,
    SelectAll,
    ClearSelection,
    CopySelection,
    CutSelection,
    PasteSelection,
    DeleteSelection,
    ScrollUp(PaneKind),
    ScrollDown(PaneKind),
    ToggleHelp,
    OpenSelected,
}

pub fn apply_action(state: &mut AppState, action: Action) -> anyhow::Result<bool> {
    match action {
        Action::Quit => Ok(true),
        Action::MoveUp => {
            state.move_up();
            Ok(false)
        }
        Action::MoveDown => {
            state.move_down();
            Ok(false)
        }
        Action::Enter => {
            if state.focus == crate::state::FocusPane::Bookmarks {
                state.open_bookmark()?;
            } else {
                state.enter_selected()?;
            }
            Ok(false)
        }
        Action::UpDir => {
            state.go_parent()?;
            Ok(false)
        }
        Action::ToggleExpand => {
            state.toggle_expand()?;
            Ok(false)
        }
        Action::ViewTree => {
            state.set_view_mode(ViewMode::Tree)?;
            Ok(false)
        }
        Action::ViewList => {
            state.set_view_mode(ViewMode::List)?;
            Ok(false)
        }
        Action::ViewColumns => {
            state.set_view_mode(ViewMode::Columns)?;
            Ok(false)
        }
        Action::CycleView => {
            let next = match state.view_mode {
                ViewMode::Tree => ViewMode::List,
                ViewMode::List => ViewMode::Columns,
                ViewMode::Columns => ViewMode::Tree,
            };
            state.set_view_mode(next)?;
            Ok(false)
        }
        Action::PromptCreateFile => {
            state.start_prompt(PromptKind::CreateFile);
            Ok(false)
        }
        Action::PromptCreateDir => {
            state.start_prompt(PromptKind::CreateDir);
            Ok(false)
        }
        Action::PromptRename => {
            state.start_prompt(PromptKind::Rename);
            Ok(false)
        }
        Action::PromptMove => {
            state.start_prompt(PromptKind::Move);
            Ok(false)
        }
        Action::PromptCopy => {
            state.start_prompt(PromptKind::Copy);
            Ok(false)
        }
        Action::PromptDelete => {
            state.start_prompt(PromptKind::Delete);
            Ok(false)
        }
        Action::PromptChar(ch) => {
            state.push_prompt_char(ch);
            Ok(false)
        }
        Action::PromptBackspace => {
            state.pop_prompt_char();
            Ok(false)
        }
        Action::PromptConfirm => {
            state.confirm_prompt()?;
            Ok(false)
        }
        Action::PromptCancel => {
            state.cancel_prompt();
            Ok(false)
        }
        Action::PromptSearch => {
            state.start_prompt(PromptKind::Search);
            Ok(false)
        }
        Action::AddBookmark => {
            state.add_bookmark_current()?;
            Ok(false)
        }
        Action::RemoveBookmark => {
            state.remove_bookmark()?;
            Ok(false)
        }
        Action::BookmarkNext => {
            state.bookmark_next();
            Ok(false)
        }
        Action::BookmarkPrev => {
            state.bookmark_prev();
            Ok(false)
        }
        Action::BookmarkOpen => {
            state.open_bookmark()?;
            Ok(false)
        }
        Action::ClearSearch => {
            state.clear_search();
            Ok(false)
        }
        Action::FocusNext => {
            state.focus_next();
            Ok(false)
        }
        Action::FocusPrev => {
            state.focus_prev();
            Ok(false)
        }
        Action::ToggleSelect => {
            state.toggle_select();
            Ok(false)
        }
        Action::SelectAll => {
            state.select_all_visible();
            Ok(false)
        }
        Action::ClearSelection => {
            state.clear_selection();
            Ok(false)
        }
        Action::CopySelection => {
            state.copy_selection(false);
            Ok(false)
        }
        Action::CutSelection => {
            state.copy_selection(true);
            Ok(false)
        }
        Action::PasteSelection => {
            state.paste_selection()?;
            Ok(false)
        }
        Action::DeleteSelection => {
            state.delete_selection_prompt();
            Ok(false)
        }
        Action::ScrollUp(pane) => {
            match pane {
                PaneKind::Bookmarks => state.scroll_bookmarks(true),
                PaneKind::Middle => state.scroll_middle(true),
                PaneKind::Preview => state.scroll_preview(true),
            }
            Ok(false)
        }
        Action::ScrollDown(pane) => {
            match pane {
                PaneKind::Bookmarks => state.scroll_bookmarks(false),
                PaneKind::Middle => state.scroll_middle(false),
                PaneKind::Preview => state.scroll_preview(false),
            }
            Ok(false)
        }
        Action::ToggleHelp => {
            state.help_open = !state.help_open;
            Ok(false)
        }
        Action::OpenSelected => {
            state.open_selected()?;
            Ok(false)
        }
    }
}
