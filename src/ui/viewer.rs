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
        render_markdown(note)
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

fn render_markdown(note: &Note) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in note.content.lines() {
        lines.push(render_line(line, note));
    }

    Text::from(lines)
}

fn render_line(line: &str, note: &Note) -> Line<'static> {
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

    // Lists
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::White),
        ));
    }

    // Parse inline elements (tags, links, bold, etc.)
    render_inline(line, note)
}

fn render_inline(line: &str, _note: &Note) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for wiki-link [[...]]
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            // Flush current text
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }

            // Find closing ]]
            let start = i;
            i += 2;
            let mut link_text = String::new();

            while i + 1 < chars.len() && !(chars[i] == ']' && chars[i + 1] == ']') {
                link_text.push(chars[i]);
                i += 1;
            }

            if i + 1 < chars.len() {
                i += 2; // Skip ]]

                // Extract display text if present
                let display = if let Some(pipe_pos) = link_text.find('|') {
                    link_text[pipe_pos + 1..].to_string()
                } else {
                    link_text.clone()
                };

                spans.push(Span::styled(
                    format!("[[{}]]", display),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            } else {
                // Malformed link, just add as text
                current.push_str(&line[start..]);
                i = chars.len();
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
