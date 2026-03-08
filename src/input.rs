use crate::commands::Action;
use crate::state::PaneKind;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use crossterm::event::MouseEventKind;

pub fn handle_event(
    event: Event,
    in_prompt: bool,
    help_open: bool,
    pane_at: impl Fn(u16, u16) -> Option<PaneKind>,
) -> Option<Action> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if help_open {
                return match key.code {
                    KeyCode::Char('?') | KeyCode::Esc => Some(Action::ToggleHelp),
                    _ => None,
                };
            }
            if in_prompt {
                match key.code {
                    KeyCode::Enter => Some(Action::PromptConfirm),
                    KeyCode::Esc => Some(Action::PromptCancel),
                    KeyCode::Backspace => Some(Action::PromptBackspace),
                    KeyCode::Char(ch) => Some(Action::PromptChar(ch)),
                    _ => None,
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => Some(Action::Quit),
                    KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
                    KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(Action::Enter),
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => Some(Action::UpDir),
                    KeyCode::Char(' ') => Some(Action::ToggleExpand),
                    KeyCode::Char('1') => Some(Action::ViewTree),
                    KeyCode::Char('2') => Some(Action::ViewList),
                    KeyCode::Char('3') => Some(Action::ViewColumns),
                    KeyCode::Char('v') => Some(Action::CycleView),
                    KeyCode::Char('n') => Some(Action::PromptCreateFile),
                    KeyCode::Char('N') => Some(Action::PromptCreateDir),
                    KeyCode::Char('r') => Some(Action::PromptRename),
                    KeyCode::Char('m') => Some(Action::PromptMove),
                    KeyCode::Char('c') => Some(Action::PromptCopy),
                    KeyCode::Char('d') => Some(Action::PromptDelete),
                    KeyCode::Char('b') => Some(Action::AddBookmark),
                    KeyCode::Char('B') => Some(Action::RemoveBookmark),
                    KeyCode::Char(']') => Some(Action::BookmarkNext),
                    KeyCode::Char('[') => Some(Action::BookmarkPrev),
                    KeyCode::Char('g') => Some(Action::BookmarkOpen),
                    KeyCode::Char('/') => Some(Action::PromptSearch),
                    KeyCode::Char('X') => Some(Action::ClearSearch),
                    KeyCode::Esc => Some(Action::ClearSearch),
                    KeyCode::Char('?') => Some(Action::ToggleHelp),
                    KeyCode::Tab => Some(Action::FocusNext),
                    KeyCode::BackTab => Some(Action::FocusPrev),
                    KeyCode::Char('s') => Some(Action::ToggleSelect),
                    KeyCode::Char('A') => Some(Action::SelectAll),
                    KeyCode::Char('u') => Some(Action::ClearSelection),
                    KeyCode::Char('y') => Some(Action::CopySelection),
                    KeyCode::Char('x') => Some(Action::CutSelection),
                    KeyCode::Char('p') => Some(Action::PasteSelection),
                    KeyCode::Char('D') => Some(Action::DeleteSelection),
                    KeyCode::Char('o') => Some(Action::OpenSelected),
                    _ => None,
                }
            }
        }
        Event::Mouse(mouse) => {
            let pane = pane_at(mouse.column, mouse.row)?;
            match mouse.kind {
                MouseEventKind::ScrollUp => Some(Action::ScrollUp(pane)),
                MouseEventKind::ScrollDown => Some(Action::ScrollDown(pane)),
                _ => None,
            }
        }
        _ => None,
    }
}
