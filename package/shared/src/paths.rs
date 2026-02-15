use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

const QUALIFIER: &str = "";
const ORGANIZATION: &str = "";
const APPLICATION: &str = "hi";

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .context("Failed to determine project directories")
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create config dir: {}", path.display()))?;
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create data dir: {}", path.display()))?;
    Ok(path)
}
