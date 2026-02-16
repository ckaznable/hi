use anyhow::Result;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use rig::tool::ToolDyn;
use shared::config::{HeartbeatConfig, ModelConfig};
use shared::runtime_index;
use crate::provider::{create_agent_from_parts, ChatAgent};

fn build_heartbeat_tools() -> Vec<Box<dyn ToolDyn>> {
    vec![
        Box::new(hi_tools::ReadFileTool) as Box<dyn ToolDyn>,
        Box::new(hi_tools::WriteFileTool),
    ]
}

fn create_heartbeat_agent(config: &ModelConfig, heartbeat_config: &HeartbeatConfig, preamble: Option<&str>) -> Result<ChatAgent> {
    let small_config = config.resolve_model_ref(&heartbeat_config.model);
    let tools = build_heartbeat_tools();
    create_agent_from_parts(
        &small_config.provider,
        &small_config.model,
        &small_config.api_key,
        &small_config.api_base,
        preamble,
        tools,
    )
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

        let index = runtime_index::load();
        let preamble = index.build_context_preamble();
        let agent = Arc::new(create_heartbeat_agent(model_config, config, Some(&preamble))?);

        let interval_secs = config.interval_secs;
        let prompt = config
            .prompt
            .clone()
            .unwrap_or_else(|| "heartbeat check".to_string());

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_secs),
            );
            interval.tick().await;

            loop {
                interval.tick().await;

                let history = vec![];
                match agent
                    .chat(
                        rig::completion::message::Message::user(&prompt),
                        history,
                    )
                    .await
                {
                    Ok(response) => {
                        let _ = tx.send(format!("[heartbeat] {}", response));
                        let epoch = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let mut idx = runtime_index::load();
                        idx.last_heartbeat_epoch = Some(epoch);
                        let _ = runtime_index::save(&idx);
                    }
                    Err(_) => {}
                }
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
