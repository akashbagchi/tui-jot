use ropey::Rope;
use std::path::PathBuf;

use crate::core::{self, Note};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Read,
    Edit,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub trigger_pos: Position,
    pub query: String,
    pub matches: Vec<(PathBuf, String)>, // (path, display_name)
    pub selected: usize,
}

pub struct ViewerState {
    // Link navigation (READ mode)
    pub selected_link: usize,
    pub visible_links: Vec<VisibleLink>,

    // Editor state
    pub mode: EditorMode,
    pub content: Rope,
    pub cursor: Position,
    pub scroll_offset: usize,
    pub dirty: bool,
    pub current_note_path: Option<PathBuf>,
    pub autocomplete: Option<AutocompleteState>,
}

#[derive(Debug, Clone)]
pub struct VisibleLink {
    pub target: String,
    pub display: String,
    pub line_index: usize,
}

impl ViewerState {
    pub fn new() -> Self {
        Self {
            selected_link: 0,
            visible_links: Vec::new(),
            mode: EditorMode::Read,
            content: Rope::new(),
            cursor: Position { line: 0, col: 0 },
            scroll_offset: 0,
            dirty: false,
            current_note_path: None,
            autocomplete: None,
        }
    }

    pub fn update_links(&mut self, note: &Note) {
        self.visible_links.clear();
        self.selected_link = 0;
        self.current_note_path = Some(note.path.clone());

        // Update content rope
        self.content = Rope::from_str(&note.content);

        // Build list of visible links with their line Position
        let mut line_index = 0;
        for line in note.content.lines() {
            for link in &note.links {
                let line_start = note.content[..note.content.len().min(link.span.start)]
                    .lines()
                    .count()
                    .saturating_sub(1);

                if line_start == line_index {
                    self.visible_links.push(VisibleLink {
                        target: link.target.clone(),
                        display: link.display.clone().unwrap_or_else(|| link.target.clone()),
                        line_index,
                    });
                }
            }
            line_index += 1;
        }
    }

    pub fn next_link(&mut self) {
        if !self.visible_links.is_empty() {
            self.selected_link = (self.selected_link + 1) % self.visible_links.len();
        }
    }

    pub fn prev_link(&mut self) {
        if !self.visible_links.is_empty() {
            self.selected_link = if self.selected_link == 0 {
                self.visible_links.len() - 1
            } else {
                self.selected_link - 1
            };
        }
    }

    pub fn current_link(&self) -> Option<&VisibleLink> {
        self.visible_links.get(self.selected_link)
    }

    // EDIT mode operations
    pub fn enter_edit_mode(&mut self) {
        self.mode = EditorMode::Edit;
        self.cursor = Position { line: 0, col: 0 };
    }

    pub fn exit_edit_mode(&mut self) -> String {
        self.mode = EditorMode::Read;
        self.dirty = false;
        self.autocomplete = None;
        self.content.to_string()
    }

    pub fn insert_char(&mut self, c: char) {
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        self.content.insert_char(char_idx, c);
        self.cursor.col += 1;
        self.dirty = true;

        // Check for autocomplete trigger
        self.check_autocomplete_trigger();
    }

    pub fn insert_newline(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        self.content.insert_char(char_idx, '\n');
        self.cursor.line += 1;
        self.cursor.col = 0;
        self.dirty = true;
        self.autocomplete = None;
    }

    pub fn delete_char(&mut self) {
        if self.cursor.col > 0 {
            let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
            if char_idx > 0 {
                self.content.remove(char_idx - 1..char_idx);
                self.cursor.col -= 1;
                self.dirty = true;
                self.check_autocomplete_trigger();
            }
        } else if self.cursor.line > 0 {
            // Join with previous line
            let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
            if char_idx > 0 {
                let prev_line_len = self
                    .content
                    .line(self.cursor.line - 1)
                    .len_chars()
                    .saturating_sub(1);
                self.content.remove(char_idx - 1..char_idx);
                self.cursor.line -= 1;
                self.cursor.col = prev_line_len;
                self.dirty = true;
                self.autocomplete = None;
            }
        }
    }

    pub fn delete_forward(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        if char_idx < self.content.len_chars() {
            self.content.remove(char_idx..char_idx + 1);
            self.dirty = true;
            self.check_autocomplete_trigger();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.current_line_len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.current_line_len();
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.line < self.content.len_lines().saturating_sub(1) {
            self.cursor.line += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.cursor.col.min(self.current_line_len());
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor.line < self.content.len_lines().saturating_sub(1) {
            self.cursor.line += 1;
            self.cursor.col = self.cursor.col.min(self.current_line_len());
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        self.cursor.col = self.current_line_len();
    }

    fn current_line_len(&self) -> usize {
        if self.cursor.line < self.content.len_lines() {
            self.content
                .line(self.cursor.line)
                .len_chars()
                .saturating_sub(1)
        } else {
            0
        }
    }

    fn line_col_to_char_idx(&self, line: usize, col: usize) -> usize {
        if line >= self.content.len_lines() {
            return self.content.len_chars();
        }
        let line_start = self.content.line_to_char(line);
        let line_len = self.content.line(line).len_chars().saturating_sub(1);
        line_start + col.min(line_len)
    }

    fn check_autocomplete_trigger(&mut self) {
        // Check if we just typed the second '[' of '[['
        if self.cursor.col >= 2 {
            let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
            if char_idx >= 2 {
                let prev_two = self.content.slice((char_idx - 2)..char_idx).to_string();
                if prev_two == "[[" {
                    // Trigger autocomplete
                    self.autocomplete = Some(AutocompleteState {
                        trigger_pos: Position {
                            line: self.cursor.line,
                            col: self.cursor.col - 2,
                        },
                        query: String::new(),
                        matches: Vec::new(),
                        selected: 0,
                    });
                }
            }
        }

        // Update autocomplete query if active
        if let Some(ref ac) = self.autocomplete {
            // Extract values before mutable operations
            let trigger_line = ac.trigger_pos.line;
            let trigger_col = ac.trigger_pos.col;
            let trigger_idx = self.line_col_to_char_idx(trigger_line, trigger_col);
            let cursor_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);

            let new_query = if cursor_idx > trigger_idx + 2 {
                self.content
                    .slice((trigger_idx + 2)..cursor_idx)
                    .to_string()
            } else {
                String::new()
            };

            let should_close = new_query.contains("]]") || self.cursor.line != trigger_line;

            // Now do the mutable operations
            if should_close {
                self.autocomplete = None;
            } else if let Some(ref mut ac) = self.autocomplete {
                ac.query = new_query;
            }
        }
    }

    pub fn update_autocomplete_matches(&mut self, vault: &crate::core::Vault) {
        if let Some(ref mut ac) = self.autocomplete {
            ac.matches.clear();
            ac.selected = 0;

            let query_lower = ac.query.to_lowercase();

            // Simple fuzzy matching - collect all notes that contain query chars in order
            for (path, note) in &vault.notes {
                let name = note.title.to_lowercase();
                if query_lower.is_empty() || core::fuzzy_match(&query_lower, &name) {
                    ac.matches.push((path.clone(), note.title.clone()));
                }
            }

            // Sort by relevance (starts with query first, then alphabetically)
            ac.matches.sort_by(|a, b| {
                let a_starts = a.1.to_lowercase().starts_with(&query_lower);
                let b_starts = b.1.to_lowercase().starts_with(&query_lower);
                match (a_starts, b_starts) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.1.cmp(&b.1),
                }
            });

            // Limit to 10 results
            ac.matches.truncate(10);
        }
    }

    pub fn autocomplete_next(&mut self) {
        if let Some(ref mut ac) = self.autocomplete {
            if !ac.matches.is_empty() {
                ac.selected = (ac.selected + 1) % ac.matches.len();
            }
        }
    }

    pub fn autocomplete_prev(&mut self) {
        if let Some(ref mut ac) = self.autocomplete {
            if !ac.matches.is_empty() {
                ac.selected = if ac.selected == 0 {
                    ac.matches.len() - 1
                } else {
                    ac.selected - 1
                };
            }
        }
    }

    pub fn autocomplete_accept(&mut self) {
        if let Some(ac) = self.autocomplete.take() {
            if let Some((path, _)) = ac.matches.get(ac.selected) {
                // Remove the [[ and any query text
                let trigger_idx =
                    self.line_col_to_char_idx(ac.trigger_pos.line, ac.trigger_pos.col);
                let cursor_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
                self.content.remove(trigger_idx..cursor_idx);

                // Insert the completed link
                let link_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");
                let completion = format!("[[{}]]", link_name);
                self.content.insert(trigger_idx, &completion);

                // Move cursor after the ]]
                self.cursor.line = ac.trigger_pos.line;
                self.cursor.col = ac.trigger_pos.col + completion.len();
                self.dirty = true;
            }
        }
    }
}
