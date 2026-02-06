use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::core::Vault;

pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    list_state: ListState,
}

pub struct SearchResult {
    pub path: PathBuf,
    pub title: String,
    pub matched_line: String,
    pub line_number: usize,
}

impl SearchState {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            list_state,
        }
    }

    pub fn update_results(&mut self, vault: &Vault) {
        self.results.clear();
        self.selected = 0;
        self.list_state.select(Some(0));

        if self.query.len() < 2 {
            return;
        }

        let query_lower = self.query.to_lowercase();

        for note in vault.notes.values() {
            for (line_num, line) in note.content.lines().enumerate() {
                if line.to_lowercase().contains(&query_lower) {
                    self.results.push(SearchResult {
                        path: note.path.clone(),
                        title: note.title.clone(),
                        matched_line: line.trim().to_string(),
                        line_number: line_num + 1,
                    });
                }
            }
        }

        // Sort by title then line number
        self.results.sort_by(|a, b| {
            a.title.cmp(&b.title).then(a.line_number.cmp(&b.line_number))
        });

        // Limit results
        self.results.truncate(50);
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

    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.results.get(self.selected)
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &SearchState) {
    let popup_width = 70u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if inner.height < 3 {
        return;
    }

    // Input field
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let input = Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(Color::Green)),
        Span::raw(&state.query),
        Span::styled(
            "_",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ]));
    frame.render_widget(input, input_area);

    // Separator
    let sep_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, sep_area);

    // Results
    let results_area = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(2));

    if state.results.is_empty() {
        let msg = if state.query.len() < 2 {
            "Type to search..."
        } else {
            "No results"
        };
        let empty = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, results_area);
    } else {
        let items: Vec<ListItem> = state
            .results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                let style = if i == state.selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // Truncate matched line if too long
                let max_line_len = (popup_width as usize).saturating_sub(6);
                let matched = if result.matched_line.len() > max_line_len {
                    format!("{}...", &result.matched_line[..max_line_len.saturating_sub(3)])
                } else {
                    result.matched_line.clone()
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(&result.title, style),
                        Span::styled(
                            format!(":{}", result.line_number),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(Span::styled(
                        format!("  {}", matched),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        let mut list_state = state.list_state.clone();
        frame.render_stateful_widget(list, results_area, &mut list_state);
    }
}
