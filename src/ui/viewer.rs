use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::find_in_note::FindInNoteState;
use super::viewer_state::{AutocompleteState, EditorMode, ViewerState};
use crate::app::App;
use crate::core::Note;
use crate::ui::layout::Focus;
use crate::ui::theme::{self, Theme};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    // Store viewer area height for scroll-follow in input handler
    // Inner height = area height minus 2 for borders
    app.viewer_area_height = area.height.saturating_sub(2);
    let t = &app.theme;
    let is_focused = app.focus == Focus::Viewer;

    let mode_indicator = match app.viewer_state.mode {
        EditorMode::Read => " Preview ".to_string(),
        EditorMode::Edit => {
            if app.viewer_state.dirty {
                format!(" {}EDIT [modified] ", theme::ICON_EDIT)
            } else {
                format!(" {}EDIT ", theme::ICON_EDIT)
            }
        }
    };

    let block = Block::default()
        .title(mode_indicator)
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(t.border_style(is_focused));

    let content = if let Some(note) = app.selected_note() {
        match app.viewer_state.mode {
            EditorMode::Read => {
                let read_cursor_line = app.viewer_state.read_cursor.line;
                render_markdown(
                    note,
                    &app.viewer_state,
                    &app.vault,
                    t,
                    read_cursor_line,
                    app.find_in_note_state.as_ref(),
                )
            }
            EditorMode::Edit => render_edit_mode(&app.viewer_state, t),
        }
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a note to preview",
                Style::default().fg(t.empty_hint),
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
                render_autocomplete(frame, area, ac, &app.viewer_state, t);
            }
        }
    }

    // Set cursor position in EDIT mode, accounting for soft wrapping
    if is_focused && app.viewer_state.mode == EditorMode::Edit {
        let inner_width = area.width.saturating_sub(2) as usize; // minus borders
        if inner_width > 0 {
            let scroll = app.viewer_scroll as usize;

            // Count visual lines consumed by all logical lines before the cursor line
            let mut visual_y: usize = 0;
            for line_idx in 0..app.viewer_state.cursor.line {
                let line = app.viewer_state.content.line(line_idx);
                let line_len = {
                    let len = line.len_chars();
                    if len > 0 && line.char(len - 1) == '\n' {
                        len - 1
                    } else {
                        len
                    }
                };
                visual_y += visual_lines_for_width(line_len, inner_width);
            }

            // Add the wrap row within the cursor's own line
            let cursor_col = app.viewer_state.cursor.col;
            visual_y += cursor_col / inner_width;
            let visual_x = cursor_col % inner_width;

            // Subtract visual scroll offset
            let visible_y = visual_y.saturating_sub(scroll);

            let x = area.x + 1 + visual_x as u16;
            let y = area.y + 1 + visible_y as u16;

            if y >= area.y + 1 && y < area.y + area.height - 1 {
                frame.set_cursor_position((x, y));
            }
        }
    }
}

fn render_edit_mode(viewer_state: &ViewerState, t: &Theme) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let has_selection = viewer_state.selection.is_some();

    for line_idx in 0..viewer_state.content.len_lines() {
        let line_text = viewer_state.content.line(line_idx).to_string();

        if has_selection {
            // Render with per-character selection highlighting
            let chars: Vec<char> = line_text.chars().collect();
            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut current = String::new();
            let mut in_selection = false;

            for (col, &ch) in chars.iter().enumerate() {
                let selected = viewer_state.is_char_selected(line_idx, col);
                if selected != in_selection {
                    // Flush current span
                    if !current.is_empty() {
                        let style = if in_selection {
                            Style::default().bg(t.selection_bg)
                        } else {
                            Style::default()
                        };
                        spans.push(Span::styled(current.clone(), style));
                        current.clear();
                    }
                    in_selection = selected;
                }
                current.push(ch);
            }
            // Flush remaining
            if !current.is_empty() {
                let style = if in_selection {
                    Style::default().bg(t.selection_bg)
                } else {
                    Style::default()
                };
                spans.push(Span::styled(current, style));
            }
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(line_text));
        }
    }

    Text::from(lines)
}

fn render_autocomplete(
    frame: &mut Frame,
    area: Rect,
    ac: &AutocompleteState,
    viewer_state: &ViewerState,
    t: &Theme,
) {
    use ratatui::widgets::{List, ListItem};

    if ac.matches.is_empty() {
        return;
    }

    // Calculate popup position (near cursor)
    let cursor_y = ac
        .trigger_pos
        .line
        .saturating_sub(viewer_state.scroll_offset);
    let cursor_x = ac.trigger_pos.col + 2; // After [[

    let popup_height = (ac.matches.len() + 2).min(12) as u16;
    let popup_width = 30;

    // Position popup near cursor, but keep it within bounds
    let popup_x = (area.x + 1 + cursor_x as u16).min(area.width.saturating_sub(popup_width + 2));
    let popup_y =
        (area.y + 1 + cursor_y as u16 + 1).min(area.height.saturating_sub(popup_height + 1));

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
                    .fg(t.selected_fg)
                    .bg(t.autocomplete_sel_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.fg1)
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
            .border_type(theme::border_type())
            .border_style(Style::default().fg(t.border_overlay))
            .title(format!(" Notes ({}) ", ac.matches.len()))
            .style(Style::default().bg(t.autocomplete_bg)),
    );

    frame.render_widget(list, popup_area);
}

fn render_markdown(
    note: &Note,
    viewer_state: &ViewerState,
    vault: &crate::core::Vault,
    t: &Theme,
    read_cursor_line: usize,
    find_state: Option<&FindInNoteState>,
) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    for (line_idx, line) in note.content.lines().enumerate() {
        let mut rendered = render_line(line, note, viewer_state, line_idx, vault, t);

        // Priority: find_current > find_match > selection > cursor_line
        let is_current_find = find_state
            .map(|fs| fs.is_current_match_line(line_idx))
            .unwrap_or(false);
        let has_find_match = find_state
            .map(|fs| fs.has_match_on_line(line_idx))
            .unwrap_or(false);
        let is_selected = viewer_state.is_line_selected(line_idx);

        if is_current_find {
            rendered = rendered.style(Style::default().bg(t.find_current_bg));
        } else if has_find_match {
            rendered = rendered.style(Style::default().bg(t.find_match_bg));
        } else if is_selected {
            rendered = rendered.style(Style::default().bg(t.selection_bg));
        } else if line_idx == read_cursor_line {
            rendered = rendered.style(Style::default().bg(t.cursor_line_bg));
        }
        lines.push(rendered);
    }

    Text::from(lines)
}

fn render_line(
    line: &str,
    note: &Note,
    viewer_state: &ViewerState,
    line_idx: usize,
    vault: &crate::core::Vault,
    t: &Theme,
) -> Line<'static> {
    let trimmed = line.trim();

    // Headings
    if trimmed.starts_with("# ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(t.heading_1)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("## ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(t.heading_2)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("### ") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default()
                .fg(t.heading_3)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Code blocks (simple detection)
    if trimmed.starts_with("```") {
        return Line::from(Span::styled(line.to_string(), Style::default().fg(t.fg4)));
    }

    // Parse inline elements (tags, links, bold, etc.)
    render_inline(line, note, viewer_state, line_idx, vault, t)
}

fn render_inline(
    line: &str,
    _note: &Note,
    viewer_state: &ViewerState,
    line_idx: usize,
    vault: &crate::core::Vault,
    t: &Theme,
) -> Line<'static> {
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
                    (
                        link_text[..pipe_pos].to_string(),
                        link_text[pipe_pos + 1..].to_string(),
                    )
                } else {
                    (link_text.clone(), link_text.clone())
                };

                // Check if this is the selected link
                let is_selected = viewer_state
                    .visible_links
                    .get(viewer_state.selected_link)
                    .map(|selected| {
                        selected.line_index == line_idx
                            && viewer_state.visible_links[..viewer_state.selected_link]
                                .iter()
                                .filter(|l| l.line_index == line_idx)
                                .count()
                                == link_count_on_line
                    })
                    .unwrap_or(false);

                // Check if the link is broken
                let is_broken = !vault.link_exists(&target);

                let style = if is_broken {
                    if is_selected {
                        Style::default()
                            .fg(t.link_broken)
                            .bg(t.link_selected_bg)
                            .add_modifier(Modifier::BOLD | Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                            .fg(t.link_broken)
                            .add_modifier(Modifier::CROSSED_OUT)
                    }
                } else if is_selected {
                    Style::default()
                        .fg(t.link_selected_fg)
                        .bg(t.link_selected_bg)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default()
                        .fg(t.link_fg)
                        .add_modifier(Modifier::UNDERLINED)
                };

                spans.push(Span::styled(format!("[[{}]]", display), style));
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
                    && (chars[i].is_alphanumeric()
                        || chars[i] == '-'
                        || chars[i] == '_'
                        || chars[i] == '/')
                {
                    tag.push(chars[i]);
                    i += 1;
                }

                spans.push(Span::styled(
                    tag,
                    Style::default().fg(t.tag_fg).add_modifier(Modifier::ITALIC),
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
                    Style::default().fg(t.inline_code),
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

/// How many visual rows a line of `char_len` characters occupies in a column of `width`.
fn visual_lines_for_width(char_len: usize, width: usize) -> usize {
    if char_len == 0 || width == 0 {
        1
    } else {
        (char_len + width - 1) / width
    }
}
