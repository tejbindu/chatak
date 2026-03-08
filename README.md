# Chatak

Chatak is a fast, terminal-first file manager built with Rust and Ratatui. It combines a tree explorer, a list view, and a two-column browser with a live preview pane, so you can move through projects without ever leaving your keyboard.

## Highlights

- Three view modes: tree, list, and dual-column browsing.
- Persistent bookmarks for your favorite directories.
- Live preview pane with file metadata, snippet, or directory summary.
- Multi-select workflow with copy, cut, paste, and delete.
- In-app prompts for create, rename, move, copy, and delete.
- Search filter for quick narrowing of large folders.
- Configurable "open with" actions by file extension.
- Mouse wheel support for scrolling in any pane.
- Dracula-inspired theme and file-type icons.

## Layout

Chatak uses a focused, three-pane layout:

- Left: bookmarks
- Center: tree/list/columns browser
- Right: preview (metadata + content snippet)
- Bottom: status and prompt line

Press `?` at any time to see the built-in help overlay.

## Getting Started

### Build and run

```bash
cargo build --release
./target/release/chatak
```

Run it in any directory to start browsing that location.

### Default openers

By default Chatak ships with opener examples:

- `pdf` -> `zathura`
- `images` -> `feh`
- `text` -> `nvim`

You can change these in the config file (see below). If a configured command is missing, the open action will fail and show a status message.

## Keybindings

### Navigation

- `j/k` or arrow keys: move selection
- `h` or Backspace: go to parent directory
- `l` or Enter: enter directory (or open bookmark when focused there)
- `Tab` / `Shift+Tab`: cycle focus between panes
- Space: expand/collapse in tree view
- `o`: open selected file with configured opener

### Views

- `1`: tree view
- `2`: list view
- `3`: columns view
- `v`: cycle view modes

### Bookmarks

- `b`: add bookmark for selection
- `B`: remove selected bookmark
- `[` / `]`: previous / next bookmark
- `g`: open selected bookmark

### Selection + Clipboard

- `s`: toggle selection
- `A`: select all visible
- `u`: clear selection
- `y`: copy selection
- `x`: cut selection
- `p`: paste into current directory
- `D`: delete selection (confirmation required)

### File Ops

- `n`: new file
- `N`: new directory
- `r`: rename
- `m`: move
- `c`: copy
- `d`: delete (confirmation required)

### Search

- `/`: search (case-insensitive substring)
- `X` or `Esc`: clear search

### Help + Quit

- `?`: toggle help
- `q`: quit

## Configuration

Config lives at:

- `$XDG_CONFIG_HOME/chatak/config.json`, or
- `~/.config/chatak/config.json`

Example:

```json
{
  "bookmarks": ["/home/me/projects", "/var/log"],
  "last_dir": "/home/me",
  "openers": [
    {
      "name": "pdf",
      "extensions": ["pdf"],
      "command": "zathura",
      "args": ["{path}"]
    },
    {
      "name": "images",
      "extensions": ["png", "jpg", "jpeg", "gif"],
      "command": "feh",
      "args": ["{path}"]
    },
    {
      "name": "text",
      "extensions": ["md", "txt", "rs", "toml"],
      "command": "nvim",
      "args": ["{path}"]
    }
  ]
}
```

`{path}` is replaced with the selected file path. When `args` is empty, the path is appended automatically.

## Development

```bash
cargo run
```

This runs the debug build from your current directory.
