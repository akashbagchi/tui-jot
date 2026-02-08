use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::Vault;

/// Pre-computed index of tags and links across all notes in the vault.
pub struct Index {
    /// tag (lowercase) → set of note paths that have this tag
    pub tags: HashMap<String, HashSet<PathBuf>>,
    /// link target (lowercase, no .md) → set of note paths that link to it
    pub forward_links: HashMap<String, HashSet<PathBuf>>,
}

impl Index {
    pub fn build(vault: &Vault) -> Self {
        let mut tags: HashMap<String, HashSet<PathBuf>> = HashMap::new();
        let mut forward_links: HashMap<String, HashSet<PathBuf>> = HashMap::new();

        for (path, note) in &vault.notes {
            // Index tags
            for tag in &note.tags {
                tags.entry(tag.clone()).or_default().insert(path.clone());
            }

            // Index forward links (normalized: lowercase, no .md extension)
            for link in &note.links {
                let target = link.target.to_lowercase();
                let target = if target.ends_with(".md") {
                    target.strip_suffix(".md").unwrap().to_string()
                } else {
                    target
                };

                forward_links
                    .entry(target)
                    .or_default()
                    .insert(path.clone());
            }
        }

        Self {
            tags,
            forward_links,
        }
    }

    /// Returns all note paths that have the given tag.
    pub fn notes_with_tag(&self, tag: &str) -> Option<&HashSet<PathBuf>> {
        self.tags.get(&tag.to_lowercase())
    }

    /// Returns all note paths that link to the given note path.
    pub fn get_backlinks(&self, note_path: &Path) -> Vec<PathBuf> {
        // Normalize: strip .md, lowercase
        let target = if note_path.extension().is_some_and(|e| e == "md") {
            note_path.with_extension("")
        } else {
            note_path.to_path_buf()
        };

        let mut backlinks = Vec::new();

        // Check by full path (lowercase)
        let target_str = target.to_string_lossy().to_lowercase();
        if let Some(sources) = self.forward_links.get(&target_str) {
            for source in sources {
                if source != note_path {
                    backlinks.push(source.clone());
                }
            }
        }

        // Also check by filename only (for links like [[note-name]] without path)
        if let Some(file_name) = target.file_name() {
            let name_str = file_name.to_string_lossy().to_lowercase();
            if name_str != target_str {
                if let Some(sources) = self.forward_links.get(&*name_str) {
                    for source in sources {
                        if source != note_path && !backlinks.contains(source) {
                            backlinks.push(source.clone());
                        }
                    }
                }
            }
        }

        backlinks.sort();
        backlinks
    }

    /// Returns a sorted list of all unique tags.
    pub fn all_tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self.tags.keys().map(|s| s.as_str()).collect();
        tags.sort();
        tags
    }
}
