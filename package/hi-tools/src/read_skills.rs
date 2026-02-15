use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ReadSkillsError(String);

#[derive(Deserialize)]
pub struct ReadSkillsArgs {}

pub struct ReadSkillsTool {
    summaries: Vec<SkillSummary>,
}

impl ReadSkillsTool {
    pub fn new(summaries: Vec<SkillSummary>) -> Self {
        Self { summaries }
    }
}

impl Tool for ReadSkillsTool {
    const NAME: &'static str = "read_skills";

    type Error = ReadSkillsError;
    type Args = ReadSkillsArgs;
    type Output = Vec<SkillSummary>;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read_skills".to_string(),
            description: "List all available skills with their names and descriptions".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(self.summaries.clone())
    }
}
