use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::ui::theme::{self, Theme};

pub struct TagFilterState {
    pub tags: Vec<String>,
    pub selected: usize,
    list_state: ListState,
}

impl TagFilterState {
    pub fn new(tags: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            tags,
            selected: 0,
            list_state,
        }
    }

    pub fn move_down(&mut self) {
        // +1 for "Clear filter" option at top
        let count = self.tags.len() + 1;
        if self.selected < count - 1 {
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

    /// Returns the selected tag, or None if "Clear filter" is selected (index 0).
    pub fn selected_tag(&self) -> Option<&str> {
        if self.selected == 0 {
            None
        } else {
            self.tags.get(self.selected - 1).map(|s| s.as_str())
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &TagFilterState, t: &Theme) {
    let popup_width = 40u16.min(area.width.saturating_sub(4));
    let popup_height = (state.tags.len() as u16 + 4).min(area.height.saturating_sub(4));

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(format!(" {}Filter by Tag ", theme::ICON_TAG))
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.tag_filter_border))
        .style(Style::default().bg(t.bg0));

    let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(Span::styled(
        "  (clear filter)",
        Style::default()
            .fg(t.fg4)
            .add_modifier(Modifier::ITALIC),
    )))];

    for tag in &state.tags {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("  {}", theme::ICON_TAG), Style::default().fg(t.fg4)),
            Span::styled(tag, Style::default().fg(t.tag_fg)),
        ])));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(t.selection_style());

    let mut list_state = state.list_state.clone();
    frame.render_stateful_widget(list, popup_area, &mut list_state);
}
