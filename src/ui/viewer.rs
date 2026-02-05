pub struct ViewerState {
    pub selected_link: usize,
    pub visible_links: Vec<VisibleLink>,
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
        }
    }

    pub fn update_links(&mut self, note: &Note) {
        self.visible_links.clear();
        self.selected_link = 0;

        // Build list of visible links with their line positions
        let mut line_index = 0;
        for line in note.content.lines() {
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
}

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::core::Note;
use crate::ui::layout::Focus;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Viewer;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if let Some(note) = app.selected_note() {
        render_markdown(note, &app.viewer_state, &app.vault)
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a note to preview",
                Style::default().fg(Color::DarkGray),
            )),
        ])
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.viewer_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn render_markdown(note: &Note, viewer_state: &ViewerState, vault: &crate::core::Vault) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for (line_idx, line) in note.content.lines().enumerate() {
        lines.push(render_line(line, note, viewer_state, line_idx, vault));
    }

    Text::from(lines)
}

fn render_line(line: &str, note: &Note, viewer_state: &ViewerState, line_idx: usize, vault: &crate::core::Vault) -> Line<'static> {
    let trimmed = line.trim();

    // Headings
    if trimmed.starts_with("# ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("## ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("### ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Code blocks (simple detection)
    if trimmed.starts_with("```") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Parse inline elements (tags, links, bold, etc.) - includes lists
    render_inline(line, note, viewer_state, line_idx, vault)
}

fn render_inline(line: &str, _note: &Note, viewer_state: &ViewerState, line_idx: usize, vault: &crate::core::Vault) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut link_count_on_line = 0;

    while i < chars.len() {
        // Check for wiki-link [[...]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            // Flush current text
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            // Find closing ]]
            i += 2;
            let mut link_text = String::new();

            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == ']' && chars[i + 1] == ']' {
                    break;
                }
                link_text.push(chars[i]);
                i += 1;
            }

            // Check if we actually found ]]
            if i + 1 < chars.len() && chars[i] == ']' && chars[i + 1] == ']' {
                i += 2; // Skip ]]

                // Extract target and display text
                let (target, display) = if let Some(pipe_pos) = link_text.find('|') {
                    (link_text[..pipe_pos].to_string(), link_text[pipe_pos + 1..].to_string())
                } else {
                    (link_text.clone(), link_text.clone())
                };

                // Check if this is the selected link
                let is_selected = viewer_state.visible_links
                    .get(viewer_state.selected_link)
                    .map(|selected| {
                        selected.line_index == line_idx &&
                        viewer_state.visible_links[..viewer_state.selected_link]
                            .iter()
                            .filter(|l| l.line_index == line_idx)
                            .count() == link_count_on_line
                    })
                    .unwrap_or(false);

                // Check if the link is broken
                let is_broken = !vault.link_exists(&target);

                let style = if is_broken {
                    if is_selected {
                        Style::default()
                            .fg(Color::Red)
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD | Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::CROSSED_OUT)
                    }
                } else if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED)
                };

                spans.push(Span::styled(
                    format!("[[{}]]", display), style));
                link_count_on_line += 1;
            } else {
                current.push_str("[[");
                current.push_str(&link_text);
            }
            continue;
        }

        // Check for tag #...
        if chars[i] == '#' {
            let prev_is_valid = i == 0 || chars[i - 1].is_whitespace();

            if prev_is_valid && i + 1 < chars.len() && chars[i + 1].is_alphanumeric() {
                // Flush current text
                if !current.is_empty() {
                    spans.push(Span::raw(current.clone()));
                    current.clear();
                }

                // Collect tag
                let mut tag = String::from("#");
                i += 1;
                while i < chars.len()
                    && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == '/')
                {
                    tag.push(chars[i]);
                    i += 1;
                }

                spans.push(Span::styled(
                    tag,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                ));
                continue;
            }
        }

        // Check for bold **...**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            i += 2;
            let mut bold_text = String::new();

            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                bold_text.push(chars[i]);
                i += 1;
            }

            if i + 1 < chars.len() {
                i += 2;
                spans.push(Span::styled(
                    bold_text,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            } else {
                current.push_str("**");
                current.push_str(&bold_text);
            }
            continue;
        }

        // Check for inline code `...`
        if chars[i] == '`' {
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            i += 1;
            let mut code_text = String::new();

            while i < chars.len() && chars[i] != '`' {
                code_text.push(chars[i]);
                i += 1;
            }

            if i < chars.len() {
                i += 1;
                spans.push(Span::styled(
                    format!("`{}`", code_text),
                    Style::default().fg(Color::Red),
                ));
            } else {
                current.push('`');
                current.push_str(&code_text);
            }
            continue;
        }

        current.push(chars[i]);
        i += 1;
    }

    // Flush remaining text
    if !current.is_empty() {
        spans.push(Span::raw(current));
    }

    Line::from(spans)
}
