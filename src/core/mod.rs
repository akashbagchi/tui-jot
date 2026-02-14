mod graph;
mod index;
mod note;
mod vault;

pub use graph::{Graph, GraphEdge, GraphNode, NodePosition};
pub use index::Index;
pub use note::Note;
pub use vault::{TreeEntry, Vault};

/// Fuzzy match: checks if all characters of `query` appear in `text` in order.
pub fn fuzzy_match(query: &str, text: &str) -> bool {
    let mut query_chars = query.chars();
    let mut current = query_chars.next();

    for c in text.chars() {
        if let Some(q) = current {
            if c == q {
                current = query_chars.next();
            }
        } else {
            return true;
        }
    }

    current.is_none()
}
