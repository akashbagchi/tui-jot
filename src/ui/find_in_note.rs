use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use ropey::Rope;

use super::theme::Theme;

#[derive(Debug, Clone)]
pub struct FindMatch {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
}

pub struct FindInNoteState {
    pub query: String,
    pub matches: Vec<FindMatch>,
    pub current_match: usize,
    pub case_sensitive: bool,
}

impl FindInNoteState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
            case_sensitive: false,
        }
    }

    pub fn update_matches(&mut self, content: &Rope) {
        self.matches.clear();
        if self.query.is_empty() {
            return;
        }

        let query = if self.case_sensitive {
            self.query.clone()
        } else {
            self.query.to_lowercase()
        };

        for line_idx in 0..content.len_lines() {
            let line_text = content.line(line_idx).to_string();
            let search_text = if self.case_sensitive {
                line_text.clone()
            } else {
                line_text.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = search_text[start..].find(&query) {
                let col_start = start + pos;
                let col_end = col_start + query.len();
                self.matches.push(FindMatch {
                    line: line_idx,
                    col_start,
                    col_end,
                });
                start = col_start + 1;
            }
        }

        // Clamp current_match
        if !self.matches.is_empty() {
            self.current_match = self.current_match.min(self.matches.len() - 1);
        } else {
            self.current_match = 0;
        }
    }

    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = if self.current_match == 0 {
                self.matches.len() - 1
            } else {
                self.current_match - 1
            };
        }
    }

    pub fn current(&self) -> Option<&FindMatch> {
        self.matches.get(self.current_match)
    }

    pub fn jump_to_nearest(&mut self, line: usize) {
        if self.matches.is_empty() {
            return;
        }
        // Find the match closest to the given line
        let mut best = 0;
        let mut best_dist = usize::MAX;
        for (i, m) in self.matches.iter().enumerate() {
            let dist = if m.line >= line {
                m.line - line
            } else {
                line - m.line
            };
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        self.current_match = best;
    }

    pub fn toggle_case_sensitivity(&mut self) {
        self.case_sensitive = !self.case_sensitive;
    }

    pub fn has_match_on_line(&self, line: usize) -> bool {
        self.matches.iter().any(|m| m.line == line)
    }

    pub fn is_current_match_line(&self, line: usize) -> bool {
        self.current().map(|m| m.line == line).unwrap_or(false)
    }
}

pub fn render_find_bar(frame: &mut Frame, area: Rect, state: &FindInNoteState, t: &Theme) {
    // Render as a 1-line bar at the bottom of the given area
    let bar_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    let case_indicator = if state.case_sensitive { "[Aa]" } else { "[aa]" };

    let match_info = if state.matches.is_empty() {
        if state.query.is_empty() {
            String::new()
        } else {
            " No matches".to_string()
        }
    } else {
        format!(" {}/{}", state.current_match + 1, state.matches.len())
    };

    let line = Line::from(vec![
        Span::styled(
            " Find: ",
            Style::default()
                .fg(t.search_prompt)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(&state.query, Style::default().fg(t.fg0)),
        Span::styled(
            "_",
            Style::default()
                .fg(t.cursor_blink)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(&match_info, Style::default().fg(t.fg3)),
        Span::styled(format!("  {}", case_indicator), Style::default().fg(t.fg4)),
        Span::styled(
            "  Alt+c: toggle case  Esc: close",
            Style::default().fg(t.fg4),
        ),
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(t.status_bar_bg));
    frame.render_widget(bar, bar_area);
}
