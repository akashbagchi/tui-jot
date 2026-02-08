use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::core::{self, Vault};
use crate::ui::theme::{self, Theme};

pub struct FinderState {
    pub query: String,
    pub results: Vec<(PathBuf, String)>, // (path, title)
    pub selected: usize,
    list_state: ListState,
}

impl FinderState {
    pub fn new(vault: &Vault) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut results: Vec<(PathBuf, String)> = vault
            .notes
            .iter()
            .map(|(path, note)| (path.clone(), note.title.clone()))
            .collect();
        results.sort_by(|a, b| a.1.cmp(&b.1));

        Self {
            query: String::new(),
            results,
            selected: 0,
            list_state,
        }
    }

    pub fn update_results(&mut self, vault: &Vault) {
        self.results.clear();
        self.selected = 0;
        self.list_state.select(Some(0));

        let query_lower = self.query.to_lowercase();

        for (path, note) in &vault.notes {
            let name_lower = note.title.to_lowercase();
            if query_lower.is_empty() || core::fuzzy_match(&query_lower, &name_lower) {
                self.results.push((path.clone(), note.title.clone()));
            }
        }

        // Sort: prefix matches first, then alphabetical
        self.results.sort_by(|a, b| {
            let a_starts = a.1.to_lowercase().starts_with(&query_lower);
            let b_starts = b.1.to_lowercase().starts_with(&query_lower);
            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.1.cmp(&b.1),
            }
        });

        self.results.truncate(20);
    }

    pub fn move_down(&mut self) {
        if !self.results.is_empty() && self.selected < self.results.len() - 1 {
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

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.results.get(self.selected).map(|(p, _)| p)
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &FinderState, t: &Theme) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 16u16.min(area.height.saturating_sub(4));

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {}Find Note ", theme::ICON_SEARCH))
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.finder_prompt))
        .style(Style::default().bg(t.bg0));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if inner.height < 3 {
        return;
    }

    // Input field
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let input = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(t.finder_prompt)),
        Span::styled(&state.query, Style::default().fg(t.fg1)),
        Span::styled(
            "_",
            Style::default()
                .fg(t.cursor_blink)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]));
    frame.render_widget(input, input_area);

    // Separator
    let sep_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(t.bg3),
    )));
    frame.render_widget(sep, sep_area);

    // Results
    let results_area = Rect::new(
        inner.x,
        inner.y + 2,
        inner.width,
        inner.height.saturating_sub(2),
    );

    if state.results.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "No matching notes",
            Style::default().fg(t.empty_hint),
        )));
        frame.render_widget(empty, results_area);
    } else {
        let items: Vec<ListItem> = state
            .results
            .iter()
            .enumerate()
            .map(|(i, (_path, title))| {
                let style = if i == state.selected {
                    t.selection_style()
                } else {
                    Style::default().fg(t.fg1)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {} ", theme::ICON_FILE),
                        if i == state.selected {
                            style
                        } else {
                            Style::default().fg(t.fg4)
                        },
                    ),
                    Span::styled(title, style),
                ]))
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(t.selected_bg)
                .add_modifier(Modifier::BOLD),
        );

        let mut list_state = state.list_state.clone();
        frame.render_stateful_widget(list, results_area, &mut list_state);
    }
}
