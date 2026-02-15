use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::skills::Skill;
use hi_tools::SkillSummary;

pub struct ContextManager {
    injected: bool,
    last_hash: u64,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            injected: false,
            last_hash: 0,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.injected = false;
        self.last_hash = 0;
    }

    pub fn build_context_message(
        &mut self,
        preamble: Option<&str>,
        tool_descriptions: &[String],
        skills: &[Skill],
    ) -> Option<String> {
        let current_hash = self.compute_hash(preamble, tool_descriptions, skills);

        if self.injected && current_hash == self.last_hash {
            return None;
        }

        let is_update = self.injected;
        self.injected = true;
        self.last_hash = current_hash;

        if is_update {
            Some(self.build_delta_message(preamble, tool_descriptions, skills))
        } else {
            Some(self.build_full_message(preamble, tool_descriptions, skills))
        }
    }

    fn build_full_message(
        &self,
        preamble: Option<&str>,
        tool_descriptions: &[String],
        skills: &[Skill],
    ) -> String {
        let mut sections = Vec::new();

        if let Some(p) = preamble {
            if !p.is_empty() {
                sections.push(format!("[System Prompt]\n{}", p));
            }
        }

        if !tool_descriptions.is_empty() {
            let tools = tool_descriptions.join("\n");
            sections.push(format!("[Available Tools]\n{}", tools));
        }

        if !skills.is_empty() {
            let skill_list: Vec<String> = skills
                .iter()
                .map(|s| format!("- {}: {}", s.name, s.description))
                .collect();
            sections.push(format!("[Available Skills]\n{}", skill_list.join("\n")));
        }

        sections.join("\n\n")
    }

    fn build_delta_message(
        &self,
        preamble: Option<&str>,
        tool_descriptions: &[String],
        skills: &[Skill],
    ) -> String {
        let mut sections = Vec::new();

        if let Some(p) = preamble {
            if !p.is_empty() {
                sections.push(format!("[Context Update]\n[System Prompt]\n{}", p));
            }
        }

        if !tool_descriptions.is_empty() {
            let tools = tool_descriptions.join("\n");
            sections.push(format!("[Context Update]\n[Available Tools]\n{}", tools));
        }

        if !skills.is_empty() {
            let skill_list: Vec<String> = skills
                .iter()
                .map(|s| format!("- {}: {}", s.name, s.description))
                .collect();
            sections.push(format!(
                "[Context Update]\n[Available Skills]\n{}",
                skill_list.join("\n")
            ));
        }

        sections.join("\n\n")
    }

    fn compute_hash(
        &self,
        preamble: Option<&str>,
        tool_descriptions: &[String],
        skills: &[Skill],
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        preamble.hash(&mut hasher);
        tool_descriptions.hash(&mut hasher);
        for skill in skills {
            skill.name.hash(&mut hasher);
            skill.description.hash(&mut hasher);
            skill.body.hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn skill_summaries(skills: &[Skill]) -> Vec<SkillSummary> {
        skills
            .iter()
            .map(|s| SkillSummary {
                name: s.name.clone(),
                description: s.description.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_message_injects() {
        let mut cm = ContextManager::new();
        let result = cm.build_context_message(Some("Hello"), &[], &[]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("[System Prompt]"));
    }

    #[test]
    fn test_second_message_no_change_skips() {
        let mut cm = ContextManager::new();
        cm.build_context_message(Some("Hello"), &[], &[]);
        let result = cm.build_context_message(Some("Hello"), &[], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_mark_dirty_causes_reinjection() {
        let mut cm = ContextManager::new();
        cm.build_context_message(Some("Hello"), &[], &[]);
        cm.mark_dirty();
        let result = cm.build_context_message(Some("Hello"), &[], &[]);
        assert!(result.is_some());
        assert!(result.unwrap().contains("[System Prompt]"));
    }

    #[test]
    fn test_change_detected_delta() {
        let mut cm = ContextManager::new();
        cm.build_context_message(Some("Hello"), &[], &[]);
        let skills = vec![Skill {
            name: "test".to_string(),
            description: "Test skill".to_string(),
            body: "Body".to_string(),
        }];
        let result = cm.build_context_message(Some("Hello"), &[], &skills);
        assert!(result.is_some());
        assert!(result.unwrap().contains("[Context Update]"));
    }

    #[test]
    fn test_full_message_format() {
        let mut cm = ContextManager::new();
        let skills = vec![Skill {
            name: "coder".to_string(),
            description: "Coding help".to_string(),
            body: "You code".to_string(),
        }];
        let tools = vec!["bash: Execute commands".to_string()];
        let result = cm
            .build_context_message(Some("System prompt"), &tools, &skills)
            .unwrap();
        assert!(result.contains("[System Prompt]\nSystem prompt"));
        assert!(result.contains("[Available Tools]\nbash: Execute commands"));
        assert!(result.contains("[Available Skills]\n- coder: Coding help"));
    }
}
