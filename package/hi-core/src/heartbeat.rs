use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use rig::tool::ToolDyn;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use shared::config::{HeartbeatConfig, ModelConfig};
use shared::heartbeat_store::{self, TaskStatus};
use shared::runtime_index;

use crate::provider::{ChatAgent, create_agent_from_parts};

fn build_heartbeat_tools(heartbeat_md_path: PathBuf) -> Vec<Box<dyn ToolDyn>> {
    vec![
        Box::new(hi_tools::ReadFileTool) as Box<dyn ToolDyn>,
        Box::new(hi_tools::WriteFileTool),
        Box::new(hi_tools::HeartbeatWriteTool::new(heartbeat_md_path.clone())),
        Box::new(hi_tools::HeartbeatEditTool::new(heartbeat_md_path)),
    ]
}

fn create_heartbeat_agent(
    config: &ModelConfig,
    heartbeat_config: &HeartbeatConfig,
    preamble: Option<&str>,
    heartbeat_md_path: PathBuf,
) -> Result<ChatAgent> {
    let small_config = config.resolve_model_ref(&heartbeat_config.model);
    let tools = build_heartbeat_tools(heartbeat_md_path);
    create_agent_from_parts(
        &small_config.provider,
        &small_config.model,
        &small_config.api_key,
        &small_config.api_base,
        preamble,
        tools,
        None,
    )
}

fn heartbeat_md_path() -> PathBuf {
    shared::paths::data_dir()
        .map(|d| d.join("HEARTBEAT.md"))
        .unwrap_or_else(|_| PathBuf::from("HEARTBEAT.md"))
}

pub struct HeartbeatSystem {
    handle: Option<JoinHandle<()>>,
}

impl HeartbeatSystem {
    pub fn start(
        config: &HeartbeatConfig,
        model_config: &ModelConfig,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        if !config.enabled {
            return Ok(Self { handle: None });
        }

        let md_path = heartbeat_md_path();
        let index = runtime_index::load();
        let preamble = index.build_context_preamble();
        let agent = Arc::new(create_heartbeat_agent(
            model_config,
            config,
            Some(&preamble),
            md_path.clone(),
        )?);

        let interval_secs = config.interval_secs;
        let fallback_prompt = config
            .prompt
            .clone()
            .unwrap_or_else(|| "heartbeat check".to_string());

        let handle = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            interval.tick().await;

            loop {
                interval.tick().await;
                run_heartbeat_tick(&agent, &tx, &md_path, &fallback_prompt).await;
            }
        });

        Ok(Self {
            handle: Some(handle),
        })
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for HeartbeatSystem {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_heartbeat_tick(
    agent: &ChatAgent,
    tx: &mpsc::UnboundedSender<String>,
    md_path: &PathBuf,
    fallback_prompt: &str,
) {
    let prompt = match build_task_prompt(md_path) {
        Some(p) => p,
        None => fallback_prompt.to_string(),
    };

    let history = vec![];
    match agent
        .chat(rig::completion::message::Message::user(&prompt), history)
        .await
    {
        Ok(response) => {
            let _ = tx.send(format!("[heartbeat] {}", response));
        }
        Err(e) => {
            eprintln!("[heartbeat] Agent error: {}", e);
        }
    }

    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut idx = runtime_index::load();
    idx.last_heartbeat_epoch = Some(epoch);
    let _ = runtime_index::save(&idx);
}

fn build_task_prompt(md_path: &PathBuf) -> Option<String> {
    let mut ledger = match heartbeat_store::load(md_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[heartbeat] Failed to load HEARTBEAT.md: {}", e);
            return None;
        }
    };

    let task_idx = ledger
        .tasks
        .iter()
        .position(|t| t.status == TaskStatus::Pending)?;

    ledger.tasks[task_idx].status = TaskStatus::InProgress;
    if let Err(e) = heartbeat_store::save(md_path, &ledger) {
        eprintln!("[heartbeat] Failed to persist in-progress status: {}", e);
    }

    let task = &ledger.tasks[task_idx];
    let mut prompt = format!("Execute heartbeat task '{}': {}", task.id, task.title);
    if let Some(ref desc) = task.description {
        prompt.push_str(&format!("\n\nDetails:\n{}", desc));
    }
    prompt.push_str(&format!(
        "\n\nWhen finished, use the heartbeat_write tool to mark task '{}' as 'done' (or 'failed' if unsuccessful).",
        task.id
    ));

    Some(prompt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::heartbeat_store::{HeartbeatLedger, HeartbeatTask};

    fn write_ledger(path: &std::path::Path, tasks: Vec<HeartbeatTask>) {
        let ledger = HeartbeatLedger {
            header: "# Heartbeat Tasks".to_string(),
            tasks,
        };
        heartbeat_store::save(path, &ledger).unwrap();
    }

    #[test]
    fn test_build_task_prompt_picks_first_pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_ledger(
            &path,
            vec![
                HeartbeatTask {
                    id: "done-task".to_string(),
                    status: TaskStatus::Done,
                    title: "Already done".to_string(),
                    description: None,
                },
                HeartbeatTask {
                    id: "pending-task".to_string(),
                    status: TaskStatus::Pending,
                    title: "Check logs".to_string(),
                    description: Some("Look at system logs".to_string()),
                },
                HeartbeatTask {
                    id: "pending-task-2".to_string(),
                    status: TaskStatus::Pending,
                    title: "Run backup".to_string(),
                    description: None,
                },
            ],
        );

        let prompt = build_task_prompt(&path.to_path_buf()).unwrap();
        assert!(prompt.contains("pending-task"));
        assert!(prompt.contains("Check logs"));
        assert!(prompt.contains("Look at system logs"));
        assert!(prompt.contains("heartbeat_write"));
        assert!(!prompt.contains("pending-task-2"));

        let ledger = heartbeat_store::load(&path).unwrap();
        assert_eq!(ledger.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(ledger.tasks[2].status, TaskStatus::Pending);
    }

    #[test]
    fn test_build_task_prompt_returns_none_when_no_pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_ledger(
            &path,
            vec![HeartbeatTask {
                id: "t1".to_string(),
                status: TaskStatus::Done,
                title: "Done task".to_string(),
                description: None,
            }],
        );

        let result = build_task_prompt(&path.to_path_buf());
        assert!(result.is_none());
    }

    #[test]
    fn test_build_task_prompt_returns_none_on_empty_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_ledger(&path, vec![]);

        let result = build_task_prompt(&path.to_path_buf());
        assert!(result.is_none());
    }

    #[test]
    fn test_build_task_prompt_returns_none_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");

        let result = build_task_prompt(&path.to_path_buf());
        assert!(result.is_none());
    }

    #[test]
    fn test_build_task_prompt_without_description() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        write_ledger(
            &path,
            vec![HeartbeatTask {
                id: "simple".to_string(),
                status: TaskStatus::Pending,
                title: "Simple check".to_string(),
                description: None,
            }],
        );

        let prompt = build_task_prompt(&path.to_path_buf()).unwrap();
        assert!(prompt.contains("simple"));
        assert!(prompt.contains("Simple check"));
        assert!(!prompt.contains("Details:"));
    }
}
