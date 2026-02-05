# tui-jot

A terminal-based notetaking application with wiki-style linking, built with Rust and Ratatui.

## Features

- **File browser** — Navigate your notes directory with a tree view
- **Markdown preview** — Syntax highlighting for headings, bold, code, and more
- **Wiki-links** — Link notes with `[[note-name]]` syntax
- **Tags** — Organize with inline `#tags` and hierarchical `#parent/child` tags
- **Backlinks** — See which notes link to the current note
- **External editor** — Open notes in your preferred editor
- **Vim-style navigation** — Keyboard-driven interface

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

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate down / up |
| `Enter` | Open note or follow link |
| `Tab` | Switch between browser and preview |
| `Ctrl+b` | Toggle backlinks panel |
| `Ctrl+n` / `Ctrl+p` | Next / previous link (in preview) |
| `e` | Open in external editor |
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
