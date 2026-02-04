use std::path::Path;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::App;
use crate::core::{TreeEntry, Vault};
use crate::ui::layout::Focus;

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

    pub fn move_down(&mut self, vault: &Vault) {
        let visible = vault.visible_entries();
        if self.selected < visible.len().saturating_sub(1) {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn move_up(&mut self, _vault: &Vault) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn move_to_top(&mut self) {
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    pub fn move_to_bottom(&mut self, vault: &Vault) {
        let visible = vault.visible_entries();
        self.selected = visible.len().saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index;
        self.list_state.select(Some(index));
    }

    pub fn selected_entry<'a>(&self, vault: &'a Vault) -> Option<&'a TreeEntry> {
        vault.visible_entries().get(self.selected).copied()
    }

    pub fn selected_path(&self) -> Option<&Path> {
        None // Will be implemented with proper state
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Browser;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Notes ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let visible = app.vault.visible_entries();

    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = if entry.is_dir {
                if entry.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let name = if entry.is_dir {
                &entry.name
            } else {
                // Remove .md extension for display
                entry.name.strip_suffix(".md").unwrap_or(&entry.name)
            };

            let style = if i == app.browser_state.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
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
        .highlight_style(Style::default().bg(Color::DarkGray));

    let mut state = app.browser_state.list_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}
