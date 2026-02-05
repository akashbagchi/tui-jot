# tui-jot

A terminal-based notetaking application with wiki-style linking, built with Rust and Ratatui.

## Features

- **File browser** — Navigate your notes directory with a tree view; create and delete notes in-place
- **Markdown rendering** — Syntax highlighting for headings, bold, italic, code blocks, and links
- **In-app editing** — Vim-style modal editing with READ and EDIT modes
- **Wiki-links** — Link notes with `[[note-name]]` syntax; autocomplete suggestions while typing
- **Tags** — Organize with inline `#tags` and hierarchical `#parent/child` tags
- **Backlinks** — See which notes link to the current note
- **Keyboard-driven** — Vim-style navigation throughout

## Installation

Requires Rust 1.75+.

```bash
git clone https://github.com/cursim/tui-jot.git
cd tui-jot
cargo build --release
```

The binary will be at `target/release/tui-jot`.

## Usage

```bash
tui-jot
```

### Keybindings

#### Browser

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate down / up |
| `Enter` | Open note / expand directory |
| `a` | Create new note |
| `d` | Delete note |
| `Tab` | Switch to viewer |

#### Viewer

| Key | Action |
|-----|--------|
| `i` | Enter edit mode |
| `Esc` | Exit edit mode (saves) / return to browser |
| `j` / `k` | Scroll down / up |
| `Enter` | Follow highlighted link |
| `Ctrl+n` / `Ctrl+p` | Next / previous link |

#### Global

| Key | Action |
|-----|--------|
| `Ctrl+b` | Toggle backlinks panel |
| `Ctrl+e` | Open in external editor |
| `Ctrl+Shift+k` | Toggle help |
| `q` | Quit |

## Configuration

Config file location: `~/.local/share/com.tui-jot.tui-jot/config.toml`

```toml
[vault]
path = "~/notes"

[editor]
external = "nvim"  # or your preferred editor
```

## License

MIT — see [LICENSE](LICENSE) for details.
