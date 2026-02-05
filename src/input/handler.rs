use std::io::Stdout;
use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::ui::{EditorMode, Focus};

pub struct InputHandler;

impl InputHandler {
    fn follow_link(app: &mut App, target: &str) {
        // Normalize target - add .md extension if not present
        let target_path = if target.ends_with(".md") {
            PathBuf::from(target)
        } else {
            PathBuf::from(format!("{}.md", target))
        };

        // Check if note exists
        if app.vault.get_note(&target_path).is_some() {
            if let Some(index) = app.vault.visible_entries()
                .iter()
                .position(|e| e.path == target_path)
            {
                app.browser_state.select(index);
                if let Some(note) = app.vault.get_note(&target_path) {
                    app.viewer_state.update_links(note);
                }
                app.viewer_scroll = 0;
            }
        }
    }

    pub fn handle(app: &mut App, key: KeyEvent, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        if app.show_help {
            match key.code {
                KeyCode::Esc => {
                    app.show_help = false;
                }
                KeyCode::Char('k') | KeyCode::Char('K')
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.modifiers.contains(KeyModifiers::SHIFT) =>
                {
                    app.show_help = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Global keybindings (work in any focus)
        match key.code {
            KeyCode::Char('q') => {
                app.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('k') | KeyCode::Char('K')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                app.show_help = true;
                return Ok(());
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Open in external editor
                app.open_in_editor(terminal)?;
                return Ok(());
            }
            KeyCode::Tab => {
                app.focus = app.focus.next();
                return Ok(());
            }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Toggle backlinks panel
                app.focus = if app.focus == Focus::Backlinks {
                    Focus::Browser
                } else {
                    Focus::Backlinks
                };
                return Ok(());
            }
            _ => {}
        }

        // Context-specific keybindings
        match app.focus {
            Focus::Browser => Self::handle_browser(app, key),
            Focus::Viewer => Self::handle_viewer(app, key),
            Focus::Backlinks => Self::handle_backlinks(app, key),
        }

        Ok(())
    }

    fn handle_browser(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.browser_state.move_down(&app.vault);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.browser_state.move_up(&app.vault);
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if let Some(entry) = app.browser_state.selected_entry(&app.vault) {
                    if entry.is_dir {
                        let path = entry.path.clone();
                        app.vault.toggle_dir(&path);
                    } else {
                        // Switch to viewer when opening a note
                        app.focus = Focus::Viewer;
                        app.viewer_scroll = 0;
                        if let Some(note) = app.vault.get_note(&entry.path) {
                            app.viewer_state.update_links(note);
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                // Collapse directory or move to parent
                if let Some(entry) = app.browser_state.selected_entry(&app.vault) {
                    if entry.is_dir && entry.expanded {
                        let path = entry.path.clone();
                        app.vault.toggle_dir(&path);
                    }
                }
            }
            KeyCode::Char('g') => {
                app.browser_state.move_to_top();
            }
            KeyCode::Char('G') => {
                app.browser_state.move_to_bottom(&app.vault);
            }
            _ => {}
        }
    }

    fn handle_viewer(app: &mut App, key: KeyEvent) {
        match app.viewer_state.mode {
            EditorMode::Read => Self::handle_viewer_read(app, key),
            EditorMode::Edit => Self::handle_viewer_edit(app, key),
        }
    }

    fn handle_viewer_read(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') => {
                // Enter edit mode
                if app.selected_note().is_some() {
                    app.viewer_state.enter_edit_mode();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.viewer_scroll = app.viewer_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.viewer_scroll = app.viewer_scroll.saturating_sub(1);
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.viewer_scroll = app.viewer_scroll.saturating_add(10);
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.viewer_scroll = app.viewer_scroll.saturating_sub(10);
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.viewer_state.next_link();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.viewer_state.prev_link();
            }
            KeyCode::Enter => {
                // Follow the current link
                if let Some(target) = app.viewer_state.current_link().map(|l| l.target.clone()) {
                    Self::follow_link(app, &target);
                }
            }
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => {
                // Go back to browser
                app.focus = Focus::Browser;
            }
            _ => {}
        }
    }

    fn handle_viewer_edit(app: &mut App, key: KeyEvent) {
        // Handle autocomplete navigation first if active
        if app.viewer_state.autocomplete.is_some() {
            match key.code {
                KeyCode::Down | KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.viewer_state.autocomplete_next();
                    return;
                }
                KeyCode::Up | KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.viewer_state.autocomplete_prev();
                    return;
                }
                KeyCode::Tab | KeyCode::Enter => {
                    app.viewer_state.autocomplete_accept();
                    app.viewer_state.update_autocomplete_matches(&app.vault);
                    return;
                }
                KeyCode::Esc => {
                    app.viewer_state.autocomplete = None;
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => {
                // Exit edit mode and save
                let content = app.viewer_state.exit_edit_mode();
                if let Some(path) = app.viewer_state.current_note_path.clone() {
                    let full_path = app.vault.root.join(&path);
                    let _ = std::fs::write(&full_path, &content);
                    // Reload the note
                    app.vault.reload_note(&path);
                    if let Some(note) = app.vault.get_note(&path) {
                        app.viewer_state.update_links(note);
                    }
                }
            }
            KeyCode::Char(c) => {
                app.viewer_state.insert_char(c);
                app.viewer_state.update_autocomplete_matches(&app.vault);
            }
            KeyCode::Enter => {
                app.viewer_state.insert_newline();
            }
            KeyCode::Backspace => {
                app.viewer_state.delete_char();
                app.viewer_state.update_autocomplete_matches(&app.vault);
            }
            KeyCode::Delete => {
                app.viewer_state.delete_forward();
                app.viewer_state.update_autocomplete_matches(&app.vault);
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Word movement - simplified: just move to start of line
                    app.viewer_state.move_to_line_start();
                } else {
                    app.viewer_state.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Word movement - simplified: just move to end of line
                    app.viewer_state.move_to_line_end();
                } else {
                    app.viewer_state.move_cursor_right();
                }
            }
            KeyCode::Up => {
                app.viewer_state.move_cursor_up();
            }
            KeyCode::Down => {
                app.viewer_state.move_cursor_down();
            }
            KeyCode::Home => {
                app.viewer_state.move_to_line_start();
            }
            KeyCode::End => {
                app.viewer_state.move_to_line_end();
            }
            _ => {}
        }
    }

    fn handle_backlinks(app: &mut App, key: KeyEvent) {
        match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if let Some(note) = app.selected_note() {
                let backlinks = app.vault.get_backlinks(&note.path);
                app.backlinks_state.move_down(&backlinks);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.backlinks_state.move_up();
        }
        KeyCode::Enter => {
            // Navigate to selected backlink
            if let Some(note) = app.selected_note() {
                let backlinks = app.vault.get_backlinks(&note.path);
                if let Some(target_path) = app.backlinks_state.selected_path(&backlinks) {
                    // Find this note in the browser tree
                    if let Some(index) = app.vault.visible_entries()
                        .iter()
                        .position(|e| &e.path == target_path)
                    {
                        app.browser_state.select(index);
                        if let Some(note) = app.vault.get_note(target_path) {
                            app.viewer_state.update_links(note);
                        }
                        app.viewer_scroll = 0;
                        app.backlinks_state.reset();
                        app.focus = Focus::Viewer;
                    }
                }
            }
        }
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => {
            app.focus = Focus::Browser;
        }
        _ => {}
    }
}
}
