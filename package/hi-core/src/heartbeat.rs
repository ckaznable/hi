use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use shared::config::{HeartbeatConfig, ModelConfig};
use crate::model_pool::ModelPool;

pub struct HeartbeatSystem {
    handle: Option<JoinHandle<()>>,
}

impl HeartbeatSystem {
    pub fn start(
        config: &HeartbeatConfig,
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        if !config.enabled {
            return Ok(Self { handle: None });
        }

        let small_config = model_config
            .resolve_model_ref(&config.model);

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

                let agent = match pool.get_or_create(&small_config, None) {
                    Ok(a) => a,
                    Err(_) => continue,
                };

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
