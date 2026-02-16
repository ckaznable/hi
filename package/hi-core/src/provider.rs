use anyhow::Result;
use futures::StreamExt;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::completion::Chat;
use rig::completion::message::Message;
use rig::completion::PromptError;
use rig::message::Text;
use rig::prelude::CompletionClient;
use rig::providers::{anthropic, gemini, ollama, openai};
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use rig::tool::ToolDyn;
use shared::config::{ModelConfig, Provider, SmallModelConfig};
use tokio::sync::mpsc;

use hi_tools::{BashTool, ListFilesTool, MemoryTool, ReadFileTool, ReadSkillsTool, ScheduleViewTool, SkillSummary, WriteFileTool};

pub const STREAM_CHANNEL_CAPACITY: usize = 256;

pub enum ChatAgent {
    OpenAI(Agent<openai::completion::CompletionModel>),
    OpenAICompatible(Agent<openai::completion::CompletionModel>),
    Anthropic(Agent<anthropic::completion::CompletionModel>),
    Gemini(Agent<gemini::completion::CompletionModel>),
    Ollama(Agent<ollama::CompletionModel>),
}

macro_rules! consume_stream {
    ($agent:expr, $prompt:expr, $history:expr, $chunk_tx:expr, $acc:expr) => {{
        let mut stream = $agent.stream_chat($prompt, $history).await;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::Text(Text { text }),
                )) => {
                    $acc.push_str(&text);
                    if let Err(e) = $chunk_tx.send(text).await {
                        tracing::warn!("Channel send failed: {e}");
                    }
                }
                Ok(MultiTurnStreamItem::FinalResponse(_)) => {}
                Err(e) => return Err(anyhow::anyhow!("{e}")),
                _ => continue,
            }
        }
        Ok(std::mem::take($acc))
    }};
}

impl ChatAgent {
    pub async fn chat(
        &self,
        prompt: impl Into<Message> + Send + Sync,
        history: Vec<Message>,
    ) -> Result<String, PromptError> {
        match self {
            Self::OpenAI(a) => a.chat(prompt, history).await,
            Self::OpenAICompatible(a) => a.chat(prompt, history).await,
            Self::Anthropic(a) => a.chat(prompt, history).await,
            Self::Gemini(a) => a.chat(prompt, history).await,
            Self::Ollama(a) => a.chat(prompt, history).await,
        }
    }

    pub async fn stream_chat(
        &self,
        prompt: impl Into<Message> + Send + Sync,
        history: Vec<Message>,
        chunk_tx: mpsc::Sender<String>,
    ) -> Result<String> {
        let msg = prompt.into();
        let mut acc = String::new();
        match self {
            Self::OpenAI(a) => consume_stream!(a, msg, history, chunk_tx, &mut acc),
            Self::OpenAICompatible(a) => consume_stream!(a, msg, history, chunk_tx, &mut acc),
            Self::Anthropic(a) => consume_stream!(a, msg, history, chunk_tx, &mut acc),
            Self::Gemini(a) => consume_stream!(a, msg, history, chunk_tx, &mut acc),
            Self::Ollama(a) => consume_stream!(a, msg, history, chunk_tx, &mut acc),
        }
    }
}

fn build_tools(skill_summaries: Vec<SkillSummary>) -> Vec<Box<dyn ToolDyn>> {
    let memory_path = shared::paths::data_dir()
        .map(|d| d.join("memory.md"))
        .unwrap_or_else(|_| std::path::PathBuf::from("memory.md"));
    let schedules_path = shared::paths::data_dir()
        .map(|d| d.join("schedules.json"))
        .unwrap_or_else(|_| std::path::PathBuf::from("schedules.json"));
    vec![
        Box::new(BashTool) as Box<dyn ToolDyn>,
        Box::new(ListFilesTool),
        Box::new(ReadFileTool),
        Box::new(WriteFileTool),
        Box::new(ReadSkillsTool::new(skill_summaries)),
        Box::new(MemoryTool::new(memory_path)),
        Box::new(ScheduleViewTool::new(schedules_path)),
    ]
}

pub fn create_agent(
    config: &ModelConfig,
    preamble: Option<&str>,
    skill_summaries: Vec<SkillSummary>,
    extra_tools: Vec<Box<dyn ToolDyn>>,
) -> Result<ChatAgent> {
    let mut tools = build_tools(skill_summaries);
    tools.extend(extra_tools);
    create_agent_from_parts(&config.provider, &config.model, &config.api_key, &config.api_base, preamble, tools)
}

pub fn create_agent_from_small(
    config: &SmallModelConfig,
    preamble: Option<&str>,
) -> Result<ChatAgent> {
    create_agent_from_parts(&config.provider, &config.model, &config.api_key, &config.api_base, preamble, vec![])
}

pub fn create_agent_from_small_with_tools(
    config: &SmallModelConfig,
    preamble: Option<&str>,
    skill_summaries: Vec<SkillSummary>,
) -> Result<ChatAgent> {
    let tools = build_tools(skill_summaries);
    create_agent_from_parts(&config.provider, &config.model, &config.api_key, &config.api_base, preamble, tools)
}

pub(crate) fn create_agent_from_parts(
    provider: &Provider,
    model: &str,
    api_key: &Option<String>,
    api_base: &Option<String>,
    preamble: Option<&str>,
    tools: Vec<Box<dyn ToolDyn>>,
) -> Result<ChatAgent> {
    match provider {
        Provider::OpenAI => {
            let key = api_key.as_deref().unwrap_or_default();
            let mut builder = openai::CompletionsClient::builder().api_key(key);
            if let Some(base) = api_base {
                builder = builder.base_url(base);
            }
            let client = builder.build()?;
            let agent = if tools.is_empty() {
                client.agent(model)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            } else {
                client.agent(model)
                    .tools(tools)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            };
            Ok(ChatAgent::OpenAI(agent))
        }
        Provider::OpenAICompatible => {
            let key = api_key.as_deref().unwrap_or_default();
            let mut builder = openai::CompletionsClient::builder().api_key(key);
            if let Some(base) = api_base {
                builder = builder.base_url(base);
            }
            let client = builder.build()?;
            let agent = if tools.is_empty() {
                client.agent(model)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            } else {
                client.agent(model)
                    .tools(tools)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            };
            Ok(ChatAgent::OpenAICompatible(agent))
        }
        Provider::Anthropic => {
            let key = api_key.as_deref().unwrap_or_default();
            let mut builder = anthropic::Client::builder().api_key(key);
            if let Some(base) = api_base {
                builder = builder.base_url(base);
            }
            let client = builder.build()?;
            let agent = if tools.is_empty() {
                client.agent(model)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            } else {
                client.agent(model)
                    .tools(tools)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            };
            Ok(ChatAgent::Anthropic(agent))
        }
        Provider::Gemini => {
            let key = api_key.as_deref().unwrap_or_default();
            let mut builder = gemini::Client::builder().api_key(key);
            if let Some(base) = api_base {
                builder = builder.base_url(base);
            }
            let client = builder.build()?;
            let agent = if tools.is_empty() {
                client.agent(model)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            } else {
                client.agent(model)
                    .tools(tools)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            };
            Ok(ChatAgent::Gemini(agent))
        }
        Provider::Ollama => {
            let mut builder = ollama::Client::builder().api_key(rig::client::Nothing);
            if let Some(base) = api_base {
                builder = builder.base_url(base);
            }
            let client = builder.build()?;
            let agent = if tools.is_empty() {
                client.agent(model)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            } else {
                client.agent(model)
                    .tools(tools)
                    .preamble(preamble.unwrap_or_default())
                    .build()
            };
            Ok(ChatAgent::Ollama(agent))
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_stream_accumulation_single_buffer() {
        let (tx, mut rx) = mpsc::channel::<String>(256);

        let chunks = vec!["Hello", " ", "world", "!"];
        for chunk in &chunks {
            tx.send(chunk.to_string()).await.unwrap();
        }
        drop(tx);

        let mut acc = String::new();
        while let Some(chunk) = rx.recv().await {
            acc.push_str(&chunk);
        }

        assert_eq!(acc, "Hello world!");
    }

    #[tokio::test]
    async fn test_stream_accumulation_empty() {
        let (tx, mut rx) = mpsc::channel::<String>(256);
        drop(tx);

        let mut acc = String::new();
        while let Some(chunk) = rx.recv().await {
            acc.push_str(&chunk);
        }

        assert_eq!(acc, "");
    }

    #[tokio::test]
    async fn test_stream_forwarding_pattern() {
        let (stream_tx, mut stream_rx) = mpsc::channel::<String>(256);
        let (reply_tx, mut reply_rx) = mpsc::channel::<String>(256);

        let forwarder = tokio::spawn(async move {
            while let Some(chunk) = stream_rx.recv().await {
                let _ = reply_tx.send(chunk).await;
            }
        });

        stream_tx.send("chunk1".to_string()).await.unwrap();
        stream_tx.send("chunk2".to_string()).await.unwrap();
        drop(stream_tx);

        forwarder.await.unwrap();

        let mut received = Vec::new();
        while let Ok(chunk) = reply_rx.try_recv() {
            received.push(chunk);
        }

        assert_eq!(received, vec!["chunk1", "chunk2"]);
    }
}
