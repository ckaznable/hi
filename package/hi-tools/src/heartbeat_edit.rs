use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct HeartbeatEditError(String);

impl From<anyhow::Error> for HeartbeatEditError {
    fn from(value: anyhow::Error) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeartbeatEditMode {
    Replace,
}

impl Default for HeartbeatEditMode {
    fn default() -> Self {
        HeartbeatEditMode::Replace
    }
}

#[derive(Deserialize)]
pub struct HeartbeatEditArgs {
    pub content: String,
    #[serde(default)]
    pub mode: HeartbeatEditMode,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatEditOutput {
    pub status: &'static str,
    pub message: String,
    pub task_count: usize,
}

pub struct HeartbeatEditTool {
    heartbeat_path: PathBuf,
}

impl HeartbeatEditTool {
    pub fn new(heartbeat_path: PathBuf) -> Self {
        Self { heartbeat_path }
    }

    fn validate_content(&self, args: &HeartbeatEditArgs) -> Result<()> {
        if args.content.trim().is_empty() {
            anyhow::bail!("Heartbeat content must not be empty");
        }
        if !args.content.contains("# Heartbeat") {
            anyhow::bail!("Heartbeat content must include a '# Heartbeat...' header");
        }
        match args.mode {
            HeartbeatEditMode::Replace => Ok(()),
        }
    }

    fn write_content(&self, content: &str) -> Result<()> {
        if let Some(parent) = self.heartbeat_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let base_dir = self
            .heartbeat_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let mut temp = NamedTempFile::new_in(&base_dir)
            .context("Failed to create temporary heartbeat file")?;
        temp.write_all(content.as_bytes())
            .context("Failed to write temporary heartbeat file")?;
        temp.flush()
            .context("Failed to flush temporary heartbeat file")?;
        temp.persist(&self.heartbeat_path)
            .map_err(|e| e.error)
            .with_context(|| format!("Failed to persist {}", self.heartbeat_path.display()))?;
        Ok(())
    }
}

impl Tool for HeartbeatEditTool {
    const NAME: &'static str = "heartbeat_edit";

    type Error = HeartbeatEditError;
    type Args = HeartbeatEditArgs;
    type Output = HeartbeatEditOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "heartbeat_edit".to_string(),
            description: "Replace the managed HEARTBEAT.md content with validated markdown"
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Full markdown content for HEARTBEAT.md. Must include '# Heartbeat Tasks' header."
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["replace"],
                        "description": "Edit mode (only 'replace' is supported currently)"
                    }
                },
                "required": ["content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.validate_content(&args)?;

        let content = ensure_trailing_newline(args.content.trim_end());
        let ledger = shared::heartbeat_store::parse(&content);

        self.write_content(&content)?;

        Ok(HeartbeatEditOutput {
            status: "ok",
            message: format!(
                "Heartbeat content replaced ({} task entries)",
                ledger.tasks.len()
            ),
            task_count: ledger.tasks.len(),
        })
    }
}

fn ensure_trailing_newline(input: &str) -> String {
    if input.ends_with('\n') {
        input.to_string()
    } else {
        format!("{}\n", input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replace_heartbeat_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        let tool = HeartbeatEditTool::new(path.clone());
        let content = "# Heartbeat Tasks\n\n- [pending] task-1: Check logs\n";
        let output = tool
            .call(HeartbeatEditArgs {
                content: content.to_string(),
                mode: HeartbeatEditMode::Replace,
            })
            .await
            .unwrap();
        assert_eq!(output.status, "ok");
        let written = std::fs::read_to_string(path).unwrap();
        assert!(written.contains("task-1"));
    }

    #[tokio::test]
    async fn test_replace_heartbeat_missing_header() {
        let dir = tempfile::tempdir().unwrap();
        let tool = HeartbeatEditTool::new(dir.path().join("HEARTBEAT.md"));
        let err = tool
            .call(HeartbeatEditArgs {
                content: "No header".to_string(),
                mode: HeartbeatEditMode::Replace,
            })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("header"));
    }

    #[tokio::test]
    async fn test_replace_heartbeat_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let tool = HeartbeatEditTool::new(dir.path().join("HEARTBEAT.md"));
        let err = tool
            .call(HeartbeatEditArgs {
                content: "   ".to_string(),
                mode: HeartbeatEditMode::Replace,
            })
            .await
            .unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }
}
