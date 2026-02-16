use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WriteFileError(String);

#[derive(Deserialize)]
pub struct WriteFileArgs {
    pub path: String,
    pub content: String,
}

pub struct WriteFileTool;

impl Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = WriteFileError;
    type Args = WriteFileArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file at the given path".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The file path to write to"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if let Some(parent) = std::path::Path::new(&args.path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| WriteFileError(e.to_string()))?;
        }
        tokio::fs::write(&args.path, &args.content)
            .await
            .map_err(|e| WriteFileError(e.to_string()))?;
        Ok(format!(
            "Written {} bytes to {}",
            args.content.len(),
            args.path
        ))
    }
}
