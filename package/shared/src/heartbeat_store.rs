use std::fmt;
use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Status of a heartbeat task in the ledger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Done,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in-progress"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "pending" => Ok(TaskStatus::Pending),
            "in-progress" => Ok(TaskStatus::InProgress),
            "done" => Ok(TaskStatus::Done),
            "failed" => Ok(TaskStatus::Failed),
            other => Err(format!("Unknown task status: '{other}'")),
        }
    }
}

/// A single heartbeat task entry.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatTask {
    pub id: String,
    pub status: TaskStatus,
    pub title: String,
    pub description: Option<String>,
}

/// The full heartbeat ledger parsed from HEARTBEAT.md.
#[derive(Debug, Clone, PartialEq)]
pub struct HeartbeatLedger {
    /// Lines before the first task entry (header, comments, etc.)
    pub header: String,
    pub tasks: Vec<HeartbeatTask>,
}

const DEFAULT_TEMPLATE: &str = "# Heartbeat Tasks\n\n";

/// Validate whether a status transition is allowed.
///
/// Valid transitions:
/// - `Pending` → `InProgress`
/// - `InProgress` → `Done`
/// - `InProgress` → `Failed`
pub fn validate_transition(from: &TaskStatus, to: &TaskStatus) -> bool {
    matches!(
        (from, to),
        (TaskStatus::Pending, TaskStatus::InProgress)
            | (TaskStatus::InProgress, TaskStatus::Done)
            | (TaskStatus::InProgress, TaskStatus::Failed)
    )
}

/// Parse a HEARTBEAT.md text into a ledger.
///
/// Non-destructive: malformed lines are preserved in the header section.
/// Never panics on bad input.
pub fn parse(text: &str) -> HeartbeatLedger {
    let mut header = String::new();
    let mut tasks: Vec<HeartbeatTask> = Vec::new();
    let mut in_tasks = false;

    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if let Some(task) = parse_task_line(line) {
            in_tasks = true;
            let mut desc_lines: Vec<&str> = Vec::new();
            while let Some(&next) = lines.peek() {
                if next.starts_with("  ") && !next.trim().is_empty() {
                    desc_lines.push(next.trim());
                    lines.next();
                } else {
                    break;
                }
            }

            let description = if desc_lines.is_empty() {
                None
            } else {
                Some(desc_lines.join("\n"))
            };

            tasks.push(HeartbeatTask {
                id: task.0,
                status: task.1,
                title: task.2,
                description,
            });
        } else if !in_tasks {
            if !header.is_empty() {
                header.push('\n');
            }
            header.push_str(line);
        }
    }

    HeartbeatLedger { header, tasks }
}

/// Try to parse a single task line: `- [status] task-id: Title text`
fn parse_task_line(line: &str) -> Option<(String, TaskStatus, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("- [")?;
    let bracket_end = rest.find(']')?;
    let status_str = &rest[..bracket_end];
    let status = TaskStatus::from_str(status_str).ok()?;

    let after_bracket = &rest[bracket_end + 1..];
    let after_bracket = after_bracket.trim_start();

    let colon_pos = after_bracket.find(':')?;
    let id = after_bracket[..colon_pos].trim().to_string();
    let title = after_bracket[colon_pos + 1..].trim().to_string();

    if id.is_empty() || title.is_empty() {
        return None;
    }

    Some((id, status, title))
}

/// Serialize a ledger back to markdown.
pub fn serialize(ledger: &HeartbeatLedger) -> String {
    let mut out = String::new();

    if !ledger.header.is_empty() {
        out.push_str(&ledger.header);
        if !ledger.header.ends_with('\n') {
            out.push('\n');
        }
        if !ledger.header.ends_with("\n\n") {
            out.push('\n');
        }
    }

    for task in &ledger.tasks {
        out.push_str(&format!(
            "- [{}] {}: {}\n",
            task.status, task.id, task.title
        ));
        if let Some(ref desc) = task.description {
            for desc_line in desc.lines() {
                out.push_str(&format!("  {}\n", desc_line));
            }
        }
    }

    out
}

/// Load a heartbeat ledger from disk.
///
/// If the file does not exist, creates a default template and returns an empty ledger.
/// If the file exists but is malformed, returns what can be parsed.
pub fn load(path: &Path) -> Result<HeartbeatLedger> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        std::fs::write(path, DEFAULT_TEMPLATE)
            .with_context(|| format!("Failed to create HEARTBEAT.md at: {}", path.display()))?;
        return Ok(parse(DEFAULT_TEMPLATE));
    }

    let text = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read HEARTBEAT.md at: {}", path.display()))?;
    Ok(parse(&text))
}

/// Save a heartbeat ledger to disk.
pub fn save(path: &Path, ledger: &HeartbeatLedger) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    let text = serialize(ledger);
    std::fs::write(path, &text)
        .with_context(|| format!("Failed to write HEARTBEAT.md at: {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(TaskStatus::Pending.to_string(), "pending");
        assert_eq!(TaskStatus::InProgress.to_string(), "in-progress");
        assert_eq!(TaskStatus::Done.to_string(), "done");
        assert_eq!(TaskStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            TaskStatus::from_str("pending").unwrap(),
            TaskStatus::Pending
        );
        assert_eq!(
            TaskStatus::from_str("in-progress").unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!(TaskStatus::from_str("done").unwrap(), TaskStatus::Done);
        assert_eq!(TaskStatus::from_str("failed").unwrap(), TaskStatus::Failed);
        assert_eq!(
            TaskStatus::from_str("PENDING").unwrap(),
            TaskStatus::Pending
        );
        assert!(TaskStatus::from_str("unknown").is_err());
    }

    #[test]
    fn test_validate_transition_valid() {
        assert!(validate_transition(
            &TaskStatus::Pending,
            &TaskStatus::InProgress
        ));
        assert!(validate_transition(
            &TaskStatus::InProgress,
            &TaskStatus::Done
        ));
        assert!(validate_transition(
            &TaskStatus::InProgress,
            &TaskStatus::Failed
        ));
    }

    #[test]
    fn test_validate_transition_invalid() {
        assert!(!validate_transition(
            &TaskStatus::Pending,
            &TaskStatus::Done
        ));
        assert!(!validate_transition(
            &TaskStatus::Pending,
            &TaskStatus::Failed
        ));
        assert!(!validate_transition(
            &TaskStatus::Pending,
            &TaskStatus::Pending
        ));
        assert!(!validate_transition(
            &TaskStatus::InProgress,
            &TaskStatus::Pending
        ));
        assert!(!validate_transition(
            &TaskStatus::InProgress,
            &TaskStatus::InProgress
        ));
        assert!(!validate_transition(
            &TaskStatus::Done,
            &TaskStatus::Pending
        ));
        assert!(!validate_transition(
            &TaskStatus::Done,
            &TaskStatus::InProgress
        ));
        assert!(!validate_transition(&TaskStatus::Done, &TaskStatus::Done));
        assert!(!validate_transition(&TaskStatus::Done, &TaskStatus::Failed));
        assert!(!validate_transition(
            &TaskStatus::Failed,
            &TaskStatus::Pending
        ));
        assert!(!validate_transition(
            &TaskStatus::Failed,
            &TaskStatus::InProgress
        ));
        assert!(!validate_transition(&TaskStatus::Failed, &TaskStatus::Done));
        assert!(!validate_transition(
            &TaskStatus::Failed,
            &TaskStatus::Failed
        ));
    }

    #[test]
    fn test_parse_empty() {
        let ledger = parse("");
        assert!(ledger.tasks.is_empty());
        assert_eq!(ledger.header, "");
    }

    #[test]
    fn test_parse_header_only() {
        let text = "# Heartbeat Tasks\n\n";
        let ledger = parse(text);
        assert!(ledger.tasks.is_empty());
        assert!(ledger.header.contains("# Heartbeat Tasks"));
    }

    #[test]
    fn test_parse_single_task() {
        let text = "# Heartbeat Tasks\n\n- [pending] check-logs: Review system logs for errors\n";
        let ledger = parse(text);
        assert_eq!(ledger.tasks.len(), 1);
        assert_eq!(ledger.tasks[0].id, "check-logs");
        assert_eq!(ledger.tasks[0].status, TaskStatus::Pending);
        assert_eq!(ledger.tasks[0].title, "Review system logs for errors");
        assert!(ledger.tasks[0].description.is_none());
    }

    #[test]
    fn test_parse_multiple_tasks() {
        let text = "# Heartbeat Tasks\n\n\
            - [pending] task-1: First task\n\
            - [in-progress] task-2: Second task\n\
            - [done] task-3: Third task\n\
            - [failed] task-4: Fourth task\n";
        let ledger = parse(text);
        assert_eq!(ledger.tasks.len(), 4);
        assert_eq!(ledger.tasks[0].status, TaskStatus::Pending);
        assert_eq!(ledger.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(ledger.tasks[2].status, TaskStatus::Done);
        assert_eq!(ledger.tasks[3].status, TaskStatus::Failed);
    }

    #[test]
    fn test_parse_task_with_description() {
        let text = "# Heartbeat Tasks\n\n- [pending] check-logs: Review system logs\n  Look at /var/log/syslog for errors\n  Also check application logs\n- [pending] backup: Run backup\n";
        let ledger = parse(text);
        assert_eq!(ledger.tasks.len(), 2);
        assert_eq!(
            ledger.tasks[0].description.as_deref(),
            Some("Look at /var/log/syslog for errors\nAlso check application logs")
        );
        assert!(ledger.tasks[1].description.is_none());
    }

    #[test]
    fn test_parse_malformed_lines_preserved_in_header() {
        let text = "# Heartbeat Tasks\n\nSome random text\n- [pending] task-1: Valid task\n";
        let ledger = parse(text);
        // "Some random text" is before the first task, so it's in header
        assert!(ledger.header.contains("Some random text"));
        assert_eq!(ledger.tasks.len(), 1);
        assert_eq!(ledger.tasks[0].id, "task-1");
    }

    #[test]
    fn test_parse_invalid_status_skipped() {
        let text = "# Heartbeat Tasks\n\n\
            - [pending] task-1: Valid task\n\
            - [bogus] task-2: Invalid status\n\
            - [done] task-3: Another valid\n";
        let ledger = parse(text);
        // Invalid status line is silently skipped
        assert_eq!(ledger.tasks.len(), 2);
        assert_eq!(ledger.tasks[0].id, "task-1");
        assert_eq!(ledger.tasks[1].id, "task-3");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let ledger = HeartbeatLedger {
            header: "# Heartbeat Tasks".to_string(),
            tasks: vec![
                HeartbeatTask {
                    id: "task-1".to_string(),
                    status: TaskStatus::Pending,
                    title: "First task".to_string(),
                    description: None,
                },
                HeartbeatTask {
                    id: "task-2".to_string(),
                    status: TaskStatus::InProgress,
                    title: "Second task".to_string(),
                    description: Some("With a description".to_string()),
                },
            ],
        };

        let text = serialize(&ledger);
        let reparsed = parse(&text);
        assert_eq!(reparsed.tasks.len(), 2);
        assert_eq!(reparsed.tasks[0].id, "task-1");
        assert_eq!(reparsed.tasks[0].status, TaskStatus::Pending);
        assert_eq!(reparsed.tasks[1].id, "task-2");
        assert_eq!(reparsed.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(
            reparsed.tasks[1].description.as_deref(),
            Some("With a description")
        );
    }

    #[test]
    fn test_serialize_preserves_order() {
        let ledger = HeartbeatLedger {
            header: "# Heartbeat Tasks".to_string(),
            tasks: vec![
                HeartbeatTask {
                    id: "z-task".to_string(),
                    status: TaskStatus::Pending,
                    title: "Z task".to_string(),
                    description: None,
                },
                HeartbeatTask {
                    id: "a-task".to_string(),
                    status: TaskStatus::Done,
                    title: "A task".to_string(),
                    description: None,
                },
            ],
        };

        let text = serialize(&ledger);
        let z_pos = text.find("z-task").unwrap();
        let a_pos = text.find("a-task").unwrap();
        assert!(z_pos < a_pos);
    }

    #[test]
    fn test_load_creates_file_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        assert!(!path.exists());

        let ledger = load(&path).unwrap();
        assert!(path.exists());
        assert!(ledger.tasks.is_empty());
        assert!(ledger.header.contains("Heartbeat Tasks"));
    }

    #[test]
    fn test_load_reads_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");
        std::fs::write(&path, "# Tasks\n\n- [pending] t1: Do something\n").unwrap();

        let ledger = load(&path).unwrap();
        assert_eq!(ledger.tasks.len(), 1);
        assert_eq!(ledger.tasks[0].id, "t1");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("HEARTBEAT.md");

        let ledger = HeartbeatLedger {
            header: "# Heartbeat Tasks".to_string(),
            tasks: vec![
                HeartbeatTask {
                    id: "task-a".to_string(),
                    status: TaskStatus::Pending,
                    title: "Task A".to_string(),
                    description: Some("Description A".to_string()),
                },
                HeartbeatTask {
                    id: "task-b".to_string(),
                    status: TaskStatus::Done,
                    title: "Task B".to_string(),
                    description: None,
                },
            ],
        };

        save(&path, &ledger).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded.tasks.len(), 2);
        assert_eq!(loaded.tasks[0].id, "task-a");
        assert_eq!(loaded.tasks[0].status, TaskStatus::Pending);
        assert_eq!(
            loaded.tasks[0].description.as_deref(),
            Some("Description A")
        );
        assert_eq!(loaded.tasks[1].id, "task-b");
        assert_eq!(loaded.tasks[1].status, TaskStatus::Done);
    }

    #[test]
    fn test_parse_no_tasks() {
        let text = "# Heartbeat Tasks\n\nNo tasks here, just text.\n";
        let ledger = parse(text);
        assert!(ledger.tasks.is_empty());
    }
}
