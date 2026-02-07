use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::App;
use crate::core::{TreeEntry, Vault};
use crate::ui::layout::Focus;
use crate::ui::theme;

pub struct BrowserState {
    pub selected: usize,
    list_state: ListState,
}

impl BrowserState {
    pub fn new(_vault: &Vault) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            selected: 0,
            list_state,
        }
    }

    pub fn move_down(&mut self, count: usize) {
        if self.selected < count.saturating_sub(1) {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn move_to_top(&mut self) {
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    pub fn move_to_bottom(&mut self, count: usize) {
        self.selected = count.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index;
        self.list_state.select(Some(index));
    }

    pub fn selected_entry<'a>(&self, entries: &'a [&TreeEntry]) -> Option<&'a TreeEntry> {
        entries.get(self.selected).copied()
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let is_focused = app.focus == Focus::Browser;

    let title = if let Some(ref tag) = app.active_tag_filter {
        format!(" Notes [{}#{}] ", theme::ICON_TAG, tag)
    } else {
        " Notes ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(t.border_style(is_focused));

    let visible = app.filtered_visible_entries();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded {
                    theme::ICON_FOLDER_OPEN
                } else {
                    theme::ICON_FOLDER_CLOSED
                }
            } else {
                theme::ICON_FILE
            };

            let name = if entry.is_dir {
                &entry.name
            } else {
                // Remove .md extension for display
                entry.name.strip_suffix(".md").unwrap_or(&entry.name)
            };

            let style = if i == app.browser_state.selected {
                t.selection_style()
            } else if entry.is_dir {
                Style::default().fg(t.dir_fg)
            } else {
                Style::default().fg(t.file_fg)
            };

            let line = Line::from(vec![
                Span::raw(indent),
                Span::styled(icon, style),
                Span::styled(name, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(t.selection_style());

    let mut state = app.browser_state.list_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}
