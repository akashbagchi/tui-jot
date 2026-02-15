use crate::core::{Graph, NodePosition};
use std::path::PathBuf;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, canvas::Canvas},
};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphMode {
    Local,
    Global,
}

pub struct GraphViewState {
    pub mode: GraphMode,
    pub selected_node: Option<PathBuf>,
    pub positions: Vec<NodePosition>,
    pub graph: Option<Graph>,
}

impl GraphViewState {
    pub fn new() -> Self {
        Self {
            mode: GraphMode::Local,
            selected_node: None,
            positions: Vec::new(),
            graph: None,
        }
    }

    pub fn update_local(
        &mut self,
        vault: &crate::core::Vault,
        center: &PathBuf,
        width: u16,
        height: u16,
    ) {
        let full_graph = Graph::from_vault(vault);
        let local = full_graph.local_graph(center);

        self.positions = local.layout_radial(center, width as f64, height as f64);
        self.selected_node = Some(center.clone());
        self.graph = Some(local);
        self.mode = GraphMode::Local;
    }

    pub fn update_global(&mut self, vault: &crate::core::Vault, width: u16, height: u16) {
        let graph = Graph::from_vault(vault);
        if let Some(first) = graph.nodes.keys().next() {
            self.positions = graph.layout_radial(first, width as f64, height as f64);
        }
        self.graph = Some(graph);
        self.mode = GraphMode::Global;
    }

    pub fn move_selection(&mut self, direction: (i32, i32)) {
        if self.positions.is_empty() {
            return;
        }

        let current_idx = self
            .selected_node
            .as_ref()
            .and_then(|p| self.positions.iter().position(|pos| &pos.path == p))
            .unwrap_or(0);

        let next_idx = match direction {
            (1, 0) | (0, 1) => (current_idx + 1) % self.positions.len(),
            (-1, 0) | (0, -1) => {
                if current_idx == 0 {
                    self.positions.len() - 1
                } else {
                    current_idx - 1
                }
            }
            _ => current_idx,
        };
        self.selected_node = Some(self.positions[next_idx].path.clone());
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &GraphViewState, t: &crate::ui::theme::Theme) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " Graph View - {} ",
            match state.mode {
                GraphMode::Local => "Local",
                GraphMode::Global => "Global",
            }
        ))
        .borders(Borders::ALL)
        .border_type(theme::border_type())
        .border_style(Style::default().fg(t.border_overlay));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(ref graph) = state.graph {
        // Clone what we need for the paint closure
        let edges = graph.edges.clone();
        let positions = state.positions.clone();
        let selected = state.selected_node.clone();
        let node_color = t.aqua;
        let selected_color = t.yellow;

        let canvas = Canvas::default()
            .x_bounds([0.0, inner.width as f64])
            .y_bounds([0.0, inner.height as f64])
            .paint(move |ctx| {
                // Draw edges
                for edge in &edges {
                    if let (Some(from_pos), Some(to_pos)) = (
                        positions.iter().find(|p| p.path == edge.from),
                        positions.iter().find(|p| p.path == edge.to),
                    ) {
                        ctx.draw(&ratatui::widgets::canvas::Line {
                            x1: from_pos.x,
                            y1: from_pos.y,
                            x2: to_pos.x,
                            y2: to_pos.y,
                            color: Color::DarkGray,
                        });
                    }
                }

                // Draw nodes
                for pos in &positions {
                    let is_selected = selected.as_ref() == Some(&pos.path);
                    let color = if is_selected {
                        selected_color
                    } else {
                        node_color
                    };

                    ctx.draw(&ratatui::widgets::canvas::Circle {
                        x: pos.x,
                        y: pos.y,
                        radius: 1.5,
                        color,
                    });
                }
            });

        frame.render_widget(canvas, inner);
        render_node_labels(frame, inner, state, t);
    } else {
        let text = Line::from(Span::styled(
            " No graph data available",
            Style::default().fg(t.fg4),
        ));
        frame.render_widget(Paragraph::new(text), inner);
    }

    render_status(frame, area, t);
}

fn render_node_labels(
    frame: &mut Frame,
    area: Rect,
    state: &GraphViewState,
    t: &crate::ui::theme::Theme,
) {
    let w = area.width as f64;
    let h = area.height as f64;

    for pos in &state.positions {
        if let Some(node) = state.graph.as_ref().and_then(|g| g.nodes.get(&pos.path)) {
            let is_selected = state.selected_node.as_ref() == Some(&pos.path);

            // Map graph coordinates to terminal coordinates
            // Canvas y-axis is inverted: 0 = bottom, max = top
            let term_x = area.x + ((pos.x / w) * w) as u16;
            let term_y = area.y + ((h - pos.y) / h * h) as u16;

            if term_x >= area.x + area.width || term_y >= area.y + area.height {
                continue;
            }

            let style = if is_selected {
                Style::default().fg(t.yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.fg1)
            };

            // Place label just below the node
            let label_y = (term_y + 1).min(area.y + area.height - 1);
            let label_width = (node.title.len() as u16)
                .min(20)
                .min(area.x + area.width - term_x);

            let label_area = Rect {
                x: term_x,
                y: label_y,
                width: label_width,
                height: 1,
            };

            let text = Line::from(Span::styled(&node.title, style));
            frame.render_widget(Paragraph::new(text), label_area);
        }
    }
}

fn render_status(frame: &mut Frame, area: Rect, t: &crate::ui::theme::Theme) {
    let status_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    let help = " [hjkl] navigate  [Tab] toggle local/global  [Enter] open  [Esc] close";
    let text = Line::from(Span::styled(help, Style::default().fg(t.fg4)));
    frame.render_widget(Paragraph::new(text), status_area);
}
