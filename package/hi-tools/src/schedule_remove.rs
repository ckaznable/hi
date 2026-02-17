use std::path::PathBuf;

use anyhow::{Result, anyhow};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use shared::config::ScheduleTaskConfig;

use crate::schedule_storage::ScheduleStorage;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ScheduleRemoveError(String);

impl From<anyhow::Error> for ScheduleRemoveError {
    fn from(value: anyhow::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Deserialize)]
pub struct ScheduleRemoveArgs {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ScheduleRemoveOutput {
    pub status: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<ScheduleTaskConfig>,
}

pub struct ScheduleRemoveTool {
    storage: ScheduleStorage,
}

impl ScheduleRemoveTool {
    pub fn new(path: PathBuf) -> Self {
        Self {
            storage: ScheduleStorage::new(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_schedules(path: &PathBuf, schedules: &[ScheduleTaskConfig]) {
        let json = serde_json::to_string_pretty(schedules).unwrap();
        std::fs::write(path, json).unwrap();
    }

    #[tokio::test]
    async fn test_remove_schedule_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        write_schedules(
            &path,
            &[ScheduleTaskConfig {
                name: "daily".into(),
                cron: "0 0 * * *".into(),
                model: None,
                prompt: "ping".into(),
                enabled: true,
            }],
        );

        let tool = ScheduleRemoveTool::new(path.clone());
        let output = tool
            .call(ScheduleRemoveArgs {
                name: "daily".into(),
            })
            .await
            .unwrap();
        assert_eq!(output.status, "ok");
        let remaining: Vec<ScheduleTaskConfig> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(remaining.is_empty());
    }

    #[tokio::test]
    async fn test_remove_schedule_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        write_schedules(
            &path,
            &[ScheduleTaskConfig {
                name: "daily".into(),
                cron: "0 0 * * *".into(),
                model: None,
                prompt: "ping".into(),
                enabled: false,
            }],
        );

        let tool = ScheduleRemoveTool::new(path);
        let err = tool
            .call(ScheduleRemoveArgs {
                name: "missing".into(),
            })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_remove_schedule_empty_name() {
        let dir = tempfile::tempdir().unwrap();
        let tool = ScheduleRemoveTool::new(dir.path().join("schedules.json"));
        let err = tool
            .call(ScheduleRemoveArgs { name: "   ".into() })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }
}

impl Tool for ScheduleRemoveTool {
    const NAME: &'static str = "cron_remove";

    type Error = ScheduleRemoveError;
    type Args = ScheduleRemoveArgs;
    type Output = ScheduleRemoveOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "cron_remove".to_string(),
            description: "Remove a cron schedule by name from schedules.json".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Schedule name to remove"
                    }
                },
                "required": ["name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if args.name.trim().is_empty() {
            return Err(ScheduleRemoveError(
                "Schedule name must not be empty".to_string(),
            ));
        }

        let mut schedules = self.storage.load()?;
        let (removed_index, removed_schedule) = schedules
            .iter()
            .enumerate()
            .find(|(_, s)| s.name.eq_ignore_ascii_case(args.name.trim()))
            .map(|(idx, s)| (idx, s.clone()))
            .ok_or_else(|| anyhow!(format!("Schedule '{}' not found", args.name)))?;

        schedules.remove(removed_index);
        self.storage.save(&schedules)?;

        Ok(ScheduleRemoveOutput {
            status: "ok",
            message: format!(
                "Removed schedule '{}'. Restart required for changes to take effect.",
                removed_schedule.name
            ),
            removed: Some(removed_schedule),
        })
    }
}
