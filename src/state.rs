use crate::fs::list_dir;
use crate::fs::Config;
use crate::fs::Opener;
use crate::fs::FileEntry;
use ratatui::prelude::Rect;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tree,
    List,
    Columns,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Bookmarks,
    Middle,
    Preview,
}

#[derive(Debug)]
pub struct AppState {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub tree_entries: Vec<TreeEntry>,
    pub visible_entries: Vec<usize>,
    pub visible_tree: Vec<usize>,
    pub selection: usize,
    pub selected_items: HashSet<PathBuf>,
    pub bookmarks: Vec<PathBuf>,
    pub bookmark_index: usize,
    pub view_mode: ViewMode,
    pub focus: FocusPane,
    pub pane_areas: Option<PaneAreas>,
    pub expanded: HashSet<PathBuf>,
    pub selection_memory: HashMap<PathBuf, PathBuf>,
    pub columns_right: Vec<FileEntry>,
    pub columns_right_path: Option<PathBuf>,
    pub clipboard: Clipboard,
    pub config: Config,
    pub prompt: Option<Prompt>,
    pub status: String,
    pub preview_path: Option<PathBuf>,
    pub preview_text: String,
    pub preview_scroll: u16,
    pub search_query: String,
    pub help_open: bool,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let config = crate::fs::load_config();
        let current_dir = std::env::current_dir()?;
        let entries = list_dir(&current_dir)?;
        let mut expanded = HashSet::new();
        expanded.insert(current_dir.clone());
        let tree_entries = build_tree_entries(&current_dir, &expanded)?;
        let bookmarks: Vec<PathBuf> =
            config.bookmarks.iter().map(PathBuf::from).collect();
        let mut state = Self {
            current_dir,
            entries,
            tree_entries,
            visible_entries: Vec::new(),
            visible_tree: Vec::new(),
            selection: 0,
            selected_items: HashSet::new(),
            bookmarks,
            bookmark_index: 0,
            view_mode: ViewMode::Tree,
            focus: FocusPane::Middle,
            pane_areas: None,
            expanded,
            selection_memory: HashMap::new(),
            columns_right: Vec::new(),
            columns_right_path: None,
            clipboard: Clipboard::default(),
            config,
            prompt: None,
            status: String::new(),
            preview_path: None,
            preview_text: String::new(),
            preview_scroll: 0,
            search_query: String::new(),
            help_open: false,
        };
        state.rebuild_visible();
        state.update_preview();
        Ok(state)
    }

    pub fn refresh_entries(&mut self) -> anyhow::Result<()> {
        self.entries = list_dir(&self.current_dir)?;
        self.rebuild_tree()?;
        self.rebuild_visible();
        self.coerce_selection();
        self.update_columns_preview();
        self.update_preview();
        Ok(())
    }

    pub fn selected_path(&self) -> Option<(&PathBuf, bool)> {
        match self.view_mode {
            ViewMode::Tree => self
                .visible_tree
                .get(self.selection)
                .and_then(|index| self.tree_entries.get(*index))
                .map(|entry| (&entry.path, entry.is_dir)),
            ViewMode::List | ViewMode::Columns => self
                .visible_entries
                .get(self.selection)
                .and_then(|index| self.entries.get(*index))
                .map(|entry| (&entry.path, entry.is_dir)),
        }
    }

    pub fn select_next(&mut self) {
        let len = self.current_view_len();
        if len == 0 {
            self.selection = 0;
            return;
        }
        self.selection = (self.selection + 1).min(len - 1);
        self.update_columns_preview();
        self.update_preview();
    }

    pub fn select_prev(&mut self) {
        let len = self.current_view_len();
        if len == 0 {
            self.selection = 0;
            return;
        }
        if self.selection == 0 {
            return;
        }
        self.selection -= 1;
        self.update_columns_preview();
        self.update_preview();
    }

    pub fn move_up(&mut self) {
        match self.focus {
            FocusPane::Bookmarks => self.bookmark_prev(),
            FocusPane::Middle => self.select_prev(),
            FocusPane::Preview => self.scroll_preview(true),
        }
    }

    pub fn move_down(&mut self) {
        match self.focus {
            FocusPane::Bookmarks => self.bookmark_next(),
            FocusPane::Middle => self.select_next(),
            FocusPane::Preview => self.scroll_preview(false),
        }
    }

    pub fn set_view_mode(&mut self, mode: ViewMode) -> anyhow::Result<()> {
        self.view_mode = mode;
        self.selection = 0;
        if self.view_mode == ViewMode::Tree {
            self.rebuild_tree()?;
        }
        self.rebuild_visible();
        self.update_columns_preview();
        self.update_preview();
        Ok(())
    }

    pub fn enter_selected(&mut self) -> anyhow::Result<()> {
        let target = self.selected_path().map(|(path, is_dir)| (path.clone(), is_dir));
        if let Some((path, is_dir)) = target {
            if is_dir {
                self.remember_selection();
                self.set_current_dir(path)?;
            }
        }
        Ok(())
    }

    pub fn go_parent(&mut self) -> anyhow::Result<()> {
        let parent = self.current_dir.parent().map(|path| path.to_path_buf());
        if let Some(parent) = parent {
            self.remember_selection();
            self.set_current_dir(parent)?;
        }
        Ok(())
    }

    pub fn toggle_expand(&mut self) -> anyhow::Result<()> {
        if self.view_mode != ViewMode::Tree {
            return Ok(());
        }
        let Some(entry) = self
            .visible_tree
            .get(self.selection)
            .and_then(|index| self.tree_entries.get(*index))
        else {
            return Ok(());
        };
        if !entry.is_dir || entry.depth == 0 {
            return Ok(());
        }
        if self.expanded.contains(&entry.path) {
            self.expanded.remove(&entry.path);
        } else {
            self.expanded.insert(entry.path.clone());
        }
        self.rebuild_tree()?;
        self.rebuild_visible();
        self.coerce_selection();
        self.update_preview();
        Ok(())
    }

    fn rebuild_tree(&mut self) -> anyhow::Result<()> {
        self.tree_entries = build_tree_entries(&self.current_dir, &self.expanded)?;
        Ok(())
    }

    fn coerce_selection(&mut self) {
        let len = self.current_view_len();
        if len == 0 {
            self.selection = 0;
        } else {
            self.selection = self.selection.min(len - 1);
        }
    }

    fn current_view_len(&self) -> usize {
        match self.view_mode {
            ViewMode::Tree => self.visible_tree.len(),
            ViewMode::List | ViewMode::Columns => self.visible_entries.len(),
        }
    }

    fn update_columns_preview(&mut self) {
        if self.view_mode != ViewMode::Columns {
            return;
        }
        let Some(entry) = self
            .visible_entries
            .get(self.selection)
            .and_then(|index| self.entries.get(*index))
        else {
            self.columns_right.clear();
            self.columns_right_path = None;
            return;
        };
        if !entry.is_dir {
            self.columns_right.clear();
            self.columns_right_path = None;
            return;
        }
        if self.columns_right_path.as_ref() == Some(&entry.path) {
            return;
        }
        self.columns_right = list_dir(&entry.path).unwrap_or_default();
        self.columns_right_path = Some(entry.path.clone());
    }

    fn update_preview(&mut self) {
        let Some((path, _)) = self.selected_path() else {
            self.preview_path = None;
            self.preview_text.clear();
            self.preview_scroll = 0;
            return;
        };
        let path = path.clone();
        if self.preview_path.as_ref() == Some(&path) {
            return;
        }
        match crate::fs::build_preview(&path, 8192, 40) {
            Ok(text) => self.preview_text = text,
            Err(err) => self.preview_text = format!("preview error: {err}"),
        }
        self.preview_path = Some(path);
        self.preview_scroll = 0;
    }

    fn rebuild_visible(&mut self) {
        let query = self.search_query.to_ascii_lowercase();
        if query.is_empty() {
            self.visible_entries = (0..self.entries.len()).collect();
            self.visible_tree = (0..self.tree_entries.len()).collect();
            return;
        }

        self.visible_entries = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let name = entry
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_ascii_lowercase();
                if name.contains(&query) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        let mut include = std::collections::HashSet::new();
        for entry in &self.tree_entries {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_ascii_lowercase();
            if name.contains(&query) {
                let mut current = entry.path.as_path();
                include.insert(entry.path.clone());
                while let Some(parent) = current.parent() {
                    include.insert(parent.to_path_buf());
                    current = parent;
                }
            }
        }
        self.visible_tree = self
            .tree_entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if include.contains(&entry.path) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        self.coerce_selection();
    }

    pub fn add_bookmark_current(&mut self) -> anyhow::Result<()> {
        let Some((path, _)) = self.selected_path() else {
            self.status = "no selection to bookmark".to_string();
            return Ok(());
        };
        if !self.bookmarks.contains(path) {
            self.bookmarks.push(path.clone());
            self.bookmark_index = self.bookmarks.len().saturating_sub(1);
            self.status = "bookmark added".to_string();
            self.save_config()?;
        } else {
            self.status = "bookmark already exists".to_string();
        }
        Ok(())
    }

    pub fn remove_bookmark(&mut self) -> anyhow::Result<()> {
        if self.bookmarks.is_empty() {
            self.status = "no bookmarks".to_string();
            return Ok(());
        }
        let index = self.bookmark_index.min(self.bookmarks.len() - 1);
        self.bookmarks.remove(index);
        if self.bookmarks.is_empty() {
            self.bookmark_index = 0;
        } else if self.bookmark_index >= self.bookmarks.len() && self.bookmark_index > 0 {
            self.bookmark_index -= 1;
        }
        self.status = "bookmark removed".to_string();
        self.save_config()?;
        Ok(())
    }

    pub fn bookmark_next(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }
        self.bookmark_index = (self.bookmark_index + 1).min(self.bookmarks.len() - 1);
    }

    pub fn bookmark_prev(&mut self) {
        if self.bookmarks.is_empty() {
            return;
        }
        if self.bookmark_index == 0 {
            return;
        }
        self.bookmark_index -= 1;
    }

    pub fn open_bookmark(&mut self) -> anyhow::Result<()> {
        let path = self.bookmarks.get(self.bookmark_index).cloned();
        let Some(path) = path else {
            self.status = "no bookmark selected".to_string();
            return Ok(());
        };
        if !path.is_dir() {
            self.status = "bookmark missing".to_string();
            return Ok(());
        }
        self.remember_selection();
        self.set_current_dir(path)?;
        Ok(())
    }

    fn set_current_dir(&mut self, path: PathBuf) -> anyhow::Result<()> {
        self.current_dir = path;
        self.entries = list_dir(&self.current_dir)?;
        self.expanded.clear();
        self.expanded.insert(self.current_dir.clone());
        self.rebuild_tree()?;
        self.rebuild_visible();
        self.restore_selection();
        self.update_columns_preview();
        self.update_preview();
        self.save_config()?;
        Ok(())
    }

    fn remember_selection(&mut self) {
        if let Some((path, _)) = self.selected_path() {
            self.selection_memory
                .insert(self.current_dir.clone(), path.clone());
        }
    }

    fn restore_selection(&mut self) {
        let Some(entry_path) = self.selection_memory.get(&self.current_dir) else {
            self.selection = 0;
            return;
        };
        match self.view_mode {
            ViewMode::Tree => {
                if let Some(pos) = self
                    .visible_tree
                    .iter()
                    .filter_map(|index| self.tree_entries.get(*index))
                    .position(|entry| &entry.path == entry_path)
                {
                    self.selection = pos;
                } else {
                    self.selection = 0;
                }
            }
            ViewMode::List | ViewMode::Columns => {
                if let Some(pos) = self
                    .visible_entries
                    .iter()
                    .filter_map(|index| self.entries.get(*index))
                    .position(|entry| &entry.path == entry_path)
                {
                    self.selection = pos;
                } else {
                    self.selection = 0;
                }
            }
        }
    }

    fn save_config(&self) -> anyhow::Result<()> {
        let config = Config {
            bookmarks: self
                .bookmarks
                .iter()
                .map(|path| path.to_string_lossy().to_string())
                .collect(),
            last_dir: Some(self.current_dir.to_string_lossy().to_string()),
            openers: self.config.openers.clone(),
        };
        crate::fs::save_config(&config)
    }

    pub fn open_selected(&mut self) -> anyhow::Result<()> {
        let Some((path, is_dir)) = self.selected_path() else {
            self.status = "no selection to open".to_string();
            return Ok(());
        };
        if is_dir {
            self.status = "cannot open directory".to_string();
            return Ok(());
        }
        let opener = find_opener(&self.config.openers, path);
        let Some(opener) = opener else {
            self.status = "no opener configured".to_string();
            return Ok(());
        };
        let mut args = Vec::new();
        if opener.args.is_empty() {
            args.push(path.to_string_lossy().to_string());
        } else {
            for arg in &opener.args {
                if arg == "{path}" {
                    args.push(path.to_string_lossy().to_string());
                } else {
                    args.push(arg.clone());
                }
            }
        }
        std::process::Command::new(&opener.command)
            .args(&args)
            .spawn()?;
        self.status = format!("opened with {}", opener.name);
        Ok(())
    }

    pub fn start_prompt(&mut self, kind: PromptKind) {
        let source = self
            .selected_path()
            .map(|(path, _)| path.clone());
        let input = if kind == PromptKind::Search {
            self.search_query.clone()
        } else {
            String::new()
        };
        self.prompt = Some(Prompt {
            kind,
            input,
            source,
        });
    }

    pub fn push_prompt_char(&mut self, ch: char) {
        if let Some(prompt) = self.prompt.as_mut() {
            prompt.input.push(ch);
        }
    }

    pub fn pop_prompt_char(&mut self) {
        if let Some(prompt) = self.prompt.as_mut() {
            prompt.input.pop();
        }
    }

    pub fn cancel_prompt(&mut self) {
        self.prompt = None;
    }

    pub fn clear_search(&mut self) {
        if !self.search_query.is_empty() {
            self.search_query.clear();
            self.rebuild_visible();
            self.coerce_selection();
            self.update_columns_preview();
            self.update_preview();
            self.status = "search cleared".to_string();
        }
    }

    pub fn scroll_bookmarks(&mut self, up: bool) {
        if up {
            self.bookmark_prev();
        } else {
            self.bookmark_next();
        }
        self.focus = FocusPane::Bookmarks;
    }

    pub fn scroll_middle(&mut self, up: bool) {
        if up {
            self.select_prev();
        } else {
            self.select_next();
        }
        self.focus = FocusPane::Middle;
    }

    pub fn scroll_preview(&mut self, up: bool) {
        self.focus = FocusPane::Preview;
        let Some(areas) = &self.pane_areas else {
            return;
        };
        let height = areas.preview.height.saturating_sub(2) as usize;
        if height == 0 {
            return;
        }
        let lines = self.preview_text.lines().count();
        let max_scroll = lines.saturating_sub(height) as u16;
        if up {
            self.preview_scroll = self.preview_scroll.saturating_sub(1);
        } else {
            self.preview_scroll = (self.preview_scroll + 1).min(max_scroll);
        }
    }

    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            FocusPane::Bookmarks => FocusPane::Middle,
            FocusPane::Middle => FocusPane::Preview,
            FocusPane::Preview => FocusPane::Bookmarks,
        };
    }

    pub fn focus_prev(&mut self) {
        self.focus = match self.focus {
            FocusPane::Bookmarks => FocusPane::Preview,
            FocusPane::Middle => FocusPane::Bookmarks,
            FocusPane::Preview => FocusPane::Middle,
        };
    }

    pub fn toggle_select(&mut self) {
        let target = self.selected_path().map(|(path, _)| path.clone());
        if let Some(path) = target {
            if self.selected_items.contains(&path) {
                self.selected_items.remove(&path);
            } else {
                self.selected_items.insert(path);
            }
        }
    }

    pub fn delete_selection_prompt(&mut self) {
        self.start_prompt(PromptKind::DeleteSelection);
    }

    pub fn select_all_visible(&mut self) {
        self.selected_items.clear();
        match self.view_mode {
            ViewMode::Tree => {
                for index in &self.visible_tree {
                    if let Some(entry) = self.tree_entries.get(*index) {
                        self.selected_items.insert(entry.path.clone());
                    }
                }
            }
            ViewMode::List | ViewMode::Columns => {
                for index in &self.visible_entries {
                    if let Some(entry) = self.entries.get(*index) {
                        self.selected_items.insert(entry.path.clone());
                    }
                }
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_items.clear();
    }

    pub fn copy_selection(&mut self, cut: bool) {
        let mut items: Vec<PathBuf> = self.selected_items.iter().cloned().collect();
        if items.is_empty() {
            if let Some((path, _)) = self.selected_path() {
                items.push(path.clone());
            }
        }
        if items.is_empty() {
            self.status = "no selection to copy".to_string();
            return;
        }
        self.clipboard.items = items;
        self.clipboard.cut = cut;
        self.status = if cut { "cut" } else { "copied" }.to_string();
    }

    pub fn paste_selection(&mut self) -> anyhow::Result<()> {
        if self.clipboard.items.is_empty() {
            self.status = "clipboard empty".to_string();
            return Ok(());
        }
        let mut results = Vec::new();
        for source in self.clipboard.items.clone() {
            let file_name = source
                .file_name()
                .map(|name| name.to_os_string())
                .unwrap_or_default();
            let dest = self.current_dir.join(file_name);
            if self.clipboard.cut {
                crate::fs::move_entry(&source, &dest)?;
            } else {
                crate::fs::copy_entry(&source, &dest)?;
            }
            results.push(dest);
        }
        if self.clipboard.cut {
            self.clipboard.items.clear();
            self.clipboard.cut = false;
        }
        self.status = format!("pasted {}", results.len());
        self.refresh_entries()?;
        Ok(())
    }

    pub fn confirm_prompt(&mut self) -> anyhow::Result<()> {
        let Some(prompt) = self.prompt.take() else {
            return Ok(());
        };
        let result = self.execute_prompt(prompt);
        if let Err(err) = result {
            self.status = err.to_string();
            return Ok(());
        }
        self.refresh_entries()?;
        self.save_config()?;
        Ok(())
    }

    fn execute_prompt(&mut self, prompt: Prompt) -> anyhow::Result<()> {
        let input = prompt.input.trim();
        match prompt.kind {
            PromptKind::CreateFile => {
                if input.is_empty() {
                    self.status = "create: name required".to_string();
                    return Ok(());
                }
                let path = self.resolve_path(input);
                crate::fs::create_file(&path)?;
                self.status = format!("created file {}", path.display());
            }
            PromptKind::CreateDir => {
                if input.is_empty() {
                    self.status = "mkdir: name required".to_string();
                    return Ok(());
                }
                let path = self.resolve_path(input);
                crate::fs::create_dir(&path)?;
                self.status = format!("created dir {}", path.display());
            }
            PromptKind::Rename => {
                let Some(source) = prompt.source else {
                    self.status = "rename: no selection".to_string();
                    return Ok(());
                };
                if input.is_empty() {
                    self.status = "rename: name required".to_string();
                    return Ok(());
                }
                let dest = self.resolve_path(input);
                crate::fs::rename_entry(&source, &dest)?;
                self.status = format!("renamed to {}", dest.display());
            }
            PromptKind::Move => {
                let Some(source) = prompt.source else {
                    self.status = "move: no selection".to_string();
                    return Ok(());
                };
                if input.is_empty() {
                    self.status = "move: destination required".to_string();
                    return Ok(());
                }
                let mut dest = self.resolve_path(input);
                if dest.is_dir() {
                    if let Some(name) = source.file_name() {
                        dest = dest.join(name);
                    }
                }
                crate::fs::move_entry(&source, &dest)?;
                self.status = format!("moved to {}", dest.display());
            }
            PromptKind::Copy => {
                let Some(source) = prompt.source else {
                    self.status = "copy: no selection".to_string();
                    return Ok(());
                };
                if input.is_empty() {
                    self.status = "copy: destination required".to_string();
                    return Ok(());
                }
                let mut dest = self.resolve_path(input);
                if dest.is_dir() {
                    if let Some(name) = source.file_name() {
                        dest = dest.join(name);
                    }
                }
                crate::fs::copy_entry(&source, &dest)?;
                self.status = format!("copied to {}", dest.display());
            }
            PromptKind::Delete => {
                let Some(source) = prompt.source else {
                    self.status = "delete: no selection".to_string();
                    return Ok(());
                };
                if input != "y" {
                    self.status = "delete cancelled".to_string();
                    return Ok(());
                }
                crate::fs::delete_entry(&source)?;
                self.selected_items.remove(&source);
                self.status = format!("deleted {}", source.display());
            }
            PromptKind::DeleteSelection => {
                if input != "y" {
                    self.status = "delete cancelled".to_string();
                    return Ok(());
                }
                let items: Vec<PathBuf> = self.selected_items.iter().cloned().collect();
                if items.is_empty() {
                    self.status = "no selection to delete".to_string();
                    return Ok(());
                }
                for path in items {
                    crate::fs::delete_entry(&path)?;
                    self.selected_items.remove(&path);
                }
                self.status = "deleted selection".to_string();
            }
            PromptKind::Search => {
                self.search_query = input.to_string();
                self.status = if self.search_query.is_empty() {
                    "search cleared".to_string()
                } else {
                    format!("search: {}", self.search_query)
                };
                self.rebuild_visible();
                self.coerce_selection();
                self.update_columns_preview();
                self.update_preview();
            }
        }
        Ok(())
    }

    fn resolve_path(&self, input: &str) -> PathBuf {
        let path = PathBuf::from(input);
        if path.is_absolute() {
            path
        } else {
            self.current_dir.join(path)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: PathBuf,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    CreateFile,
    CreateDir,
    Rename,
    Move,
    Copy,
    Delete,
    DeleteSelection,
    Search,
}

#[derive(Debug, Clone)]
pub struct Prompt {
    pub kind: PromptKind,
    pub input: String,
    pub source: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct Clipboard {
    pub items: Vec<PathBuf>,
    pub cut: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneKind {
    Bookmarks,
    Middle,
    Preview,
}

#[derive(Debug, Clone, Copy)]
pub struct PaneAreas {
    pub bookmarks: Rect,
    pub middle: Rect,
    pub preview: Rect,
    pub status: Rect,
}

impl PaneAreas {
    pub fn pane_at(&self, col: u16, row: u16) -> Option<PaneKind> {
        if contains(self.bookmarks, col, row) {
            Some(PaneKind::Bookmarks)
        } else if contains(self.middle, col, row) {
            Some(PaneKind::Middle)
        } else if contains(self.preview, col, row) {
            Some(PaneKind::Preview)
        } else {
            None
        }
    }
}

fn contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

fn build_tree_entries(root: &PathBuf, expanded: &HashSet<PathBuf>) -> anyhow::Result<Vec<TreeEntry>> {
    let mut out = Vec::new();
    out.push(TreeEntry {
        path: root.clone(),
        depth: 0,
        is_dir: true,
        is_expanded: true,
    });
    append_children(root, 1, expanded, &mut out)?;
    Ok(out)
}

fn append_children(
    path: &PathBuf,
    depth: usize,
    expanded: &HashSet<PathBuf>,
    out: &mut Vec<TreeEntry>,
) -> anyhow::Result<()> {
    let children = list_dir(path)?;
    for child in children {
        let is_expanded = child.is_dir && expanded.contains(&child.path);
        out.push(TreeEntry {
            path: child.path.clone(),
            depth,
            is_dir: child.is_dir,
            is_expanded,
        });
        if is_expanded {
            append_children(&child.path, depth + 1, expanded, out)?;
        }
    }
    Ok(())
}

fn find_opener(openers: &[Opener], path: &PathBuf) -> Option<Opener> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let ext = ext.to_ascii_lowercase();
    if ext.is_empty() {
        return None;
    }
    for opener in openers {
        if opener
            .extensions
            .iter()
            .any(|item| item.eq_ignore_ascii_case(&ext))
        {
            return Some(opener.clone());
        }
    }
    None
}
