use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shared::config::ScheduleTaskConfig;
use tempfile::NamedTempFile;

pub struct ScheduleStorage {
    path: PathBuf,
}

impl ScheduleStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<ScheduleTaskConfig>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Failed to read {}", self.path.display()))?;
        let schedules: Vec<ScheduleTaskConfig> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", self.path.display()))?;

        Ok(filter_valid_schedules(schedules))
    }

    pub fn save(&self, schedules: &[ScheduleTaskConfig]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let json =
            serde_json::to_string_pretty(schedules).context("Failed to serialize schedules")?;

        let mut temp = NamedTempFile::new_in(
            self.path
                .parent()
                .map(PathBuf::from)
                .as_deref()
                .unwrap_or(Path::new(".")),
        )
        .context("Failed to create temporary schedules file")?;
        temp.write_all(json.as_bytes())
            .context("Failed to write temporary schedules file")?;
        temp.flush()
            .context("Failed to flush temporary schedules file")?;
        temp.persist(&self.path)
            .map_err(|e| e.error)
            .with_context(|| format!("Failed to persist {}", self.path.display()))?;

        Ok(())
    }
}

fn filter_valid_schedules(mut schedules: Vec<ScheduleTaskConfig>) -> Vec<ScheduleTaskConfig> {
    schedules.retain(|s| {
        !s.name.trim().is_empty() && !s.cron.trim().is_empty() && !s.prompt.trim().is_empty()
    });
    schedules
}
