pub mod session_manager;
pub mod telegram;

use std::path::PathBuf;

use anyhow::{Result, bail};
use shared::config::ModelConfig;

pub async fn run_remote(config_path: Option<PathBuf>) -> Result<()> {
    let config = match config_path {
        Some(ref p) => ModelConfig::load_from_path(p)?,
        None => ModelConfig::load()?,
    };

    let telegram_config = config
        .remote
        .as_ref()
        .and_then(|r| r.telegram.as_ref())
        .filter(|t| t.enabled);

    let telegram_config = match telegram_config {
        Some(tc) => tc.clone(),
        None => bail!(
            "Telegram remote is not enabled in config. Set remote.telegram.enabled = true and provide bot_token."
        ),
    };

    telegram::run_polling_loop(&config, &telegram_config).await
}
