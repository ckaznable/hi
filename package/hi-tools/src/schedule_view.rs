use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ScheduleViewError(String);

#[derive(Deserialize)]
pub struct ScheduleViewArgs {
    #[serde(default)]
    pub name: Option<String>,
}

pub struct ScheduleViewTool {
    schedules_path: PathBuf,
}

impl ScheduleViewTool {
    pub fn new(schedules_path: PathBuf) -> Self {
        Self { schedules_path }
    }
}

impl Tool for ScheduleViewTool {
    const NAME: &'static str = "view_schedules";

    type Error = ScheduleViewError;
    type Args = ScheduleViewArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "view_schedules".to_string(),
            description: "View configured cron schedules. Lists all schedules or shows details for a specific one by name."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Optional schedule name to filter. Omit to list all schedules."
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let schedules = load_schedules(&self.schedules_path)
            .map_err(|e| ScheduleViewError(e.to_string()))?;

        if schedules.is_empty() {
            return Ok("No schedules configured.".to_string());
        }

        match args.name.as_deref() {
            Some(name) => {
                let lower = name.to_lowercase();
                let matched: Vec<&ScheduleEntry> = schedules
                    .iter()
                    .filter(|s| s.name.to_lowercase() == lower)
                    .collect();

                if matched.is_empty() {
                    let names: Vec<&str> = schedules.iter().map(|s| s.name.as_str()).collect();
                    Ok(format!(
                        "No schedule named '{}'. Available: {}",
                        name,
                        names.join(", ")
                    ))
                } else {
                    Ok(matched
                        .iter()
                        .map(|s| format_schedule(s))
                        .collect::<Vec<_>>()
                        .join("\n"))
                }
            }
            None => {
                let mut out = format!("{} schedule(s) configured:\n", schedules.len());
                for s in &schedules {
                    out.push('\n');
                    out.push_str(&format_schedule(s));
                }
                Ok(out)
            }
        }
    }
}

#[derive(Deserialize)]
struct ScheduleEntry {
    name: String,
    cron: String,
    #[serde(default)]
    model: Option<serde_json::Value>,
    prompt: String,
}

fn format_schedule(s: &ScheduleEntry) -> String {
    let model_str = match &s.model {
        Some(serde_json::Value::String(m)) => m.clone(),
        Some(v) => v.to_string(),
        None => "(default)".to_string(),
    };
    format!(
        "- {}\n  cron: {}\n  model: {}\n  prompt: {}",
        s.name, s.cron, model_str, s.prompt
    )
}

fn load_schedules(path: &std::path::Path) -> anyhow::Result<Vec<ScheduleEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", path.display(), e))?;

    let schedules: Vec<ScheduleEntry> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;

    Ok(schedules
        .into_iter()
        .filter(|s| !s.name.is_empty() && !s.cron.is_empty() && !s.prompt.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_schedules_nonexistent() {
        let path = std::path::PathBuf::from("/tmp/nonexistent_schedules_test.json");
        let result = load_schedules(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_schedules_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[
                {"name": "daily", "cron": "0 0 * * *", "prompt": "summarize"},
                {"name": "hourly", "cron": "0 * * * *", "model": "small", "prompt": "check"}
            ]"#,
        )
        .unwrap();

        let result = load_schedules(&path).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "daily");
        assert_eq!(result[1].name, "hourly");
        assert_eq!(result[1].model, Some(serde_json::Value::String("small".to_string())));
    }

    #[test]
    fn test_load_schedules_filters_invalid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[
                {"name": "valid", "cron": "0 0 * * *", "prompt": "do something"},
                {"name": "", "cron": "0 0 * * *", "prompt": "no name"},
                {"name": "no-cron", "cron": "", "prompt": "missing cron"}
            ]"#,
        )
        .unwrap();

        let result = load_schedules(&path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "valid");
    }

    #[test]
    fn test_format_schedule_with_model() {
        let s = ScheduleEntry {
            name: "daily".to_string(),
            cron: "0 0 * * *".to_string(),
            model: Some(serde_json::Value::String("small".to_string())),
            prompt: "summarize".to_string(),
        };
        let out = format_schedule(&s);
        assert!(out.contains("daily"));
        assert!(out.contains("0 0 * * *"));
        assert!(out.contains("small"));
        assert!(out.contains("summarize"));
    }

    #[test]
    fn test_format_schedule_without_model() {
        let s = ScheduleEntry {
            name: "test".to_string(),
            cron: "*/5 * * * *".to_string(),
            model: None,
            prompt: "check status".to_string(),
        };
        let out = format_schedule(&s);
        assert!(out.contains("(default)"));
    }

    #[tokio::test]
    async fn test_view_schedules_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        let tool = ScheduleViewTool::new(path);
        let args = ScheduleViewArgs { name: None };
        let result = tool.call(args).await.unwrap();
        assert_eq!(result, "No schedules configured.");
    }

    #[tokio::test]
    async fn test_view_schedules_list_all() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[
                {"name": "daily", "cron": "0 0 * * *", "prompt": "summarize"},
                {"name": "hourly", "cron": "0 * * * *", "prompt": "check"}
            ]"#,
        )
        .unwrap();

        let tool = ScheduleViewTool::new(path);
        let args = ScheduleViewArgs { name: None };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("2 schedule(s) configured"));
        assert!(result.contains("daily"));
        assert!(result.contains("hourly"));
    }

    #[tokio::test]
    async fn test_view_schedules_filter_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[
                {"name": "daily", "cron": "0 0 * * *", "prompt": "summarize"},
                {"name": "hourly", "cron": "0 * * * *", "prompt": "check"}
            ]"#,
        )
        .unwrap();

        let tool = ScheduleViewTool::new(path);
        let args = ScheduleViewArgs {
            name: Some("daily".to_string()),
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("daily"));
        assert!(!result.contains("hourly"));
    }

    #[tokio::test]
    async fn test_view_schedules_name_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[{"name": "daily", "cron": "0 0 * * *", "prompt": "summarize"}]"#,
        )
        .unwrap();

        let tool = ScheduleViewTool::new(path);
        let args = ScheduleViewArgs {
            name: Some("weekly".to_string()),
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("No schedule named 'weekly'"));
        assert!(result.contains("daily"));
    }

    #[tokio::test]
    async fn test_view_schedules_case_insensitive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("schedules.json");
        std::fs::write(
            &path,
            r#"[{"name": "Daily-Summary", "cron": "0 0 * * *", "prompt": "summarize"}]"#,
        )
        .unwrap();

        let tool = ScheduleViewTool::new(path);
        let args = ScheduleViewArgs {
            name: Some("daily-summary".to_string()),
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.contains("Daily-Summary"));
    }
}
