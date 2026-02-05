use std::collections::HashMap;
use std::path::{Path, PathBuf};

use color_eyre::Result;
use walkdir::WalkDir;

use super::Note;

#[derive(Debug)]
pub struct Vault {
    pub root: PathBuf,
    pub notes: HashMap<PathBuf, Note>,
    pub tree: Vec<TreeEntry>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

impl Vault {
    pub fn open(path: &Path) -> Result<Self> {
        let root = path.to_path_buf();

        // Ensure vault directory exists
        if !root.exists() {
            std::fs::create_dir_all(&root)?;
        }

        let mut notes = HashMap::new();
        let tree = Vec::new();

        // Load all markdown files
        for entry in WalkDir::new(&root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
                let relative = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
                let content = std::fs::read_to_string(path).unwrap_or_default();
                let modified = entry.metadata().map(|m| m.modified().ok()).ok().flatten()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                let note = Note::from_file(relative.clone(), content, modified);
                notes.insert(relative, note);
            }
        }

        let mut vault = Self { root, notes, tree };
        vault.rebuild_tree();

        Ok(vault)
    }

    pub fn rebuild_tree(&mut self) {
        let mut entries: Vec<TreeEntry> = Vec::new();

        for entry in WalkDir::new(&self.root)
            .min_depth(1)
            .sort_by(|a, b| {
                // Directories first, then alphabetical
                let a_is_dir = a.file_type().is_dir();
                let b_is_dir = b.file_type().is_dir();
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.file_name().cmp(b.file_name()),
                }
            })
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let relative = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();
            let is_dir = entry.file_type().is_dir();
            let depth = entry.depth() - 1;

            // Skip non-markdown files
            if !is_dir && path.extension().map(|e| e != "md").unwrap_or(true) {
                continue;
            }

            // Skip hidden files/directories
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            entries.push(TreeEntry {
                path: relative,
                name,
                is_dir,
                depth,
                expanded: true, // Start expanded
            });
        }

        self.tree = entries;
    }

    pub fn get_note(&self, path: &Path) -> Option<&Note> {
        self.notes.get(path)
    }

    pub fn visible_entries(&self) -> Vec<&TreeEntry> {
        let mut visible = Vec::new();
        let mut collapsed_dirs: Vec<&Path> = Vec::new();

        for entry in &self.tree {
            // Check if this entry is under a collapsed directory
            let is_hidden = collapsed_dirs.iter().any(|dir| entry.path.starts_with(dir));

            if !is_hidden {
                visible.push(entry);

                // Track collapsed directories
                if entry.is_dir && !entry.expanded {
                    collapsed_dirs.push(&entry.path);
                }
            }
        }

        visible
    }

    pub fn toggle_dir(&mut self, path: &Path) {
        if let Some(entry) = self.tree.iter_mut().find(|e| e.path == path && e.is_dir) {
            entry.expanded = !entry.expanded;
        }
    }

    pub fn get_backlinks(&self, note_path: &Path) -> Vec<&Note> {
        let mut backlinks = Vec::new();

        // Normalize the target path - remove .md extension for comparison
        let target_name = note_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Search through all notes for links to this note
        for (source_path, note) in &self.notes {
            // Skip note itself
            if source_path == note_path {
                continue;
            }

            for link in &note.links {
                let link_target = if link.target.ends_with(".md") {
                    link.target.strip_suffix(".md").unwrap_or(&link.target)
                } else {
                    &link.target
                };

                // Case-insensitive comparison
                if link_target.eq_ignore_ascii_case(target_name) {
                    backlinks.push(note);
                    break;
                }
            }
        }

        backlinks.sort_by(|a, b| a.title.cmp(&b.title));

        backlinks
    }

    pub fn link_exists(&self, target: &str) -> bool {
        let target_name = if target.ends_with(".md") {
            target.strip_suffix(".md").unwrap_or(target)
        } else {
            target
        };

        // Check all notes for a match (Case-insensitive)
        self.notes.keys().any(|path| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|name| name.eq_ignore_ascii_case(target_name))
                .unwrap_or(false)
        })
    }
}
