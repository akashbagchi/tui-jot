use std::io::Stdout;
use std::path::PathBuf;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::{App, CreateNoteState, DeleteConfirmState};
use crate::core::Index;
use crate::ui::{EditorMode, FinderState, Focus, GraphViewState, SearchState, TagFilterState};

pub struct InputHandler;

impl InputHandler {
    fn follow_link(app: &mut App, target: &str) {
        // Normalize target - strip .md extension for comparison
        let target_name = if target.ends_with(".md") {
            target.strip_suffix(".md").unwrap_or(target)
        } else {
            target
        };

        // Find the note by case-insensitive name match (handles subdirectories too)
        let found_path = app
            .vault
            .notes
            .keys()
            .find(|path| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|name| name.eq_ignore_ascii_case(target_name))
                    .unwrap_or(false)
            })
            .cloned();

        if let Some(target_path) = found_path {
            if let Some(index) = app
                .vault
                .visible_entries()
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

    pub fn handle(
        app: &mut App,
        key: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
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

        // Handle create note dialog
        if app.create_note_state.is_some() {
            Self::handle_create_dialog(app, key)?;
            return Ok(());
        }

        // Handle delete confirmation dialog
        if app.delete_confirm_state.is_some() {
            Self::handle_delete_dialog(app, key)?;
            return Ok(());
        }

        // Handle tag filter dialog
        if app.tag_filter_state.is_some() {
            Self::handle_tag_filter(app, key);
            return Ok(());
        }

        // Handle search dialog
        if app.search_state.is_some() {
            Self::handle_search(app, key);
            return Ok(());
        }

        // Handle finder dialog
        if app.finder_state.is_some() {
            Self::handle_finder(app, key);
            return Ok(());
        }

        // Handle graph view
        if app.graph_view_state.is_some() {
            Self::handle_graph_view(app, key);
            return Ok(());
        }

        // Global keybindings (work in any focus)
        match key.code {
            KeyCode::Char('q')
                if app.viewer_state.mode != EditorMode::Edit
                    && key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
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
                let old_focus = app.focus;
                app.focus = app.focus.next();

                // Sync viewer state when switching from Browser to Viewer
                if old_focus == Focus::Browser && app.focus == Focus::Viewer {
                    let path = {
                        let entries = app.filtered_visible_entries();
                        app.browser_state
                            .selected_entry(&entries)
                            .filter(|e| !e.is_dir)
                            .map(|e| e.path.clone())
                    };
                    if let Some(path) = path {
                        if let Some(note) = app.vault.get_note(&path) {
                            app.viewer_state.update_links(note);
                            app.viewer_scroll = 0;
                        }
                    }
                }
                return Ok(());
            }
            KeyCode::Char('/') if app.viewer_state.mode != EditorMode::Edit => {
                app.search_state = Some(SearchState::new());
                return Ok(());
            }
            KeyCode::Char('p')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && app.viewer_state.mode != EditorMode::Edit =>
            {
                app.finder_state = Some(FinderState::new(&app.vault));
                return Ok(());
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Open graph view (local graph centered on current note)
                let center_path = {
                    let entries = app.filtered_visible_entries();
                    app.browser_state
                        .selected_entry(&entries)
                        .filter(|e| !e.is_dir)
                        .map(|e| e.path.clone())
                };
                let size = terminal.size()?;
                let mut state = GraphViewState::new();
                if let Some(ref path) = center_path {
                    state.update_local(&app.vault, path, size.width, size.height);
                } else {
                    state.update_global(&app.vault, size.width, size.height);
                }
                app.graph_view_state = Some(state);
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
                app.browser_state
                    .move_down(app.filtered_visible_entries().len());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.browser_state.move_up();
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                let entry_info = {
                    let entries = app.filtered_visible_entries();
                    app.browser_state
                        .selected_entry(&entries)
                        .map(|e| (e.is_dir, e.path.clone()))
                };
                if let Some((is_dir, path)) = entry_info {
                    if is_dir {
                        app.vault.toggle_dir(&path);
                    } else {
                        app.focus = Focus::Viewer;
                        app.viewer_scroll = 0;
                        if let Some(note) = app.vault.get_note(&path) {
                            app.viewer_state.update_links(note);
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                let dir_path = {
                    let entries = app.filtered_visible_entries();
                    app.browser_state
                        .selected_entry(&entries)
                        .filter(|e| e.is_dir && e.expanded)
                        .map(|e| e.path.clone())
                };
                if let Some(path) = dir_path {
                    app.vault.toggle_dir(&path);
                }
            }
            KeyCode::Char('g') => {
                app.browser_state.move_to_top();
            }
            KeyCode::Char('G') => {
                app.browser_state
                    .move_to_bottom(app.filtered_visible_entries().len());
            }
            KeyCode::Char('A') => {
                // Create new note/directory in vault root
                app.create_note_state = Some(CreateNoteState {
                    filename: String::new(),
                    parent_dir: PathBuf::new(),
                });
            }
            KeyCode::Char('a') => {
                // Create new note - determine parent directory from selection
                let parent_dir = {
                    let entries = app.filtered_visible_entries();
                    if let Some(entry) = app.browser_state.selected_entry(&entries) {
                        if entry.is_dir {
                            entry.path.clone()
                        } else {
                            entry
                                .path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_default()
                        }
                    } else {
                        PathBuf::new()
                    }
                };

                app.create_note_state = Some(CreateNoteState {
                    filename: String::new(),
                    parent_dir,
                });
            }
            KeyCode::Char('t') => {
                // Open tag filter
                let tags = app.index.all_tags().into_iter().map(String::from).collect();
                app.tag_filter_state = Some(TagFilterState::new(tags));
            }
            KeyCode::Char('d') => {
                // Delete note or directory
                let delete_info = {
                    let entries = app.filtered_visible_entries();
                    app.browser_state
                        .selected_entry(&entries)
                        .map(|e| (e.path.clone(), e.name.clone(), e.is_dir))
                };
                if let Some((path, name, is_dir)) = delete_info {
                    let note_count = if is_dir {
                        app.vault
                            .notes
                            .keys()
                            .filter(|p| p.starts_with(&path))
                            .count()
                    } else {
                        0
                    };
                    app.delete_confirm_state = Some(DeleteConfirmState {
                        path,
                        name,
                        is_dir,
                        note_count,
                    });
                }
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
                KeyCode::Down | KeyCode::Char('n')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    app.viewer_state.autocomplete_next();
                    return;
                }
                KeyCode::Up | KeyCode::Char('p')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
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
                    // Reload the note and rebuild index
                    app.vault.reload_note(&path);
                    app.index = Index::build(&app.vault);
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
                    let backlinks = app.index.get_backlinks(&note.path);
                    app.backlinks_state.move_down(backlinks.len());
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.backlinks_state.move_up();
            }
            KeyCode::Enter => {
                // Navigate to selected backlink
                if let Some(note) = app.selected_note() {
                    let backlinks = app.index.get_backlinks(&note.path);
                    if let Some(target_path) = app.backlinks_state.selected_path(&backlinks) {
                        // Find this note in the browser tree
                        if let Some(index) = app
                            .vault
                            .visible_entries()
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

    fn handle_create_dialog(app: &mut App, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                app.create_note_state = None;
            }
            KeyCode::Enter => {
                if let Some(state) = app.create_note_state.take() {
                    if !state.filename.is_empty() {
                        Self::create_note(app, &state.parent_dir, &state.filename)?;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut state) = app.create_note_state {
                    state.filename.pop();
                }
            }
            KeyCode::Char(c) => {
                // Allow valid filename characters including '/' for directories
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '/' {
                    if let Some(ref mut state) = app.create_note_state {
                        state.filename.push(c);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_delete_dialog(app: &mut App, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(state) = app.delete_confirm_state.take() {
                    Self::delete_entry(app, &state.path, state.is_dir)?;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.delete_confirm_state = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tag_filter(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref mut state) = app.tag_filter_state {
                    state.move_down();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref mut state) = app.tag_filter_state {
                    state.move_up();
                }
            }
            KeyCode::Enter => {
                if let Some(state) = app.tag_filter_state.take() {
                    app.active_tag_filter = state.selected_tag().map(String::from);
                    app.browser_state.move_to_top();
                }
            }
            KeyCode::Esc => {
                app.tag_filter_state = None;
            }
            _ => {}
        }
    }

    fn handle_search(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.search_state = None;
            }

            // Navigate the list with Ctrl+n and Ctrl+p
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut state) = app.search_state {
                    state.move_down();
                }
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut state) = app.search_state {
                    state.move_up();
                }
            }

            // Alt: Navigate using arrow keys (no modifier needed)
            KeyCode::Down => {
                if let Some(ref mut state) = app.search_state {
                    state.move_down();
                }
            }
            KeyCode::Up => {
                if let Some(ref mut state) = app.search_state {
                    state.move_up();
                }
            }
            KeyCode::Enter => {
                let target_path = app
                    .search_state
                    .as_ref()
                    .and_then(|s| s.selected_result())
                    .map(|r| r.path.clone());

                if let Some(path) = target_path {
                    app.search_state = None;
                    // Navigate to the note
                    if let Some(index) = app
                        .filtered_visible_entries()
                        .iter()
                        .position(|e| e.path == path)
                    {
                        app.browser_state.select(index);
                        if let Some(note) = app.vault.get_note(&path) {
                            app.viewer_state.update_links(note);
                        }
                        app.viewer_scroll = 0;
                        app.focus = Focus::Viewer;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut state) = app.search_state {
                    state.query.pop();
                    state.update_results(&app.vault);
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut state) = app.search_state {
                    state.query.push(c);
                    state.update_results(&app.vault);
                }
            }
            _ => {}
        }
    }

    fn handle_finder(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.finder_state = None;
            }
            KeyCode::Down | KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut state) = app.finder_state {
                    state.move_down();
                }
            }
            KeyCode::Up | KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut state) = app.finder_state {
                    state.move_up();
                }
            }
            KeyCode::Enter => {
                let target_path = app
                    .finder_state
                    .as_ref()
                    .and_then(|s| s.selected_path())
                    .cloned();

                if let Some(path) = target_path {
                    app.finder_state = None;
                    if let Some(index) = app
                        .filtered_visible_entries()
                        .iter()
                        .position(|e| e.path == path)
                    {
                        app.browser_state.select(index);
                        if let Some(note) = app.vault.get_note(&path) {
                            app.viewer_state.update_links(note);
                        }
                        app.viewer_scroll = 0;
                        app.focus = Focus::Viewer;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut state) = app.finder_state {
                    state.query.pop();
                    state.update_results(&app.vault);
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut state) = app.finder_state {
                    state.query.push(c);
                    state.update_results(&app.vault);
                }
            }
            _ => {}
        }
    }

    fn handle_graph_view(app: &mut App, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                app.graph_view_state = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref mut state) = app.graph_view_state {
                    state.move_selection((0, 1));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref mut state) = app.graph_view_state {
                    state.move_selection((0, -1));
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if let Some(ref mut state) = app.graph_view_state {
                    state.move_selection((1, 0));
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(ref mut state) = app.graph_view_state {
                    state.move_selection((-1, 0));
                }
            }
            KeyCode::Enter => {
                // Navigate to the selected node
                let target = app
                    .graph_view_state
                    .as_ref()
                    .and_then(|s| s.selected_node.clone());
                if let Some(path) = target {
                    app.graph_view_state = None;
                    if let Some(index) = app
                        .filtered_visible_entries()
                        .iter()
                        .position(|e| e.path == path)
                    {
                        app.browser_state.select(index);
                        if let Some(note) = app.vault.get_note(&path) {
                            app.viewer_state.update_links(note);
                        }
                        app.viewer_scroll = 0;
                        app.focus = Focus::Viewer;
                    }
                }
            }
            _ => {}
        }
    }

    fn create_note(app: &mut App, parent_dir: &std::path::Path, filename: &str) -> Result<()> {
        // If filename ends with '/', create a standalone directory
        if filename.ends_with('/') {
            let dir_name = filename.trim_end_matches('/');
            let relative_path = parent_dir.join(dir_name);
            let full_path = app.vault.root.join(&relative_path);
            std::fs::create_dir_all(&full_path)?;
            app.refresh_vault()?;

            // Select the newly created directory
            if let Some(index) = app
                .vault
                .visible_entries()
                .iter()
                .position(|e| e.path == relative_path)
            {
                app.browser_state.select(index);
            }
            return Ok(());
        }

        // Build the relative path
        let relative_path = parent_dir.join(format!("{}.md", filename));
        let full_path = app.vault.root.join(&relative_path);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create the file with a basic header
        // Extract just the filename (not the path) for the title
        let title = relative_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename)
            .replace(['-', '_'], " ");

        let content = format!("# {}\n\n", title);
        std::fs::write(&full_path, content)?;

        // Refresh vault to pick up the new file
        app.refresh_vault()?;

        // Select the newly created note
        if let Some(index) = app
            .vault
            .visible_entries()
            .iter()
            .position(|e| e.path == relative_path)
        {
            app.browser_state.select(index);
            if let Some(note) = app.vault.get_note(&relative_path) {
                app.viewer_state.update_links(note);
            }
        }

        Ok(())
    }

    fn delete_entry(app: &mut App, path: &PathBuf, is_dir: bool) -> Result<()> {
        let full_path = app.vault.root.join(path);

        // Delete the file or directory (including contents)
        if is_dir {
            std::fs::remove_dir_all(&full_path)?;
        } else {
            std::fs::remove_file(&full_path)?;
        }

        // Get current selection before refresh
        let current_idx = app.browser_state.selected;

        // Refresh vault
        app.refresh_vault()?;

        // Adjust selection if needed (stay in bounds)
        let visible_count = app.filtered_visible_entries().len();
        if visible_count > 0 {
            let new_idx = current_idx.min(visible_count - 1);
            app.browser_state.select(new_idx);

            // Update viewer state if we have a selection
            let note_path = {
                let entries = app.filtered_visible_entries();
                app.browser_state
                    .selected_entry(&entries)
                    .filter(|e| !e.is_dir)
                    .map(|e| e.path.clone())
            };
            if let Some(path) = note_path {
                if let Some(note) = app.vault.get_note(&path) {
                    app.viewer_state.update_links(note);
                }
            }
        }

        Ok(())
    }
}
