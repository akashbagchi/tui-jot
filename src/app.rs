use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::config::Config;
use crate::core::{Index, Vault};
use crate::input::InputHandler;
use crate::ui::theme::Theme;
use crate::ui::{self, Focus};

/// State for the create note dialog
pub struct CreateNoteState {
    pub filename: String,    // User-typed name (without .md)
    pub parent_dir: PathBuf, // Directory to create in
}

/// State for the delete confirmation dialog
pub struct DeleteConfirmState {
    pub path: PathBuf,     // Relative path to delete
    pub name: String,      // Display name for dialog
    pub is_dir: bool,      // Whether the entry is a directory
    pub note_count: usize, // Number of notes inside (directories only)
}

pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub vault: Vault,
    pub index: Index,
    pub focus: Focus,
    pub should_quit: bool,
    pub browser_state: ui::BrowserState,
    pub viewer_scroll: u16,
    pub viewer_state: ui::ViewerState,
    pub backlinks_state: ui::BacklinksState,
    pub show_help: bool,
    pub create_note_state: Option<CreateNoteState>,
    pub delete_confirm_state: Option<DeleteConfirmState>,
    pub tag_filter_state: Option<ui::TagFilterState>,
    pub active_tag_filter: Option<String>,
    pub search_state: Option<ui::SearchState>,
    pub finder_state: Option<ui::FinderState>,
    pub graph_view_state: Option<ui::GraphViewState>,
}

impl App {
    pub fn new(config: Config) -> Result<Self> {
        let vault = Vault::open(&config.vault.path)?;
        let index = Index::build(&vault);
        let browser_state = ui::BrowserState::new(&vault);
        let theme = Theme::from_config(&config.ui);

        Ok(Self {
            config,
            theme,
            vault,
            index,
            focus: Focus::Browser,
            should_quit: false,
            browser_state,
            viewer_scroll: 0,
            viewer_state: ui::ViewerState::new(),
            backlinks_state: ui::BacklinksState::new(),
            show_help: false,
            create_note_state: None,
            delete_confirm_state: None,
            tag_filter_state: None,
            active_tag_filter: None,
            search_state: None,
            finder_state: None,
            graph_view_state: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = self.setup_terminal()?;

        let result = self.event_loop(&mut terminal).await;

        self.restore_terminal(&mut terminal)?;
        result
    }

    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn restore_terminal(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    async fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        loop {
            terminal.draw(|frame| ui::render(frame, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        InputHandler::handle(self, key, terminal)?;
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Returns visible entries filtered by the active tag filter (if any).
    /// When a tag filter is active, only shows notes that have that tag
    /// (plus their parent directories to preserve tree structure).
    pub fn filtered_visible_entries(&self) -> Vec<&crate::core::TreeEntry> {
        let entries = self.vault.visible_entries();

        let tag = match &self.active_tag_filter {
            Some(tag) => tag,
            None => return entries,
        };

        let matching_paths = match self.index.notes_with_tag(tag) {
            Some(paths) => paths,
            None => return Vec::new(),
        };

        // Include entries whose path matches the tag, or directories that are
        // ancestors of matching entries
        entries
            .into_iter()
            .filter(|entry| {
                if entry.is_dir {
                    // Keep directory if any matching note is under it
                    matching_paths.iter().any(|p| p.starts_with(&entry.path))
                } else {
                    matching_paths.contains(&entry.path)
                }
            })
            .collect()
    }

    pub fn selected_note(&self) -> Option<&crate::core::Note> {
        let entries = self.filtered_visible_entries();
        self.browser_state
            .selected_entry(&entries)
            .filter(|entry| !entry.is_dir)
            .and_then(|entry| self.vault.get_note(&entry.path))
    }

    pub fn refresh_vault(&mut self) -> Result<()> {
        // Preserve the currently selected path before refreshing
        let selected_path = {
            let entries = self.filtered_visible_entries();
            self.browser_state
                .selected_entry(&entries)
                .map(|e| e.path.clone())
        };

        self.vault = Vault::open(&self.config.vault.path)?;
        self.index = Index::build(&self.vault);
        self.browser_state = ui::BrowserState::new(&self.vault);
        self.backlinks_state.reset();

        // Restore selection if the path still exists
        if let Some(path) = selected_path {
            if let Some(index) = self
                .vault
                .visible_entries()
                .iter()
                .position(|e| e.path == path)
            {
                self.browser_state.select(index);
                // Also update viewer state to reflect the reloaded note
                if let Some(note) = self.vault.get_note(&path) {
                    self.viewer_state.update_links(note);
                }
            }
        }

        Ok(())
    }

    pub fn open_in_editor(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        let note_path = {
            let entries = self.filtered_visible_entries();
            self.browser_state
                .selected_entry(&entries)
                .filter(|e| !e.is_dir)
                .map(|e| self.vault.root.join(&e.path))
        };
        if let Some(note_path) = note_path {
            // Suspend TUI
            self.restore_terminal(terminal)?;

            // Launch editor
            std::process::Command::new(&self.config.editor.external)
                .arg(&note_path)
                .status()?;

            // Resume TUI
            *terminal = self.setup_terminal()?;
            terminal.clear()?;

            // Reload vault to pick up changes
            self.refresh_vault()?;
        }
        Ok(())
    }
}
