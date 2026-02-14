use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub path: PathBuf,
    pub title: String,
    pub connections: usize,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone)]
pub struct NodePosition {
    pub path: PathBuf,
    pub x: f64,
    pub y: f64,
}

pub struct Graph {
    pub nodes: HashMap<PathBuf, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl Graph {
    pub fn from_vault(vault: &crate::core::Vault) -> Self {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();

        // Build nodes from all notes
        for (path, note) in &vault.notes {
            nodes.insert(
                path.clone(),
                GraphNode {
                    path: path.clone(),
                    title: note.title.clone(),
                    connections: 0,
                },
            );
        }

        // Build edges from links
        for (source_path, note) in &vault.notes {
            for link in &note.links {
                if let Some(target_path) = vault.notes.keys().find(|p| {
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|name| name.eq_ignore_ascii_case(&link.target))
                        .unwrap_or(false)
                }) {
                    edges.push(GraphEdge {
                        from: source_path.clone(),
                        to: target_path.clone(),
                    });

                    if let Some(node) = nodes.get_mut(source_path) {
                        node.connections += 1;
                    }
                    if let Some(node) = nodes.get_mut(target_path) {
                        node.connections += 1;
                    }
                }
            }
        }

        Self { nodes, edges }
    }

    pub fn layout_radial(&self, center: &PathBuf, width: f64, height: f64) -> Vec<NodePosition> {
        let mut positions = Vec::new();

        // Center node at middle
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        positions.push(NodePosition {
            path: center.clone(),
            x: center_x,
            y: center_y,
        });

        // Connected nodes in a circle around center
        let connected: Vec<&PathBuf> = self.nodes.keys().filter(|p| *p != center).collect();

        let radius = width.min(height) * 0.35;
        let angle_step = if connected.is_empty() {
            0.0
        } else {
            2.0 * PI / connected.len() as f64
        };

        for (i, path) in connected.iter().enumerate() {
            let angle = i as f64 * angle_step;
            positions.push(NodePosition {
                path: (*path).clone(),
                x: center_x + radius * angle.cos(),
                y: center_y + radius * angle.sin(),
            });
        }

        positions
    }

    // Get local graph
    pub fn local_graph(&self, center: &PathBuf) -> Graph {
        let mut local_nodes = HashMap::new();
        let mut local_edges = Vec::new();

        // Add center node
        if let Some(center_node) = self.nodes.get(center) {
            local_nodes.insert(center.clone(), center_node.clone());

            for edge in &self.edges {
                if &edge.from == center || &edge.to == center {
                    local_edges.push(edge.clone());

                    if let Some(node) = self.nodes.get(&edge.from) {
                        local_nodes.insert(edge.from.clone(), node.clone());
                    }
                    if let Some(node) = self.nodes.get(&edge.to) {
                        local_nodes.insert(edge.to.clone(), node.clone());
                    }
                }
            }
        }

        Graph {
            nodes: local_nodes,
            edges: local_edges,
        }
    }
}
