use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

use super::{browser, viewer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Browser,
    Viewer,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Browser => Focus::Viewer,
            Focus::Viewer => Focus::Browser,
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
    use ratatui::{
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem},
    };

    let border_style = Style::default().fg(Color::DarkGray);

    let block = Block::default()
        .title(" Backlinks ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = if let Some(note) = app.selected_note() {
        let backlinks = app.vault.get_backlinks(&note.path);

        if backlinks.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                        "   No backlinks",
                        Style::default().fg(Color::DarkGray),
                        )))]
        } else {
            backlinks
                .iter()
                .map(|backlink| {
                    let name = backlink.path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown");

                    ListItem::new(Line::from(vec![
                            Span::raw("  <- "),
                            Span::styled(name, Style::default().fg(Color::Yellow)),
                    ]))
                })
                .collect()
        }
    } else {
        vec![ListItem::new(Line::from(Span::styled(
                    "   No note selected",
                    Style::default().fg(Color::DarkGray),
                    )))]
    };

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.focus {
        Focus::Browser => "j/k: navigate  Enter: open  e: edit  Tab: switch pane  q: quit",
        Focus::Viewer => "j/k: scroll  h/Esc: back  e: edit  Tab: switch pane  q: quit",
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
