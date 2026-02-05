use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::App;
use crate::core::Note;
use crate::ui::layout::Focus;

pub struct BacklinksState {
    pub selected: usize,
    list_state: ListState,
}

impl BacklinksState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            selected: 0,
            list_state,
        }
    }

    pub fn move_down(&mut self, backlinks: &[&Note]) {
        if !backlinks.is_empty() && self.selected < backlinks.len() - 1 {
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

    pub fn reset(&mut self) {
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    pub fn selected_path<'a>(&self, backlinks: &'a [&Note]) -> Option<&'a PathBuf> {
        backlinks.get(self.selected).map(|note| &note.path)
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Backlinks;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let backlinks = if let Some(note) = app.selected_note() {
        app.vault.get_backlinks(&note.path)
    } else {
        Vec::new()
    };

    let title = format!(" Backlinks ({}) ", backlinks.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = if backlinks.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "   No backlinks",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        backlinks
            .iter()
            .enumerate()
            .map(|(i, backlink)| {
                let name = backlink
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown");

                let style = if is_focused && i == app.backlinks_state.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                ListItem::new(Line::from(vec![
                    Span::raw("  <- "),
                    Span::styled(name, style),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = app.backlinks_state.list_state.clone();
    frame.render_stateful_widget(list, area, &mut state);
}
