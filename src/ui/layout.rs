use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, CreateNoteState, DeleteConfirmState};

use super::theme;
use super::{backlinks, browser, finder, search, tag_filter, viewer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Browser,
    Viewer,
    Backlinks,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Browser => Focus::Viewer,
            Focus::Viewer => Focus::Browser,
            Focus::Backlinks => Focus::Browser,
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let t = &app.theme;

    // Fill entire screen with theme background
    let bg = Block::default().style(Style::default().bg(t.bg0).fg(t.fg1));
    frame.render_widget(bg, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    render_title_bar(frame, chunks[0], app);
    render_main(frame, chunks[1], app);
    render_status_bar(frame, chunks[2], app);

    if app.show_help {
        render_help(frame, app);
    }

    if let Some(state) = &app.create_note_state {
        render_create_dialog(frame, state, app);
    }

    if let Some(state) = &app.delete_confirm_state {
        render_delete_dialog(frame, state, app);
    }

    if let Some(state) = &app.tag_filter_state {
        tag_filter::render(frame, frame.area(), state, t);
    }

    if let Some(state) = &app.search_state {
        search::render(frame, frame.area(), state, t);
    }

    if let Some(state) = &app.finder_state {
        finder::render(frame, frame.area(), state, t);
    }
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let title = Line::from(vec![
        Span::styled(
            format!(" {}", theme::ICON_APP),
            Style::default().fg(t.title_fg),
        ),
        Span::styled(
            "tui-jot ",
            Style::default()
                .fg(t.title_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(t.bg3)),
        Span::styled(
            app.vault.root.display().to_string(),
            Style::default().fg(t.fg4),
        ),
    ]);

    let title_bar = Paragraph::new(title).style(Style::default().bg(t.title_bar_bg));

    frame.render_widget(title_bar, area);
}

fn render_main(frame: &mut Frame, area: Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(app.config.ui.tree_width),
            Constraint::Min(0),
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Min(5)])
        .split(main_chunks[0]);

    browser::render(frame, left_chunks[0], app);
    render_backlinks(frame, left_chunks[1], app);
    viewer::render(frame, main_chunks[1], app);
}

fn render_backlinks(frame: &mut Frame, area: Rect, app: &App) {
    backlinks::render(frame, area, app);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let help_text = match app.focus {
        Focus::Browser => "j/k: navigate  Enter: open  a: new  d: delete  t: tags  /: search  Ctrl+q: quit",
        Focus::Viewer => "j/k: scroll  h/Esc: back  i: edit  /: search  Ctrl+p: find  Ctrl+q: quit",
        Focus::Backlinks => "j/k: navigate  Enter: open  Tab: switch pane  Ctrl+q: quit",
    };

    let note_info = app
        .selected_note()
        .map(|n| {
            format!(
                "{} │ {} tags │ {} links",
                n.path.display(),
                n.tags.len(),
                n.links.len()
            )
        })
        .unwrap_or_default();

    let status = Line::from(vec![
        Span::styled(help_text, Style::default().fg(t.fg4)),
        Span::raw("  "),
        Span::styled(note_info, Style::default().fg(t.aqua)),
    ]);

    let status_bar = Paragraph::new(status).style(Style::default().bg(t.status_bar_bg));

    frame.render_widget(status_bar, area);
}

fn render_help(frame: &mut Frame, app: &App) {
    let t = &app.theme;
    let keybindings = vec![
        (
            "Navigation",
            vec![
                ("j / k", "Move down / up"),
                ("Enter", "Open note or follow link"),
                ("Tab", "Switch pane"),
                ("h / Esc", "Go back"),
            ],
        ),
        (
            "Browser",
            vec![
                ("a", "Create new note"),
                ("d", "Delete note"),
                ("t", "Filter by tag"),
            ],
        ),
        (
            "Viewer",
            vec![
                ("i", "Enter edit mode"),
                ("Ctrl+n / p", "Next / previous link"),
                ("Ctrl+d / u", "Page down / up"),
            ],
        ),
        (
            "Global",
            vec![
                ("/", "Full-text search"),
                ("Ctrl+p", "Find note"),
                ("Ctrl+e", "Open in external editor"),
                ("Ctrl+b", "Toggle backlinks panel"),
                ("Ctrl+Shift+K", "Toggle this help"),
                ("Ctrl+q", "Quit"),
            ],
        ),
    ];

    // Calculate content size
    let content_height = keybindings
        .iter()
        .map(|(_, items)| items.len() + 2) // +2 for header and blank line
        .sum::<usize>()
        + 1;
    let content_width = 42;

    let area = centered_fixed_rect(content_width, content_height as u16, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.border_overlay));

    let mut text = Vec::new();
    for (i, (section, items)) in keybindings.iter().enumerate() {
        if i > 0 {
            text.push(Line::from(""));
        }
        text.push(Line::from(Span::styled(
            *section,
            Style::default()
                .fg(t.aqua)
                .add_modifier(Modifier::BOLD),
        )));
        for (key, action) in items {
            text.push(Line::from(vec![
                Span::styled(format!("  {:<14}", key), Style::default().fg(t.yellow)),
                Span::styled(*action, Style::default().fg(t.fg1)),
            ]));
        }
    }

    let help_paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(t.fg1).bg(t.bg0));

    frame.render_widget(help_paragraph, area);
}

fn centered_fixed_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_width = width.min(r.width.saturating_sub(4));
    let popup_height = height.min(r.height.saturating_sub(2));

    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(popup_height)) / 2;

    Rect::new(x, y, popup_width, popup_height)
}

fn render_create_dialog(frame: &mut Frame, state: &CreateNoteState, app: &App) {
    let t = &app.theme;
    let area = centered_fixed_rect(50, 6, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" New Note ")
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.aqua))
        .style(Style::default().bg(t.bg0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show parent directory info
    let parent_display = state.parent_dir.display().to_string();
    let parent_text = if parent_display.is_empty() {
        "/"
    } else {
        &parent_display
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Location: ", Style::default().fg(t.fg4)),
            Span::styled(parent_text, Style::default().fg(t.fg2)),
        ]),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(t.yellow)),
            Span::styled(&state.filename, Style::default().fg(t.fg1)),
            Span::styled(
                "_",
                Style::default()
                    .fg(t.cursor_blink)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]),
        Line::from(vec![Span::styled(
            "Tip: path/ = directory, path/name = note",
            Style::default()
                .fg(t.fg4)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}

fn render_delete_dialog(frame: &mut Frame, state: &DeleteConfirmState, app: &App) {
    let t = &app.theme;
    let has_warning = state.is_dir && state.note_count > 0;
    let height = if has_warning { 7 } else if state.is_dir { 6 } else { 5 };
    let area = centered_fixed_rect(45, height, frame.area());
    frame.render_widget(Clear, area);

    let title = if state.is_dir {
        " Delete Directory "
    } else {
        " Delete Note "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.red))
        .style(Style::default().bg(t.bg0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut text = vec![
        Line::from(vec![
            Span::styled(
                if state.is_dir {
                    "Delete directory "
                } else {
                    "Delete "
                },
                Style::default().fg(t.fg1),
            ),
            Span::styled(&state.name, Style::default().fg(t.yellow)),
            Span::styled("?", Style::default().fg(t.fg1)),
        ]),
    ];

    if has_warning {
        let warning = format!(
            "Contains {} note{}!",
            state.note_count,
            if state.note_count == 1 { "" } else { "s" }
        );
        text.push(Line::from(Span::styled(
            warning,
            Style::default()
                .fg(t.red)
                .add_modifier(Modifier::BOLD),
        )));
        text.push(Line::from(Span::styled(
            "This will delete all notes inside.",
            Style::default()
                .fg(t.fg4)
                .add_modifier(Modifier::ITALIC),
        )));
    } else if state.is_dir {
        text.push(Line::from(Span::styled(
            "(empty directory)",
            Style::default()
                .fg(t.fg4)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    text.push(Line::from(vec![
        Span::styled(
            "y",
            Style::default()
                .fg(t.green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = yes    ", Style::default().fg(t.fg3)),
        Span::styled(
            "n/Esc",
            Style::default()
                .fg(t.red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" = cancel", Style::default().fg(t.fg3)),
    ]));

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, inner);
}
