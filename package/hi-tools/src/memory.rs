use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct MemoryError(String);

#[derive(Deserialize)]
pub struct MemoryArgs {
    pub action: String,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

pub struct MemoryTool {
    memory_path: PathBuf,
}

impl MemoryTool {
    pub fn new(memory_path: PathBuf) -> Self {
        Self { memory_path }
    }
}

impl Tool for MemoryTool {
    const NAME: &'static str = "memory";

    type Error = MemoryError;
    type Args = MemoryArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "memory".to_string(),
            description: "Read and write persistent memory organized as hierarchical markdown sections. \
                Use this to store important information across conversations."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["read", "write", "list"],
                        "description": "Action to perform: 'read' to read memory, 'write' to write a section, 'list' to list all section paths"
                    },
                    "section": {
                        "type": "string",
                        "description": "Section path using '/' separator, e.g. 'Projects/hi-cli' or 'Notes'. Omit for full read or list."
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write (required for 'write' action). Will be placed under the section header."
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.action.as_str() {
            "read" => read_memory(&self.memory_path, args.section.as_deref())
                .map_err(|e| MemoryError(e.to_string())),
            "write" => {
                let section = args.section.as_deref().ok_or_else(|| {
                    MemoryError("'section' is required for write action".to_string())
                })?;
                let content = args.content.as_deref().ok_or_else(|| {
                    MemoryError("'content' is required for write action".to_string())
                })?;
                write_memory(&self.memory_path, section, content)
                    .map_err(|e| MemoryError(e.to_string()))
            }
            "list" => list_sections(&self.memory_path).map_err(|e| MemoryError(e.to_string())),
            other => Err(MemoryError(format!(
                "Unknown action '{}'. Use 'read', 'write', or 'list'.",
                other
            ))),
        }
    }
}

struct Section {
    path: String,
    level: usize,
    content: String,
}

fn parse_sections(text: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut ancestor_stack: Vec<(usize, String)> = Vec::new();
    let mut current_content = String::new();
    let mut current_path = String::new();
    let mut current_level = 0;

    for line in text.lines() {
        if let Some((level, name)) = parse_header_line(line) {
            if !current_path.is_empty() {
                sections.push(Section {
                    path: current_path.clone(),
                    level: current_level,
                    content: current_content.trim_end().to_string(),
                });
            }

            while let Some(&(l, _)) = ancestor_stack.last() {
                if l >= level {
                    ancestor_stack.pop();
                } else {
                    break;
                }
            }
            ancestor_stack.push((level, name.clone()));
            current_path = ancestor_stack.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join("/");
            current_level = level;
            current_content = String::new();
        } else if !current_content.is_empty() || !line.is_empty() {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }

    if !current_path.is_empty() {
        sections.push(Section {
            path: current_path,
            level: current_level,
            content: current_content.trim_end().to_string(),
        });
    }

    sections
}

fn parse_header_line(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &trimmed[level..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None;
    }
    let name = rest.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some((level, name))
}

fn read_memory(path: &Path, section: Option<&str>) -> Result<String, String> {
    if !path.exists() {
        return Ok("(memory is empty)".to_string());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("Failed to read memory: {e}"))?;

    match section {
        None => Ok(if text.trim().is_empty() {
            "(memory is empty)".to_string()
        } else {
            text
        }),
        Some(section_path) => {
            let sections = parse_sections(&text);
            let target = section_path.to_lowercase();
            let mut result = String::new();

            for s in &sections {
                let s_lower = s.path.to_lowercase();
                if s_lower == target || s_lower.starts_with(&format!("{}/", target)) {
                    let hashes = "#".repeat(s.level);
                    let name = s.path.split('/').last().unwrap_or(&s.path);
                    if !result.is_empty() {
                        result.push_str("\n\n");
                    }
                    result.push_str(&format!("{} {}", hashes, name));
                    if !s.content.is_empty() {
                        result.push('\n');
                        result.push_str(&s.content);
                    }
                }
            }

            if result.is_empty() {
                Ok(format!("(section '{}' not found)", section_path))
            } else {
                Ok(result)
            }
        }
    }
}

fn write_memory(path: &Path, section_path: &str, content: &str) -> Result<String, String> {
    let text = if path.exists() {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read memory: {e}"))?
    } else {
        String::new()
    };

    let parts: Vec<&str> = section_path.split('/').collect();
    let new_text = rebuild_with_section(&text, &parts, content);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create memory directory: {e}"))?;
    }
    std::fs::write(path, &new_text).map_err(|e| format!("Failed to write memory: {e}"))?;

    Ok(format!("Written to section '{}'", section_path))
}

fn rebuild_with_section(text: &str, path_parts: &[&str], content: &str) -> String {
    let sections = parse_sections(text);
    let target_path = path_parts.join("/").to_lowercase();
    let exists = sections.iter().any(|s| s.path.to_lowercase() == target_path);

    if exists {
        let mut result = String::new();
        for s in &sections {
            let s_lower = s.path.to_lowercase();
            let hashes = "#".repeat(s.level);
            let name = s.path.split('/').last().unwrap_or(&s.path);
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str(&format!("{} {}", hashes, name));
            if s_lower == target_path {
                if !content.is_empty() {
                    result.push('\n');
                    result.push_str(content);
                }
            } else if !s.content.is_empty() {
                result.push('\n');
                result.push_str(&s.content);
            }
        }
        result.push('\n');
        result
    } else {
        let mut result = if text.is_empty() {
            String::new()
        } else {
            let mut t = text.trim_end().to_string();
            t.push_str("\n\n");
            t
        };

        for (i, part) in path_parts.iter().enumerate() {
            let level = i + 1;
            let partial_path = path_parts[..=i].join("/").to_lowercase();
            let exists_partial = sections.iter().any(|s| s.path.to_lowercase() == partial_path);
            if !exists_partial {
                let hashes = "#".repeat(level);
                result.push_str(&format!("{} {}\n", hashes, part));
                if i == path_parts.len() - 1 && !content.is_empty() {
                    result.push_str(content);
                    result.push('\n');
                }
                if i < path_parts.len() - 1 {
                    result.push('\n');
                }
            }
        }
        result
    }
}

fn list_sections(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok("(memory is empty, no sections)".to_string());
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("Failed to read memory: {e}"))?;
    let sections = parse_sections(&text);

    if sections.is_empty() {
        return Ok("(no sections found)".to_string());
    }

    let mut output = String::new();
    for s in &sections {
        let indent = "  ".repeat(s.level.saturating_sub(1));
        let name = s.path.split('/').last().unwrap_or(&s.path);
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&format!("{}- {} [{}]", indent, name, s.path));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_line() {
        assert_eq!(parse_header_line("# Hello"), Some((1, "Hello".to_string())));
        assert_eq!(
            parse_header_line("## Sub Section"),
            Some((2, "Sub Section".to_string()))
        );
        assert_eq!(
            parse_header_line("### Deep"),
            Some((3, "Deep".to_string()))
        );
        assert_eq!(parse_header_line("Not a header"), None);
        assert_eq!(parse_header_line("#NoSpace"), None);
        assert_eq!(parse_header_line("# "), None);
        assert_eq!(parse_header_line("#"), None);
    }

    #[test]
    fn test_parse_sections_basic() {
        let text = "# Notes\nSome notes here\n\n## Projects\nProject list\n\n### hi-cli\nRust chat tool";
        let sections = parse_sections(text);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].path, "Notes");
        assert_eq!(sections[0].content, "Some notes here");
        assert_eq!(sections[1].path, "Notes/Projects");
        assert_eq!(sections[1].content, "Project list");
        assert_eq!(sections[2].path, "Notes/Projects/hi-cli");
        assert_eq!(sections[2].content, "Rust chat tool");
    }

    #[test]
    fn test_parse_sections_sibling_headers() {
        let text = "# A\nContent A\n\n# B\nContent B";
        let sections = parse_sections(text);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].path, "A");
        assert_eq!(sections[1].path, "B");
    }

    #[test]
    fn test_parse_sections_back_to_parent_level() {
        let text = "# A\n## A1\nDeep\n\n# B\nTop level B";
        let sections = parse_sections(text);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].path, "A");
        assert_eq!(sections[1].path, "A/A1");
        assert_eq!(sections[1].content, "Deep");
        assert_eq!(sections[2].path, "B");
        assert_eq!(sections[2].content, "Top level B");
    }

    #[test]
    fn test_read_memory_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        let result = read_memory(&path, None).unwrap();
        assert_eq!(result, "(memory is empty)");
    }

    #[test]
    fn test_read_memory_full() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nHello world").unwrap();
        let result = read_memory(&path, None).unwrap();
        assert_eq!(result, "# Notes\nHello world");
    }

    #[test]
    fn test_read_memory_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# A\nContent A\n\n# B\nContent B").unwrap();
        let result = read_memory(&path, Some("B")).unwrap();
        assert!(result.contains("Content B"));
        assert!(!result.contains("Content A"));
    }

    #[test]
    fn test_read_memory_section_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# A\nContent").unwrap();
        let result = read_memory(&path, Some("Z")).unwrap();
        assert!(result.contains("not found"));
    }

    #[test]
    fn test_read_memory_section_includes_children() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Projects\nOverview\n\n## hi-cli\nRust tool").unwrap();
        let result = read_memory(&path, Some("Projects")).unwrap();
        assert!(result.contains("Overview"));
        assert!(result.contains("hi-cli"));
    }

    #[test]
    fn test_write_memory_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        let result = write_memory(&path, "Notes", "Hello world").unwrap();
        assert!(result.contains("Written"));

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Notes"));
        assert!(content.contains("Hello world"));
    }

    #[test]
    fn test_write_memory_nested_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        write_memory(&path, "Projects/hi-cli", "A Rust chat tool").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Projects"));
        assert!(content.contains("## hi-cli"));
        assert!(content.contains("A Rust chat tool"));
    }

    #[test]
    fn test_write_memory_update_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nOld content\n").unwrap();
        write_memory(&path, "Notes", "New content").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("New content"));
        assert!(!content.contains("Old content"));
    }

    #[test]
    fn test_write_memory_preserves_other_sections() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# A\nContent A\n\n# B\nContent B\n").unwrap();
        write_memory(&path, "A", "Updated A").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Updated A"));
        assert!(content.contains("Content B"));
    }

    #[test]
    fn test_list_sections_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        let result = list_sections(&path).unwrap();
        assert!(result.contains("empty"));
    }

    #[test]
    fn test_list_sections_hierarchical() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nHi\n\n## Sub\nThere\n\n# Projects\nList").unwrap();
        let result = list_sections(&path).unwrap();
        assert!(result.contains("Notes [Notes]"));
        assert!(result.contains("Sub [Notes/Sub]"));
        assert!(result.contains("Projects [Projects]"));
    }

    #[test]
    fn test_write_memory_case_insensitive_match() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nOld\n").unwrap();
        write_memory(&path, "notes", "New").unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("New"));
        assert!(!content.contains("Old"));
    }

    #[test]
    fn test_read_memory_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nContent here").unwrap();
        let result = read_memory(&path, Some("notes")).unwrap();
        assert!(result.contains("Content here"));
    }

    #[test]
    fn test_parse_sections_empty() {
        let sections = parse_sections("");
        assert!(sections.is_empty());
    }

    #[test]
    fn test_parse_sections_no_headers() {
        let sections = parse_sections("Just plain text\nwithout headers");
        assert!(sections.is_empty());
    }
}
