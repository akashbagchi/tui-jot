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

#[derive(Debug, Clone)]
struct EditorSnapshot {
    content: Rope,
    cursor: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Visual,     // Read mode, line-level
    CharSelect, // Edit mode, char-level
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
    pub mode: SelectionMode,
}

impl Selection {
    /// Returns (start, end) positions in document order
    pub fn ordered(&self) -> (&Position, &Position) {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            (&self.anchor, &self.head)
        } else {
            (&self.head, &self.anchor)
        }
    }

    /// Returns (start_line, end_line) for visual mode
    pub fn line_range(&self) -> (usize, usize) {
        let (start, end) = self.ordered();
        (start.line, end.line)
    }
}

pub struct ViewerState {
    // Link navigation (READ mode)
    pub selected_link: usize,
    pub visible_links: Vec<VisibleLink>,

    // Editor state
    pub mode: EditorMode,
    pub content: Rope,
    pub cursor: Position,
    pub read_cursor: Position,
    pub scroll_offset: usize,
    pub dirty: bool,
    pub current_note_path: Option<PathBuf>,
    pub autocomplete: Option<AutocompleteState>,

    // Selection
    pub selection: Option<Selection>,
    pub clipboard: Option<String>,

    // Undo/Redo stacks
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
    max_undo_history: usize,
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
            read_cursor: Position { line: 0, col: 0 },
            scroll_offset: 0,
            dirty: false,
            current_note_path: None,
            autocomplete: None,
            selection: None,
            clipboard: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_undo_history: 100,
        }
    }

    pub fn update_links(&mut self, note: &Note) {
        self.visible_links.clear();
        self.selected_link = 0;
        self.selection = None;
        self.current_note_path = Some(note.path.clone());

        // Update content rope
        self.content = Rope::from_str(&note.content);

        // Reset cursors when loading new note
        self.cursor = Position { line: 0, col: 0 };
        self.read_cursor = Position { line: 0, col: 0 };

        // Clear undo/redo history when loading a new note
        self.undo_stack.clear();
        self.redo_stack.clear();

        // Build list of visible links with their line Position
        let mut line_index = 0;
        for _line in note.content.lines() {
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
        self.cursor = self.read_cursor.clone();
        self.selection = None;
        self.save_undo_snapshot();
    }

    pub fn exit_edit_mode(&mut self) -> String {
        self.mode = EditorMode::Read;
        self.dirty = false;
        self.autocomplete = None;
        self.selection = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.content.to_string()
    }

    fn save_undo_snapshot(&mut self) {
        let snapshot = EditorSnapshot {
            content: self.content.clone(),
            cursor: self.cursor.clone(),
        };

        self.undo_stack.push(snapshot);

        if self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.remove(0);
        }

        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current = EditorSnapshot {
                content: self.content.clone(),
                cursor: self.cursor.clone(),
            };
            self.redo_stack.push(current);

            // Restore snapshot
            self.content = snapshot.content;
            self.cursor = snapshot.cursor;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current = EditorSnapshot {
                content: self.content.clone(),
                cursor: self.cursor.clone(),
            };
            self.undo_stack.push(current);

            // Restore snapshot
            self.content = snapshot.content;
            self.cursor = snapshot.cursor;
            self.dirty = true;
            true
        } else {
            false
        }
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
        self.save_undo_snapshot();

        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        self.content.insert_char(char_idx, '\n');
        self.cursor.line += 1;
        self.cursor.col = 0;
        self.dirty = true;
        self.autocomplete = None;
    }

    pub fn delete_char(&mut self) {
        if self.cursor.col > 0 {
            self.save_undo_snapshot();

            let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
            if char_idx > 0 {
                self.content.remove(char_idx - 1..char_idx);
                self.cursor.col -= 1;
                self.dirty = true;
                self.check_autocomplete_trigger();
            }
        } else if self.cursor.line > 0 {
            self.save_undo_snapshot();

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
            self.save_undo_snapshot();

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

    // Word-based navigation for EDIT mode
    pub fn move_word_left(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        if char_idx == 0 {
            return;
        }

        let mut new_idx = char_idx.saturating_sub(1);

        // Skip whitespace
        while new_idx > 0 && self.content.char(new_idx).is_whitespace() {
            new_idx -= 1;
        }

        // Skip word characters
        while new_idx > 0 {
            let ch = self.content.char(new_idx.saturating_sub(1));
            if ch.is_whitespace() || is_word_separator(ch) {
                break;
            }
            new_idx -= 1;
        }

        let new_line = self.content.char_to_line(new_idx);
        let line_start = self.content.line_to_char(new_line);
        let new_col = new_idx - line_start;

        self.cursor.line = new_line;
        self.cursor.col = new_col;
    }

    pub fn move_word_right(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        let len = self.content.len_chars();
        if char_idx >= len {
            return;
        }

        let mut new_idx = char_idx;

        // Skip current word
        while new_idx < len {
            let ch = self.content.char(new_idx);
            if ch.is_whitespace() || is_word_separator(ch) {
                break;
            }
            new_idx += 1;
        }

        // Skip whitespace
        while new_idx < len && self.content.char(new_idx).is_whitespace() {
            new_idx += 1;
        }

        // Convert back to line/col
        let new_line = self.content.char_to_line(new_idx);
        let line_start = self.content.line_to_char(new_line);
        let new_col = new_idx - line_start;

        self.cursor.line = new_line;
        self.cursor.col = new_col;
    }

    pub fn move_read_cursor_left(&mut self) {
        if self.read_cursor.col > 0 {
            self.read_cursor.col -= 1;
        } else if self.read_cursor.line > 0 {
            self.read_cursor.line -= 1;
            self.read_cursor.col = self.read_line_len();
        }
    }

    pub fn move_read_cursor_right(&mut self) {
        let line_len = self.read_line_len();
        if self.read_cursor.col < line_len {
            self.read_cursor.col += 1;
        } else if self.read_cursor.line < self.content.len_lines().saturating_sub(1) {
            self.read_cursor.line += 1;
            self.read_cursor.col = 0;
        }
    }

    pub fn move_read_cursor_up(&mut self) {
        if self.read_cursor.line > 0 {
            self.read_cursor.line -= 1;
            self.read_cursor.col = self.read_cursor.col.min(self.read_line_len());
        }
    }

    pub fn move_read_cursor_down(&mut self) {
        if self.read_cursor.line < self.content.len_lines().saturating_sub(1) {
            self.read_cursor.line += 1;
            self.read_cursor.col = self.read_cursor.col.min(self.read_line_len());
        }
    }

    pub fn move_read_word_left(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.read_cursor.line, self.read_cursor.col);
        if char_idx == 0 {
            return;
        }

        let mut new_idx = char_idx.saturating_sub(1);

        // Skip whitespace
        while new_idx > 0 && self.content.char(new_idx).is_whitespace() {
            new_idx -= 1;
        }

        // Skip word characters
        while new_idx > 0 {
            let ch = self.content.char(new_idx.saturating_sub(1));
            if ch.is_whitespace() || is_word_separator(ch) {
                break;
            }
            new_idx -= 1;
        }

        // Convert back to line/col
        let new_line = self.content.char_to_line(new_idx);
        let line_start = self.content.line_to_char(new_line);
        let new_col = new_idx - line_start;

        self.read_cursor.line = new_line;
        self.read_cursor.col = new_col;
    }

    pub fn move_read_word_right(&mut self) {
        let char_idx = self.line_col_to_char_idx(self.read_cursor.line, self.read_cursor.col);
        let len = self.content.len_chars();
        if char_idx >= len {
            return;
        }

        let mut new_idx = char_idx;

        // Skip current word
        while new_idx < len {
            let ch = self.content.char(new_idx);
            if ch.is_whitespace() || is_word_separator(ch) {
                break;
            }
            new_idx += 1;
        }

        // Skip whitespace
        while new_idx < len && self.content.char(new_idx).is_whitespace() {
            new_idx += 1;
        }

        // Convert back to line/col
        let new_line = self.content.char_to_line(new_idx);
        let line_start = self.content.line_to_char(new_line);
        let new_col = new_idx - line_start;

        self.read_cursor.line = new_line;
        self.read_cursor.col = new_col;
    }

    fn read_line_len(&self) -> usize {
        if self.read_cursor.line < self.content.len_lines() {
            Self::line_content_len(self.content.line(self.read_cursor.line))
        } else {
            0
        }
    }

    fn line_content_len(line: ropey::RopeSlice) -> usize {
        let len = line.len_chars();
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    fn current_line_len(&self) -> usize {
        if self.cursor.line < self.content.len_lines() {
            Self::line_content_len(self.content.line(self.cursor.line))
        } else {
            0
        }
    }

    // ── Selection methods ──────────────────────────────────────────

    pub fn start_visual_selection(&mut self) {
        self.selection = Some(Selection {
            anchor: self.read_cursor.clone(),
            head: self.read_cursor.clone(),
            mode: SelectionMode::Visual,
        });
    }

    pub fn start_char_selection(&mut self) {
        if self.selection.is_none() {
            self.selection = Some(Selection {
                anchor: self.cursor.clone(),
                head: self.cursor.clone(),
                mode: SelectionMode::CharSelect,
            });
        }
    }

    pub fn update_selection_head(&mut self) {
        if let Some(ref mut sel) = self.selection {
            match sel.mode {
                SelectionMode::Visual => {
                    sel.head = self.read_cursor.clone();
                }
                SelectionMode::CharSelect => {
                    sel.head = self.cursor.clone();
                }
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn is_line_selected(&self, line: usize) -> bool {
        if let Some(ref sel) = self.selection {
            if sel.mode == SelectionMode::Visual {
                let (start, end) = sel.line_range();
                return line >= start && line <= end;
            }
        }
        false
    }

    pub fn is_char_selected(&self, line: usize, col: usize) -> bool {
        if let Some(ref sel) = self.selection {
            if sel.mode == SelectionMode::CharSelect {
                let (start, end) = sel.ordered();
                if line < start.line || line > end.line {
                    return false;
                }
                if start.line == end.line {
                    return col >= start.col && col < end.col;
                }
                if line == start.line {
                    return col >= start.col;
                }
                if line == end.line {
                    return col < end.col;
                }
                return true;
            }
        }
        false
    }

    pub fn selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        match sel.mode {
            SelectionMode::Visual => {
                let (start_line, end_line) = sel.line_range();
                let start_idx = self.content.line_to_char(start_line);
                let end_idx = if end_line + 1 < self.content.len_lines() {
                    self.content.line_to_char(end_line + 1)
                } else {
                    self.content.len_chars()
                };
                Some(self.content.slice(start_idx..end_idx).to_string())
            }
            SelectionMode::CharSelect => {
                let (start, end) = sel.ordered();
                let start_idx = self.line_col_to_char_idx(start.line, start.col);
                let end_idx = self.line_col_to_char_idx(end.line, end.col);
                if start_idx < end_idx {
                    Some(self.content.slice(start_idx..end_idx).to_string())
                } else {
                    None
                }
            }
        }
    }

    pub fn delete_selected_text(&mut self) -> Option<String> {
        let sel = self.selection.take()?;
        self.save_undo_snapshot();
        match sel.mode {
            SelectionMode::Visual => {
                let (start_line, end_line) = sel.line_range();
                let start_idx = self.content.line_to_char(start_line);
                let end_idx = if end_line + 1 < self.content.len_lines() {
                    self.content.line_to_char(end_line + 1)
                } else {
                    self.content.len_chars()
                };
                let text = self.content.slice(start_idx..end_idx).to_string();
                self.content.remove(start_idx..end_idx);
                self.read_cursor.line = start_line.min(
                    self.content.len_lines().saturating_sub(1),
                );
                self.read_cursor.col = 0;
                self.dirty = true;
                Some(text)
            }
            SelectionMode::CharSelect => {
                let (start, end) = sel.ordered();
                let start_pos = start.clone();
                let start_idx = self.line_col_to_char_idx(start.line, start.col);
                let end_idx = self.line_col_to_char_idx(end.line, end.col);
                if start_idx < end_idx {
                    let text = self.content.slice(start_idx..end_idx).to_string();
                    self.content.remove(start_idx..end_idx);
                    self.cursor = start_pos;
                    self.dirty = true;
                    Some(text)
                } else {
                    None
                }
            }
        }
    }

    pub fn paste_text(&mut self, text: &str) {
        self.save_undo_snapshot();
        let char_idx = self.line_col_to_char_idx(self.cursor.line, self.cursor.col);
        self.content.insert(char_idx, text);

        // Advance cursor past inserted text
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.len() > 1 {
            self.cursor.line += lines.len() - 1;
            self.cursor.col = lines.last().map(|l| l.len()).unwrap_or(0);
        } else {
            self.cursor.col += text.len();
        }
        self.dirty = true;
    }

    pub fn paste_text_at_read_cursor(&mut self, text: &str) {
        self.save_undo_snapshot();
        // Insert below the current read_cursor line
        let insert_line = self.read_cursor.line + 1;
        let char_idx = if insert_line < self.content.len_lines() {
            self.content.line_to_char(insert_line)
        } else {
            self.content.len_chars()
        };

        // Ensure text ends with newline if it doesn't
        let insert_text = if insert_line >= self.content.len_lines() && !text.starts_with('\n') {
            format!("\n{}", text)
        } else {
            text.to_string()
        };

        self.content.insert(char_idx, &insert_text);
        self.read_cursor.line = insert_line;
        self.read_cursor.col = 0;
        self.dirty = true;
    }

    fn line_col_to_char_idx(&self, line: usize, col: usize) -> usize {
        if line >= self.content.len_lines() {
            return self.content.len_chars();
        }
        let line_start = self.content.line_to_char(line);
        let line_len = Self::line_content_len(self.content.line(line));
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

fn is_word_separator(ch: char) -> bool {
    matches!(
        ch,
        '.' | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '"'
            | '\''
            | '`'
            | '/'
            | '\\'
            | '-'
            | '_'
    )
}
