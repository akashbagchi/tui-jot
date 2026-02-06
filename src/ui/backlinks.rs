use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::app::App;
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

    pub fn move_down(&mut self, count: usize) {
        if count > 0 && self.selected < count - 1 {
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

    pub fn selected_path<'a>(&self, backlinks: &'a [PathBuf]) -> Option<&'a PathBuf> {
        backlinks.get(self.selected)
    }
}

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Backlinks;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let backlink_paths = if let Some(note) = app.selected_note() {
        app.index.get_backlinks(&note.path)
    } else {
        Vec::new()
    };

    let title = format!(" Backlinks ({}) ", backlink_paths.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = if backlink_paths.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "   No backlinks",
            Style::default().fg(Color::DarkGray),
        )))]
    } else {
        backlink_paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = path
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
