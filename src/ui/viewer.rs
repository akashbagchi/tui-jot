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
use super::viewer_state::{AutocompleteState, EditorMode, ViewerState};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Viewer;

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mode_indicator = match app.viewer_state.mode {
        EditorMode::Read => " Preview ",
        EditorMode::Edit => if app.viewer_state.dirty {
            " EDIT [modified] "
        } else {
            " EDIT "
        },
    };

    let block = Block::default()
        .title(mode_indicator)
        .borders(Borders::ALL)
        .border_style(border_style);

    let content = if let Some(note) = app.selected_note() {
        match app.viewer_state.mode {
            EditorMode::Read => render_markdown(note, &app.viewer_state, &app.vault),
            EditorMode::Edit => render_edit_mode(&app.viewer_state),
        }
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

    // Render autocomplete popup if active
    if app.viewer_state.mode == EditorMode::Edit {
        if let Some(ref ac) = app.viewer_state.autocomplete {
            if !ac.matches.is_empty() {
                render_autocomplete(frame, area, ac, &app.viewer_state);
            }
        }
    }

    // Set cursor position in EDIT mode
    if is_focused && app.viewer_state.mode == EditorMode::Edit {
        let cursor_line = app.viewer_state.cursor.line.saturating_sub(app.viewer_scroll as usize);
        let cursor_col = app.viewer_state.cursor.col;
        
        // Account for border (1 char) and ensure cursor is within visible area
        let x = area.x + 1 + cursor_col as u16;
        let y = area.y + 1 + cursor_line as u16;
        
        if y >= area.y + 1 && y < area.y + area.height - 1 {
            frame.set_cursor_position((x, y));
        }
    }
}

fn render_edit_mode(viewer_state: &ViewerState) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line_idx in 0..viewer_state.content.len_lines() {
        let line = viewer_state.content.line(line_idx).to_string();
        lines.push(Line::from(line));
    }

    Text::from(lines)
}

fn render_autocomplete(
    frame: &mut Frame,
    area: Rect,
    ac: &AutocompleteState,
    viewer_state: &ViewerState,
) {
    use ratatui::widgets::{List, ListItem};

    if ac.matches.is_empty() {
        return;
    }

    // Calculate popup position (near cursor)
    let cursor_y = ac.trigger_pos.line.saturating_sub(viewer_state.scroll_offset);
    let cursor_x = ac.trigger_pos.col + 2; // After [[

    let popup_height = (ac.matches.len() + 2).min(12) as u16;
    let popup_width = 30;

    // Position popup near cursor, but keep it within bounds
    let popup_x = (area.x + 1 + cursor_x as u16).min(area.width.saturating_sub(popup_width + 2));
    let popup_y = (area.y + 1 + cursor_y as u16 + 1).min(area.height.saturating_sub(popup_height + 1));

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    let items: Vec<ListItem> = ac
        .matches
        .iter()
        .enumerate()
        .map(|(i, (_, name)): (usize, &(std::path::PathBuf, String))| {
            let style = if i == ac.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let display = if name.len() > popup_width as usize - 4 {
                format!("{}...", &name[..popup_width as usize - 7])
            } else {
                name.clone()
            };

            ListItem::new(Line::from(Span::styled(format!(" {} ", display), style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Notes ({}) ", ac.matches.len()))
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(list, popup_area);
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

    // Parse inline elements (tags, links, bold, etc.)
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
