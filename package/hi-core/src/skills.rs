use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

pub fn load_skills(config_dir: &Path) -> Result<Vec<Skill>> {
    let skills_dir = config_dir.join("skills");
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();
    let entries = std::fs::read_dir(&skills_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let content = std::fs::read_to_string(&path)?;
        let (description, body) = parse_frontmatter(&content, &name);
        skills.push(Skill {
            name,
            description,
            body,
        });
    }

    Ok(skills)
}

fn parse_frontmatter(content: &str, fallback_name: &str) -> (String, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (fallback_name.to_string(), content.to_string());
    }

    let after_first = &trimmed[3..];
    if let Some(end_idx) = after_first.find("---") {
        let frontmatter = &after_first[..end_idx];
        let body = after_first[end_idx + 3..].trim_start().to_string();

        let description = frontmatter
            .lines()
            .find_map(|line| {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("description:") {
                    Some(rest.trim().trim_matches('"').trim_matches('\'').to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| fallback_name.to_string());

        (description, body)
    } else {
        (fallback_name.to_string(), content.to_string())
    }
}

pub fn build_preamble(base_preamble: Option<&str>, skills: &[Skill]) -> String {
    let mut parts = Vec::new();
    if let Some(p) = base_preamble {
        if !p.is_empty() {
            parts.push(p.to_string());
        }
    }
    for skill in skills {
        parts.push(skill.body.clone());
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_with_description() {
        let content = "---\ndescription: Expert coding guidance\n---\nYou are a coding expert.";
        let (desc, body) = parse_frontmatter(content, "fallback");
        assert_eq!(desc, "Expert coding guidance");
        assert_eq!(body, "You are a coding expert.");
    }

    #[test]
    fn test_parse_frontmatter_without_frontmatter() {
        let content = "You are a translator.";
        let (desc, body) = parse_frontmatter(content, "translator");
        assert_eq!(desc, "translator");
        assert_eq!(body, "You are a translator.");
    }

    #[test]
    fn test_parse_frontmatter_quoted_description() {
        let content = "---\ndescription: \"Quoted desc\"\n---\nBody text.";
        let (desc, body) = parse_frontmatter(content, "fallback");
        assert_eq!(desc, "Quoted desc");
        assert_eq!(body, "Body text.");
    }

    #[test]
    fn test_build_preamble_with_skills() {
        let skills = vec![
            Skill {
                name: "a".to_string(),
                description: "Skill A".to_string(),
                body: "You are skill A.".to_string(),
            },
            Skill {
                name: "b".to_string(),
                description: "Skill B".to_string(),
                body: "You are skill B.".to_string(),
            },
        ];
        let result = build_preamble(Some("Base prompt"), &skills);
        assert!(result.contains("Base prompt"));
        assert!(result.contains("You are skill A."));
        assert!(result.contains("You are skill B."));
    }

    #[test]
    fn test_build_preamble_no_base() {
        let skills = vec![Skill {
            name: "a".to_string(),
            description: "A".to_string(),
            body: "Skill body".to_string(),
        }];
        let result = build_preamble(None, &skills);
        assert_eq!(result, "Skill body");
    }

    #[test]
    fn test_load_skills_no_dir() {
        let dir = std::path::PathBuf::from("/tmp/nonexistent_skills_test_dir");
        let skills = load_skills(&dir).unwrap();
        assert!(skills.is_empty());
    }
}
