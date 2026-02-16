use std::path::PathBuf;
use std::str::FromStr;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;

use shared::heartbeat_store::{self, TaskStatus};

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HeartbeatWriteError(String);

#[derive(Deserialize)]
pub struct HeartbeatWriteArgs {
    pub task_id: String,
    pub new_status: String,
    #[serde(default)]
    pub note: Option<String>,
}

pub struct HeartbeatWriteTool {
    heartbeat_md_path: PathBuf,
}

impl HeartbeatWriteTool {
    pub fn new(heartbeat_md_path: PathBuf) -> Self {
        Self { heartbeat_md_path }
    }
}

impl Tool for HeartbeatWriteTool {
    const NAME: &'static str = "heartbeat_write";

    type Error = HeartbeatWriteError;
    type Args = HeartbeatWriteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "heartbeat_write".to_string(),
            description: "Update the status of a heartbeat task in HEARTBEAT.md. \
                Valid transitions: pending -> in-progress, in-progress -> done, in-progress -> failed."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The task identifier to update."
                    },
                    "new_status": {
                        "type": "string",
                        "enum": ["pending", "in-progress", "done", "failed"],
                        "description": "The new status to set for the task."
                    },
                    "note": {
                        "type": "string",
                        "description": "Optional note to append to the task description."
                    }
                },
                "required": ["task_id", "new_status"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let new_status =
            TaskStatus::from_str(&args.new_status).map_err(|e| HeartbeatWriteError(e))?;

        let mut ledger = heartbeat_store::load(&self.heartbeat_md_path)
            .map_err(|e| HeartbeatWriteError(e.to_string()))?;

        let task_idx = ledger
            .tasks
            .iter()
            .position(|t| t.id == args.task_id)
            .ok_or_else(|| {
                HeartbeatWriteError(format!("Task '{}' not found in HEARTBEAT.md", args.task_id))
            })?;

        if !heartbeat_store::validate_transition(&ledger.tasks[task_idx].status, &new_status) {
            return Err(HeartbeatWriteError(format!(
                "Invalid transition: {} -> {} for task '{}'",
                ledger.tasks[task_idx].status, new_status, args.task_id
            )));
        }

        ledger.tasks[task_idx].status = new_status.clone();

        if let Some(note) = args.note {
            if !note.is_empty() {
                match &mut ledger.tasks[task_idx].description {
                    Some(desc) => {
                        desc.push('\n');
                        desc.push_str(&note);
                    }
                    None => {
                        ledger.tasks[task_idx].description = Some(note);
                    }
                }
            }
        }

        heartbeat_store::save(&self.heartbeat_md_path, &ledger)
            .map_err(|e| HeartbeatWriteError(e.to_string()))?;

        Ok(format!("Task '{}' updated to {}", args.task_id, new_status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::heartbeat_store::{HeartbeatLedger, HeartbeatTask};

    fn write_test_ledger(path: &std::path::Path, tasks: Vec<HeartbeatTask>) {
        let ledger = HeartbeatLedger {
            header: "# Heartbeat Tasks".to_string(),
            tasks,
        };
        heartbeat_store::save(path, &ledger).unwrap();
    }

    #[tokio::test]
    async fn test_transition_pending_to_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Pending,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "in-progress".to_string(),
            note: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("in-progress"));

        let ledger = heartbeat_store::load(&path).unwrap();
        assert_eq!(ledger.tasks[0].status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_transition_in_progress_to_done() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::InProgress,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "done".to_string(),
            note: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("done"));

        let ledger = heartbeat_store::load(&path).unwrap();
        assert_eq!(ledger.tasks[0].status, TaskStatus::Done);
    }

    #[tokio::test]
    async fn test_transition_in_progress_to_failed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::InProgress,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "failed".to_string(),
            note: Some("Network timeout".to_string()),
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("failed"));

        let ledger = heartbeat_store::load(&path).unwrap();
        assert_eq!(ledger.tasks[0].status, TaskStatus::Failed);
        assert_eq!(
            ledger.tasks[0].description.as_deref(),
            Some("Network timeout")
        );
    }

    #[tokio::test]
    async fn test_invalid_transition_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Pending,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "done".to_string(),
            note: None,
        };
        let err = tool.call(args).await.unwrap_err();
        assert!(err.to_string().contains("Invalid transition"));
    }

    #[tokio::test]
    async fn test_task_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Pending,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "nonexistent".to_string(),
            new_status: "in-progress".to_string(),
            note: None,
        };
        let err = tool.call(args).await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_note_appends_to_existing_description() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Pending,
                title: "Test task".to_string(),
                description: Some("Original description".to_string()),
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "in-progress".to_string(),
            note: Some("Started processing".to_string()),
        };
        tool.call(args).await.unwrap();

        let ledger = heartbeat_store::load(&path).unwrap();
        let desc = ledger.tasks[0].description.as_deref().unwrap();
        assert!(desc.contains("Original description"));
        assert!(desc.contains("Started processing"));
    }

    #[tokio::test]
    async fn test_invalid_status_string() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_test_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Pending,
                title: "Test task".to_string(),
                description: None,
            }],
        );

        let tool = HeartbeatWriteTool::new(path.clone());
        let args = HeartbeatWriteArgs {
            task_id: "t1".to_string(),
            new_status: "bogus".to_string(),
            note: None,
        };
        let err = tool.call(args).await.unwrap_err();
        assert!(err.to_string().contains("Unknown task status"));
    }
}
