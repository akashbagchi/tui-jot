use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;

use super::{backlinks, browser, viewer};

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
        render_help(frame);
    }
}

fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled(" tui-jot ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled(
            app.vault.root.display().to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let title_bar = Paragraph::new(title)
        .style(Style::default().bg(Color::Black));

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
        .constraints([
            Constraint::Percentage(70),
            Constraint::Min(5),
        ])
        .split(main_chunks[0]);

    browser::render(frame, left_chunks[0], app);
    render_backlinks(frame, left_chunks[1], app);
    viewer::render(frame, main_chunks[1], app);
}

fn render_backlinks(frame: &mut Frame, area: Rect, app: &App) {
    backlinks::render(frame, area, app);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.focus {
        Focus::Browser => "j/k: navigate  Enter: open  i: edit  Tab: switch pane  q: quit",
        Focus::Viewer => "j/k: scroll  h/Esc: back  i: edit  Tab: switch pane  q: quit",
        Focus::Backlinks => "j/k: navigate  Enter: open  Tab: switch pane  q: quit"
    };

    let note_info = app.selected_note()
        .map(|n| format!("{} │ {} tags │ {} links",
            n.path.display(),
            n.tags.len(),
            n.links.len()
        ))
        .unwrap_or_default();

    let status = Line::from(vec![
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(note_info, Style::default().fg(Color::Cyan)),
    ]);

    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(status_bar, area);
}

fn render_help(frame: &mut Frame) {
    let keybindings = vec![
        ("Navigation", vec![
            ("j / k", "Move down / up"),
            ("Enter", "Open note or follow link"),
            ("Tab", "Switch pane"),
            ("h / Esc", "Go back"),
        ]),
        ("Viewer", vec![
            ("i", "Enter edit mode"),
            ("Ctrl+n / p", "Next / previous link"),
            ("Ctrl+d / u", "Page down / up"),
        ]),
        ("Global", vec![
            ("Ctrl+e", "Open in external editor"),
            ("Ctrl+b", "Toggle backlinks panel"),
            ("Ctrl+Shift+K", "Toggle this help"),
            ("q", "Quit"),
        ]),
    ];

    // Calculate content size
    let content_height = keybindings.iter()
        .map(|(_, items)| items.len() + 2) // +2 for header and blank line
        .sum::<usize>() + 1;
    let content_width = 42;

    let area = centered_fixed_rect(content_width, content_height as u16, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keybindings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut text = Vec::new();
    for (i, (section, items)) in keybindings.iter().enumerate() {
        if i > 0 {
            text.push(Line::from(""));
        }
        text.push(Line::from(Span::styled(
            *section,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        for (key, action) in items {
            text.push(Line::from(vec![
                Span::styled(format!("  {:<14}", key), Style::default().fg(Color::Yellow)),
                Span::raw(*action),
            ]));
        }
    }

    let help_paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(help_paragraph, area);
}

fn centered_fixed_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_width = width.min(r.width.saturating_sub(4));
    let popup_height = height.min(r.height.saturating_sub(2));

    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(popup_height)) / 2;

    Rect::new(x, y, popup_width, popup_height)
}
