use std::collections::HashSet;
use std::ops::Range;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Note {
    pub path: PathBuf,
    pub title: String,
    pub content: String,
    pub tags: HashSet<String>,
    pub links: Vec<Link>,
    pub modified: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub target: String,
    pub display: Option<String>,
    pub span: Range<usize>,
}

impl Note {
    pub fn from_file(path: PathBuf, content: String, modified: SystemTime) -> Self {
        let title = Self::extract_title(&path, &content);
        let tags = Self::extract_tags(&content);
        let links = Self::extract_links(&content);

        Self {
            path,
            title,
            content,
            tags,
            links,
            modified,
        }
    }

    fn extract_title(path: &PathBuf, content: &str) -> String {
        // Try to find first H1 heading
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") {
                return trimmed[2..].trim().to_string();
            }
        }

        // Fall back to filename without extension
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    fn extract_tags(content: &str) -> HashSet<String> {
        let mut tags = HashSet::new();
        let mut chars = content.chars().peekable();
        let mut i = 0;

        while let Some(c) = chars.next() {
            if c == '#' {
                // Check if this is a tag (not a heading)
                // Must be preceded by whitespace or start of line
                let prev_is_valid = i == 0 || {
                    let prev_char = content[..i].chars().last();
                    prev_char.map(|c| c.is_whitespace()).unwrap_or(true)
                };

                if prev_is_valid {
                    // Collect tag characters
                    let mut tag = String::new();
                    while let Some(&next) = chars.peek() {
                        if next.is_alphanumeric() || next == '-' || next == '_' || next == '/' {
                            tag.push(next);
                            chars.next();
                            i += next.len_utf8();
                        } else {
                            break;
                        }
                    }

                    if !tag.is_empty() {
                        tags.insert(tag.to_lowercase());
                    }
                }
            }
            i += c.len_utf8();
        }

        tags
    }

    fn extract_links(content: &str) -> Vec<Link> {
        let mut links = Vec::new();
        let mut i = 0;
        let bytes = content.as_bytes();

        while i < bytes.len() {
            // Look for [[
            if i + 1 < bytes.len() && bytes[i] == b'[' && bytes[i + 1] == b'[' {
                let start = i;
                i += 2;

                // Find closing ]]
                let mut target = String::new();
                let mut display = None;
                let mut found_pipe = false;

                while i + 1 < bytes.len() && !(bytes[i] == b']' && bytes[i + 1] == b']') {
                    let c = bytes[i] as char;
                    if c == '|' && !found_pipe {
                        found_pipe = true;
                        display = Some(String::new());
                    } else if found_pipe {
                        if let Some(ref mut d) = display {
                            d.push(c);
                        }
                    } else {
                        target.push(c);
                    }
                    i += 1;
                }

                if i + 1 < bytes.len() && bytes[i] == b']' && bytes[i + 1] == b']' {
                    let end = i + 2;
                    if !target.is_empty() {
                        links.push(Link {
                            target: target.trim().to_string(),
                            display: display.map(|d| d.trim().to_string()),
                            span: start..end,
                        });
                    }
                    i = end;
                    continue;
                }
            }
            i += 1;
        }

        links
    }
}
