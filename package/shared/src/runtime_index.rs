use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeIndex {
    #[serde(default)]
    pub memory_sections: Vec<String>,
    #[serde(default)]
    pub schedule_names: Vec<String>,
    #[serde(default)]
    pub last_heartbeat_epoch: Option<u64>,
}

impl RuntimeIndex {
    pub fn build_context_preamble(&self) -> String {
        let mut parts = Vec::new();

        if !self.memory_sections.is_empty() {
            let list = self.memory_sections.join(", ");
            parts.push(format!("Persistent memory sections: [{list}]."));
        }

        if !self.schedule_names.is_empty() {
            let list = self.schedule_names.join(", ");
            parts.push(format!("Configured schedules: [{list}]."));
        }

        if let Some(ts) = self.last_heartbeat_epoch {
            parts.push(format!("Last heartbeat at unix epoch {ts}."));
        }

        if parts.is_empty() {
            "You are a background agent with read_file and write_file tools.".to_string()
        } else {
            parts.join(" ")
        }
    }
}

fn index_path() -> Result<PathBuf> {
    let data_dir = crate::paths::data_dir()?;
    Ok(data_dir.join("runtime_index.json"))
}

pub fn load() -> RuntimeIndex {
    let path = match index_path() {
        Ok(p) => p,
        Err(_) => return RuntimeIndex::default(),
    };

    if !path.exists() {
        return RuntimeIndex::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => RuntimeIndex::default(),
    }
}

pub fn save(index: &RuntimeIndex) -> Result<()> {
    let path = index_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(index)?;
    std::fs::write(&path, json)?;
    Ok(())
}

pub fn refresh_memory_sections(memory_path: &std::path::Path) -> Vec<String> {
    if !memory_path.exists() {
        return Vec::new();
    }
    let text = match std::fs::read_to_string(memory_path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    let mut sections = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level > 0 && level <= 6 {
                let name = trimmed[level..].trim();
                if !name.is_empty() {
                    sections.push(name.to_string());
                }
            }
        }
    }
    sections
}

pub fn refresh_schedule_names(schedules: &[crate::config::ScheduleTaskConfig]) -> Vec<String> {
    schedules.iter().map(|s| s.name.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_index() {
        let index = RuntimeIndex::default();
        assert!(index.memory_sections.is_empty());
        assert!(index.schedule_names.is_empty());
        assert!(index.last_heartbeat_epoch.is_none());
    }

    #[test]
    fn test_preamble_empty() {
        let index = RuntimeIndex::default();
        let preamble = index.build_context_preamble();
        assert_eq!(
            preamble,
            "You are a background agent with read_file and write_file tools."
        );
    }

    #[test]
    fn test_preamble_with_memory() {
        let index = RuntimeIndex {
            memory_sections: vec!["Notes".to_string(), "Tasks".to_string()],
            ..Default::default()
        };
        let preamble = index.build_context_preamble();
        assert!(preamble.contains("Notes, Tasks"));
    }

    #[test]
    fn test_preamble_with_schedules() {
        let index = RuntimeIndex {
            schedule_names: vec!["daily-summary".to_string()],
            ..Default::default()
        };
        let preamble = index.build_context_preamble();
        assert!(preamble.contains("daily-summary"));
    }

    #[test]
    fn test_preamble_with_heartbeat() {
        let index = RuntimeIndex {
            last_heartbeat_epoch: Some(1700000000),
            ..Default::default()
        };
        let preamble = index.build_context_preamble();
        assert!(preamble.contains("1700000000"));
    }

    #[test]
    fn test_load_returns_default_when_no_file() {
        let index = load();
        assert!(index.memory_sections.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runtime_index.json");

        let index = RuntimeIndex {
            memory_sections: vec!["Notes".to_string()],
            schedule_names: vec!["daily".to_string()],
            last_heartbeat_epoch: Some(123456),
        };
        let json = serde_json::to_string_pretty(&index).unwrap();
        std::fs::write(&path, &json).unwrap();

        let loaded: RuntimeIndex =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.memory_sections, vec!["Notes"]);
        assert_eq!(loaded.schedule_names, vec!["daily"]);
        assert_eq!(loaded.last_heartbeat_epoch, Some(123456));
    }

    #[test]
    fn test_refresh_memory_sections_parses_headers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "# Notes\nsome text\n## Sub\nmore\n# Tasks\n").unwrap();

        let sections = refresh_memory_sections(&path);
        assert_eq!(sections, vec!["Notes", "Sub", "Tasks"]);
    }

    #[test]
    fn test_refresh_memory_sections_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.md");
        std::fs::write(&path, "").unwrap();

        let sections = refresh_memory_sections(&path);
        assert!(sections.is_empty());
    }

    #[test]
    fn test_refresh_memory_sections_no_file() {
        let path = std::path::PathBuf::from("/tmp/__nonexistent_memory_test__.md");
        let sections = refresh_memory_sections(&path);
        assert!(sections.is_empty());
    }

    #[test]
    fn test_refresh_schedule_names() {
        let schedules = vec![
            crate::config::ScheduleTaskConfig {
                name: "daily".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "test".to_string(),
                enabled: true,
            },
            crate::config::ScheduleTaskConfig {
                name: "hourly".to_string(),
                cron: "0 * * * *".to_string(),
                model: None,
                prompt: "test".to_string(),
                enabled: false,
            },
        ];
        let names = refresh_schedule_names(&schedules);
        assert_eq!(names, vec!["daily", "hourly"]);
    }
}
