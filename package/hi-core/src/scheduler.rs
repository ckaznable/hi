use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_cron_scheduler::{Job, JobScheduler};
use shared::config::{ModelConfig, ScheduleTaskConfig};
use shared::runtime_index;
use crate::model_pool::ModelPool;

pub struct Scheduler {
    job_scheduler: JobScheduler,
}

impl Scheduler {
    pub async fn start_with_store(
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        let tasks = shared::schedule_store::load(
            model_config.schedules.as_deref(),
        );
        Self::start(&tasks, model_config, pool, tx).await
    }

    pub async fn start(
        tasks: &[ScheduleTaskConfig],
        model_config: &ModelConfig,
        pool: Arc<ModelPool>,
        tx: mpsc::UnboundedSender<String>,
    ) -> Result<Self> {
        let job_scheduler = JobScheduler::new().await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let index = runtime_index::load();
        let context_preamble = index.build_context_preamble();

        for task in tasks {
            let small_config = model_config
                .resolve_model_ref(&task.model);

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
                        .chat(
                            rig::completion::message::Message::user(&prompt),
                            history,
                        )
                        .await
                    {
                        Ok(response) => {
                            let _ = tx.send(format!("[schedule:{}] {}", name, response));
                        }
                        Err(_) => {}
                    }
                })
            }).map_err(|e| anyhow::anyhow!("{:?}", e))?;

            job_scheduler.add(job).await
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }

        job_scheduler.start().await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        Ok(Self { job_scheduler })
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.job_scheduler.shutdown().await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        Ok(())
    }
}
