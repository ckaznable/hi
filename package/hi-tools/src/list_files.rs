use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ListFilesError(String);

#[derive(Deserialize)]
pub struct ListFilesArgs {
    pub path: String,
}

pub struct ListFilesTool;

impl Tool for ListFilesTool {
    const NAME: &'static str = "list_files";

    type Error = ListFilesError;
    type Args = ListFilesArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List files and directories at the given path".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The directory path to list"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&args.path)
            .await
            .map_err(|e| ListFilesError(e.to_string()))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| ListFilesError(e.to_string()))?
        {
            if let Some(name) = entry.file_name().to_str() {
                let file_type = entry
                    .file_type()
                    .await
                    .map_err(|e| ListFilesError(e.to_string()))?;
                if file_type.is_dir() {
                    entries.push(format!("{}/", name));
                } else {
                    entries.push(name.to_string());
                }
            }
        }

        entries.sort();
        Ok(entries.join("\n"))
    }
}
