use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use shared::config::{ModelRef, ScheduleTaskConfig};

use crate::schedule_storage::ScheduleStorage;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ScheduleAddError(String);

impl From<anyhow::Error> for ScheduleAddError {
    fn from(value: anyhow::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Deserialize)]
pub struct ScheduleAddArgs {
    pub name: String,
    pub cron: String,
    pub prompt: String,
    #[serde(default)]
    pub model: Option<ModelRef>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleMutationOutput {
    pub status: &'static str,
    pub message: String,
    pub schedule: Option<ScheduleTaskConfig>,
}

pub struct ScheduleAddTool {
    storage: ScheduleStorage,
}

impl ScheduleAddTool {
    pub fn new(path: PathBuf) -> Self {
        Self {
            storage: ScheduleStorage::new(path),
        }
    }

    fn validate_args(args: &ScheduleAddArgs) -> Result<()> {
        if args.name.trim().is_empty() {
            return Err(anyhow!("Schedule name must not be empty"));
        }
        if args.prompt.trim().is_empty() {
            return Err(anyhow!("Prompt must not be empty"));
        }
        validate_cron_expression(&args.cron)?;
        Ok(())
    }
}

impl Tool for ScheduleAddTool {
    const NAME: &'static str = "cron_add";

    type Error = ScheduleAddError;
    type Args = ScheduleAddArgs;
    type Output = ScheduleMutationOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "cron_add".to_string(),
            description:
                "Add a cron schedule to schedules.json with validation and duplicate detection"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Unique identifier for the schedule"
                    },
                    "cron": {
                        "type": "string",
                        "description": "Five-field cron expression (min hour dom mon dow)"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Prompt that will be sent when this schedule runs"
                    },
                    "model": {
                        "description": "Optional model override. Either a string name or inline {\"provider\":...,\"model\":...,\"context_window\":...} object",
                        "oneOf": [
                            {"type": "string"},
                            {"type": "object"}
                        ]
                    }
                },
                "required": ["name", "cron", "prompt"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        Self::validate_args(&args)?;

        let mut schedules = self.storage.load()?;
        if schedules
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(&args.name))
        {
            return Err(ScheduleAddError(format!(
                "Schedule '{}' already exists; remove it first to replace",
                args.name
            )));
        }

        let auto_enable = !schedules.iter().any(|s| s.enabled);

        let new_schedule = ScheduleTaskConfig {
            name: args.name.trim().to_string(),
            cron: args.cron.trim().to_string(),
            model: args.model.clone(),
            prompt: args.prompt.trim().to_string(),
            enabled: auto_enable,
        };

        schedules.push(new_schedule.clone());
        self.storage.save(&schedules)?;

        let message = if auto_enable {
            format!(
                "Added schedule '{}' (cron: {}). Schedule auto-enabled. Restart to activate.",
                new_schedule.name, new_schedule.cron
            )
        } else {
            format!(
                "Added schedule '{}' (cron: {}). Restart required for changes to take effect.",
                new_schedule.name, new_schedule.cron
            )
        };

        Ok(ScheduleMutationOutput {
            status: "ok",
            message,
            schedule: Some(new_schedule),
        })
    }
}

fn validate_cron_expression(expr: &str) -> Result<()> {
    croner::Cron::from_str(expr.trim())
        .map_err(|e| anyhow!("Invalid cron expression '{}': {}", expr, e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(temp_dir: &tempfile::TempDir) -> ScheduleAddTool {
        let path = temp_dir.path().join("schedules.json");
        ScheduleAddTool::new(path)
    }

    fn read_schedules(path: &PathBuf) -> Vec<ScheduleTaskConfig> {
        let content = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[tokio::test]
    async fn test_add_schedule_success() {
        let dir = tempfile::tempdir().unwrap();
        let tool = make_tool(&dir);
        let args = ScheduleAddArgs {
            name: "daily".to_string(),
            cron: "0 0 * * *".to_string(),
            prompt: "Generate daily summary".to_string(),
            model: None,
        };

        let result = tool.call(args).await.unwrap();
        assert_eq!(result.status, "ok");
        let schedules = read_schedules(&dir.path().join("schedules.json"));
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].name, "daily");
    }

    #[tokio::test]
    async fn test_add_schedule_invalid_cron() {
        let dir = tempfile::tempdir().unwrap();
        let tool = make_tool(&dir);
        let args = ScheduleAddArgs {
            name: "bad".to_string(),
            cron: "invalid cron".to_string(),
            prompt: "noop".to_string(),
            model: None,
        };

        let err = tool.call(args).await.unwrap_err();
        assert!(err.to_string().contains("Invalid cron expression"));
    }

    #[tokio::test]
    async fn test_add_schedule_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[
                {"name": "daily", "cron": "0 0 * * *", "prompt": "hey"}
            ]"#,
        )
        .unwrap();

        let tool = ScheduleAddTool::new(path);
        let args = ScheduleAddArgs {
            name: "daily".to_string(),
            cron: "0 12 * * *".to_string(),
            prompt: "ping".to_string(),
            model: None,
        };

        let err = tool.call(args).await.unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }
}
