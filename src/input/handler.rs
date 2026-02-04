use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::ui::Focus;

pub struct InputHandler;

impl InputHandler {
    pub fn handle(app: &mut App, key: KeyEvent) {
        // Global keybindings (work in any focus)
        match key.code {
            KeyCode::Char('q') => {
                app.should_quit = true;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
                return;
            }
            KeyCode::Tab => {
                app.focus = app.focus.next();
                return;
            }
            _ => {}
        }

        // Context-specific keybindings
        match app.focus {
            Focus::Browser => Self::handle_browser(app, key),
            Focus::Viewer => Self::handle_viewer(app, key),
        }
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
        match key.code {
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
            KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => {
                // Go back to browser
                app.focus = Focus::Browser;
            }
            _ => {}
        }
    }
}
