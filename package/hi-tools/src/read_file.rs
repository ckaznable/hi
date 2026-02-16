use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ReadFileError(String);

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub path: String,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

pub struct ReadFileTool;

impl Tool for ReadFileTool {
    const NAME: &'static str = "read_file";

    type Error = ReadFileError;
    type Args = ReadFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file at the given path. \
                Supports optional line-based offset and limit for partial reads. \
                Output includes line numbers."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to read"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Starting line number (1-based). Omit to start from the beginning."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of lines to return. Omit to read all remaining lines."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let content = tokio::fs::read_to_string(&args.path)
            .await
            .map_err(|e| ReadFileError(e.to_string()))?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // offset is 1-based; default to line 1
        let start = args
            .offset
            .map(|o| if o == 0 { 0 } else { o - 1 })
            .unwrap_or(0)
            .min(total_lines);

        let end = match args.limit {
            Some(l) => (start + l).min(total_lines),
            None => total_lines,
        };

        let selected = &lines[start..end];

        let width = if end > 0 { end.to_string().len() } else { 1 };
        let numbered: Vec<String> = selected
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>width$}| {}", start + i + 1, line, width = width))
            .collect();

        let mut output = numbered.join("\n");
        if total_lines > end || start > 0 {
            output.push_str(&format!(
                "\n[Showing lines {}-{} of {}]",
                start + 1,
                end,
                total_lines
            ));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_file_full() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "line one").unwrap();
        writeln!(tmp, "line two").unwrap();
        writeln!(tmp, "line three").unwrap();

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: None,
            limit: None,
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(result.contains("1| line one"));
        assert!(result.contains("2| line two"));
        assert!(result.contains("3| line three"));
        assert!(!result.contains("[Showing lines"));
    }

    #[tokio::test]
    async fn test_read_file_with_offset_and_limit() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        for i in 1..=10 {
            writeln!(tmp, "line {i}").unwrap();
        }

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: Some(3),
            limit: Some(2),
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(result.contains("3| line 3"));
        assert!(result.contains("4| line 4"));
        assert!(!result.contains("5| line 5"));
        assert!(result.contains("[Showing lines 3-4 of 10]"));
    }

    #[tokio::test]
    async fn test_read_file_offset_only() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        for i in 1..=5 {
            writeln!(tmp, "line {i}").unwrap();
        }

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: Some(4),
            limit: None,
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(result.contains("4| line 4"));
        assert!(result.contains("5| line 5"));
        assert!(!result.contains("3| line 3"));
        assert!(result.contains("[Showing lines 4-5 of 5]"));
    }

    #[tokio::test]
    async fn test_read_file_limit_exceeds_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "only line").unwrap();

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: Some(1),
            limit: Some(100),
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(result.contains("1| only line"));
        assert!(!result.contains("[Showing lines"));
    }

    #[tokio::test]
    async fn test_read_file_offset_beyond_end() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "line 1").unwrap();

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: Some(999),
            limit: None,
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(
            result.contains("[Showing lines 2-1 of 1]")
                || result.is_empty()
                || result.contains("[Showing lines")
        );
    }

    #[tokio::test]
    async fn test_read_file_nonexistent() {
        let args = ReadFileArgs {
            path: "/nonexistent/file.txt".to_string(),
            offset: None,
            limit: None,
        };

        let result = ReadFileTool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_file_utf8_content() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "你好世界").unwrap();
        writeln!(tmp, "こんにちは").unwrap();
        writeln!(tmp, "🎉🎊").unwrap();

        let args = ReadFileArgs {
            path: tmp.path().to_str().unwrap().to_string(),
            offset: None,
            limit: None,
        };

        let result = ReadFileTool.call(args).await.unwrap();
        assert!(result.contains("1| 你好世界"));
        assert!(result.contains("2| こんにちは"));
        assert!(result.contains("3| 🎉🎊"));
    }
}
