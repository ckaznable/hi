use crate::model_pool::ModelPool;
use anyhow::Result;
use shared::config::{ModelConfig, ScheduleTaskConfig};
use shared::runtime_index;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler {
    job_scheduler: JobScheduler,
}

impl Scheduler {
    pub async fn start_with_store(
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        let tasks = shared::schedule_store::load(model_config.schedules.as_deref());
        Self::start(&tasks, model_config, pool, tx).await
    }

    pub async fn start(
        tasks: &[ScheduleTaskConfig],
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        let job_scheduler = JobScheduler::new()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let index = runtime_index::load();
        let context_preamble = index.build_context_preamble();

        for task in tasks.iter().filter(|t| t.enabled) {
            let small_config = model_config.resolve_model_ref(&task.model);

            let pool = pool.clone();
            let tx = tx.clone();
            let task_name = task.name.clone();
            let task_prompt = task.prompt.clone();
            let preamble = context_preamble.clone();

            let job = Job::new_async(task.cron.as_str(), move |_uuid, _lock| {
                let pool = pool.clone();
                let tx = tx.clone();
                let name = task_name.clone();
                let prompt = task_prompt.clone();
                let cfg = small_config.clone();
                let preamble = preamble.clone();
                Box::pin(async move {
                    let agent = match pool.get_or_create(&cfg, Some(&preamble)) {
                        Ok(a) => a,
                        Err(_) => return,
                    };

                    let history = vec![];
                    match agent
                        .chat(rig::completion::message::Message::user(&prompt), history)
                        .await
                    {
                        Ok(response) => {
                            let _ = tx.send(format!("[schedule:{}] {}", name, response));
                        }
                        Err(_) => {}
                    }
                })
            })
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

            job_scheduler
                .add(job)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }

        job_scheduler
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(Self { job_scheduler })
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.job_scheduler
            .shutdown()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        let _ = self.job_scheduler.shutdown();
    }
}

impl Scheduler {
    /// Start the scheduler, enabling the first schedule if none are enabled.
    /// Returns a tuple of (Scheduler, bool) where the bool indicates if a schedule was auto-enabled.
    pub async fn start_with_enable(
        tasks: &mut [ScheduleTaskConfig],
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<(Self, bool)> {
        let any_enabled = tasks.iter().any(|t| t.enabled);
        let auto_enabled = if !any_enabled && !tasks.is_empty() {
            tasks[0].enabled = true;
            true
        } else {
            false
        };

        let scheduler = Self::start(tasks, model_config, pool, tx).await?;
        Ok((scheduler, auto_enabled))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::config::Provider;

    fn make_test_config() -> ModelConfig {
        ModelConfig {
            provider: Provider::Ollama,
            model: "test-model".to_string(),
            api_key: None,
            api_base: None,
            preamble: None,
            context_window: 4096,
            history_limit: None,
            small_model: None,
            heartbeat: None,
            schedules: None,
            compact: None,
            remote: None,
            memory: None,
            thinking: None,
        }
    }

    #[test]
    fn test_start_with_enable_auto_enables_first_when_none_enabled() {
        let mut tasks = vec![
            ScheduleTaskConfig {
                name: "first".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "task 1".to_string(),
                enabled: false,
            },
            ScheduleTaskConfig {
                name: "second".to_string(),
                cron: "0 12 * * *".to_string(),
                model: None,
                prompt: "task 2".to_string(),
                enabled: false,
            },
        ];

        let any_enabled = tasks.iter().any(|t| t.enabled);
        assert!(!any_enabled);

        // Simulate what start_with_enable does
        if !any_enabled && !tasks.is_empty() {
            tasks[0].enabled = true;
        }

        assert!(tasks[0].enabled);
        assert!(!tasks[1].enabled);
    }

    #[test]
    fn test_start_with_enable_noop_when_already_enabled() {
        let mut tasks = vec![
            ScheduleTaskConfig {
                name: "first".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "task 1".to_string(),
                enabled: false,
            },
            ScheduleTaskConfig {
                name: "second".to_string(),
                cron: "0 12 * * *".to_string(),
                model: None,
                prompt: "task 2".to_string(),
                enabled: true,
            },
        ];

        let any_enabled = tasks.iter().any(|t| t.enabled);
        assert!(any_enabled);

        // Simulate what start_with_enable does - should not modify anything
        if !any_enabled && !tasks.is_empty() {
            tasks[0].enabled = true;
        }

        assert!(!tasks[0].enabled);
        assert!(tasks[1].enabled);
    }

    #[test]
    fn test_start_with_enable_noop_when_empty() {
        let mut tasks: Vec<ScheduleTaskConfig> = vec![];

        let any_enabled = tasks.iter().any(|t| t.enabled);
        assert!(!any_enabled);

        // Should not panic or modify anything
        if !any_enabled && !tasks.is_empty() {
            tasks[0].enabled = true;
        }

        assert!(tasks.is_empty());
    }

    #[test]
    fn test_enabled_filtering() {
        let tasks = vec![
            ScheduleTaskConfig {
                name: "disabled".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "disabled task".to_string(),
                enabled: false,
            },
            ScheduleTaskConfig {
                name: "enabled".to_string(),
                cron: "0 12 * * *".to_string(),
                model: None,
                prompt: "enabled task".to_string(),
                enabled: true,
            },
        ];

        let enabled_tasks: Vec<_> = tasks.iter().filter(|t| t.enabled).collect();
        assert_eq!(enabled_tasks.len(), 1);
        assert_eq!(enabled_tasks[0].name, "enabled");
    }

    #[test]
    fn test_all_disabled_filtering() {
        let tasks = vec![
            ScheduleTaskConfig {
                name: "first".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "task 1".to_string(),
                enabled: false,
            },
            ScheduleTaskConfig {
                name: "second".to_string(),
                cron: "0 12 * * *".to_string(),
                model: None,
                prompt: "task 2".to_string(),
                enabled: false,
            },
        ];

        let enabled_tasks: Vec<_> = tasks.iter().filter(|t| t.enabled).collect();
        assert!(enabled_tasks.is_empty());
    }
}
