use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::ScheduleTaskConfig;

fn schedules_path() -> Result<PathBuf> {
    Ok(crate::paths::data_dir()?.join("schedules.json"))
}

pub fn load(config_schedules: Option<&[ScheduleTaskConfig]>) -> Vec<ScheduleTaskConfig> {
    match load_from_file() {
        Ok(schedules) => schedules,
        Err(_) => config_schedules.unwrap_or_default().to_vec(),
    }
}

fn load_from_file() -> Result<Vec<ScheduleTaskConfig>> {
    let path = schedules_path()?;
    if !path.exists() {
        anyhow::bail!("schedules.json not found");
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let schedules: Vec<ScheduleTaskConfig> = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let valid: Vec<ScheduleTaskConfig> = schedules
        .into_iter()
        .filter(|s| {
            if s.name.is_empty() || s.cron.is_empty() || s.prompt.is_empty() {
                warn!(
                    name = s.name,
                    "Skipping schedule with missing required fields (name, cron, or prompt)"
                );
                return false;
            }
            true
        })
        .collect();

    info!(count = valid.len(), path = %path.display(), "Loaded schedules from file");
    Ok(valid)
}

pub fn save(schedules: &[ScheduleTaskConfig]) -> Result<()> {
    let path = schedules_path()?;
    let content =
        serde_json::to_string_pretty(schedules).context("Failed to serialize schedules")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    info!(count = schedules.len(), path = %path.display(), "Saved schedules to file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduleTaskConfig;

    fn make_schedule(name: &str, cron: &str, prompt: &str) -> ScheduleTaskConfig {
        ScheduleTaskConfig {
            name: name.to_string(),
            cron: cron.to_string(),
            model: None,
            prompt: prompt.to_string(),
            enabled: false,
        }
    }

    #[test]
    fn test_load_fallback_to_config_schedules() {
        let config_schedules = vec![make_schedule("daily", "0 0 * * *", "summarize")];
        let result = load(Some(&config_schedules));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "daily");
    }

    #[test]
    fn test_load_fallback_to_empty_when_no_config() {
        let result = load(None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_save_and_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        let schedules = vec![
            make_schedule("daily", "0 0 * * *", "summarize"),
            make_schedule("hourly", "0 * * * *", "check status"),
        ];

        let content = serde_json::to_string_pretty(&schedules).unwrap();
        std::fs::write(&path, content).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "daily");
        assert_eq!(loaded[1].name, "hourly");
    }

    #[test]
    fn test_filter_invalid_schedules() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");

        let json = r#"[
            {"name": "valid", "cron": "0 0 * * *", "prompt": "do something"},
            {"name": "", "cron": "0 0 * * *", "prompt": "missing name"},
            {"name": "no-cron", "cron": "", "prompt": "missing cron"},
            {"name": "no-prompt", "cron": "0 0 * * *", "prompt": ""}
        ]"#;
        std::fs::write(&path, json).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let valid: Vec<_> = loaded
            .into_iter()
            .filter(|s| !s.name.is_empty() && !s.cron.is_empty() && !s.prompt.is_empty())
            .collect();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].name, "valid");
    }

    #[test]
    fn test_save_creates_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");

        let schedules = vec![make_schedule("test", "0 0 * * *", "test prompt")];
        let content = serde_json::to_string_pretty(&schedules).unwrap();
        std::fs::write(&path, &content).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "test");
        assert_eq!(loaded[0].prompt, "test prompt");
    }

    #[test]
    fn test_roundtrip_save_load_preserves_all_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");

        let schedules = vec![
            ScheduleTaskConfig {
                name: "with-model".to_string(),
                cron: "*/5 * * * *".to_string(),
                model: Some(crate::config::ModelRef::Named("small".to_string())),
                prompt: "check status".to_string(),
                enabled: false,
            },
            make_schedule("no-model", "0 0 * * *", "daily task"),
        ];

        let content = serde_json::to_string_pretty(&schedules).unwrap();
        std::fs::write(&path, &content).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "with-model");
        assert_eq!(
            loaded[0].model,
            Some(crate::config::ModelRef::Named("small".to_string()))
        );
        assert_eq!(loaded[1].name, "no-model");
        assert_eq!(loaded[1].model, None);
    }

    #[test]
    fn test_save_load_with_enabled_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");

        let schedules = vec![
            ScheduleTaskConfig {
                name: "enabled-schedule".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "daily task".to_string(),
                enabled: true,
            },
            ScheduleTaskConfig {
                name: "disabled-schedule".to_string(),
                cron: "0 12 * * *".to_string(),
                model: None,
                prompt: "noon task".to_string(),
                enabled: false,
            },
        ];

        let content = serde_json::to_string_pretty(&schedules).unwrap();
        std::fs::write(&path, &content).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "enabled-schedule");
        assert!(loaded[0].enabled);
        assert_eq!(loaded[1].name, "disabled-schedule");
        assert!(!loaded[1].enabled);
    }

    #[test]
    fn test_default_enabled_is_false() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");

        // Write JSON without enabled field
        let json = r#"[{
            "name": "test",
            "cron": "0 0 * * *",
            "prompt": "test prompt"
        }]"#;
        std::fs::write(&path, json).unwrap();

        let loaded: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "test");
        assert!(!loaded[0].enabled, "enabled should default to false");
    }
}
