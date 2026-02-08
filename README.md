# tui-jot

A terminal-based notetaking app with wiki-style linking, built with Rust and [Ratatui](https://ratatui.rs).

[![CI](https://github.com/akashbagchi/tui-jot/actions/workflows/ci.yml/badge.svg)](https://github.com/akashbagchi/tui-jot/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **File browser** — Navigate your notes directory with a collapsible tree view; create and delete notes and subdirectories in-place
- **Markdown rendering** — Syntax highlighting for headings, bold, inline code, code blocks, tags, and wiki-links
- **In-app editing** — Modal editing with READ and EDIT modes; auto-saves on exit
- **Wiki-links** — Link notes with `[[note-name]]` or `[[target|display text]]` syntax; broken link detection with visual indicator
- **Link autocomplete** — Type `[[` in edit mode to get fuzzy-matched note suggestions
- **Tags** — Organize with inline `#tags` and hierarchical `#parent/child` tags
- **Tag filtering** — Filter the browser tree to show only notes matching a selected tag
- **Backlinks** — Dedicated panel showing which notes link to the current note
- **Full-text search** — Search across all notes with result highlighting
- **Fuzzy finder** — Quick note switching with `Ctrl+p`
- **Configurable themes** — 8 built-in color schemes (dark and light), with per-color overrides
- **External editor** — Open any note in your preferred editor with `Ctrl+e`
- **Keyboard-driven** — Vim-style navigation throughout; no mouse required

## Installation

Requires Rust 1.85+ (edition 2024).

```bash
git clone https://github.com/akashbagchi/tui-jot.git
cd tui-jot
cargo install --path .
```

Or build without installing:

```bash
cargo build --release
# Binary at target/release/tui-jot
```

## Usage

```bash
tui-jot
```

On first launch, tui-jot creates a default config file and uses `~/notes` as the vault directory. Place `.md` files in that directory (or change the path in the config).

### Notes syntax

```markdown
# My Note

Link to another note: [[other-note]]
Link with display text: [[other-note|click here]]

Tags: #project #status/active

**bold text** and `inline code`
```

## Keybindings

### Browser

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate down / up |
| `h` / `l` | Collapse / expand directory |
| `g` / `G` | Jump to top / bottom |
| `Enter` | Open note / toggle directory |
| `a` | Create note or directory (relative to selection) |
| `A` | Create note or directory (at vault root) |
| `d` | Delete note or directory |
| `t` | Filter by tag |
| `Tab` | Switch to viewer |

### Viewer (READ mode)

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down / up |
| `Ctrl+d` / `Ctrl+u` | Page down / up |
| `Ctrl+n` / `Ctrl+p` | Next / previous link |
| `Enter` | Follow selected link |
| `i` | Enter edit mode |
| `h` / `Esc` | Return to browser |
| `Tab` | Switch to browser |

### Editor (EDIT mode)

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| `Home` / `End` | Line start / end |
| `Ctrl+Left` / `Ctrl+Right` | Line start / end |
| `Backspace` / `Delete` | Delete character |
| `Enter` | New line |
| `[[` | Trigger link autocomplete |
| `Tab` / `Enter` | Accept autocomplete |
| `Esc` | Exit edit mode (auto-saves) |

### Global

| Key | Action |
|-----|--------|
| `/` | Full-text search |
| `Ctrl+p` | Fuzzy note finder |
| `Ctrl+e` | Open in external editor |
| `Ctrl+b` | Toggle backlinks panel |
| `Ctrl+Shift+K` | Toggle keybindings help |
| `Ctrl+q` | Quit |

## Themes

tui-jot ships with 8 built-in color schemes:

| Theme | Style |
|-------|-------|
| `gruvbox-dark` | Dark (default) |
| `gruvbox-light` | Light |
| `catppuccin-mocha` | Dark |
| `catppuccin-latte` | Light |
| `tokyo-night` | Dark |
| `tokyo-night-day` | Light |
| `nord` | Dark |
| `dracula` | Dark |

Set the theme in your config file:

```toml
[ui]
theme = "catppuccin-mocha"
```

You can override individual colors using hex values:

```toml
[ui.theme_overrides]
heading_1 = "#ff5555"
link_fg = "#8be9fd"
tag_fg = "#50fa7b"
```

## Configuration

Config file location: `~/.config/tui-jot/config.toml`

A default config is created on first launch. Full example:

```toml
[vault]
path = "~/notes"
default_extension = "md"

[ui]
tree_width = 25
show_hidden = false
show_backlinks = true
theme = "gruvbox-dark"

[ui.theme_overrides]
# heading_1 = "#ff5555"

[editor]
external = "nvim"    # defaults to $EDITOR
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT — see [LICENSE](LICENSE) for details.
