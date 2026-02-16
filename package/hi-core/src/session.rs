use anyhow::Result;
use rig::completion::message::Message;
use tokio::sync::mpsc;

use hi_history::{ChatHistory, ChatMessage};
use shared::config::{CompactStrategy, ModelConfig};

use crate::context::ContextManager;
use crate::mcp::{McpManager, load_and_connect};
use crate::provider::{ChatAgent, create_agent, create_agent_from_small, create_agent_from_small_with_tools};
use crate::skills::{Skill, build_preamble, load_skills};

const DEFAULT_COMPACT_PROMPT: &str = "Summarize the following conversation concisely. \
Preserve key topics, decisions, tool results, and any context needed to continue naturally. \
Output only the summary, no preamble.";

pub const DEFAULT_PREAMBLE: &str = "You are a helpful assistant with access to tools. \
Use them when appropriate to fulfill user requests.";

fn refresh_runtime_index(config: &ModelConfig, data_dir: &std::path::Path) {
    let memory_path = data_dir.join("memory.md");
    let memory_sections = shared::runtime_index::refresh_memory_sections(&memory_path);
    let schedules = shared::schedule_store::load(config.schedules.as_deref());
    let schedule_names = shared::runtime_index::refresh_schedule_names(&schedules);
    let mut index = shared::runtime_index::load();
    index.memory_sections = memory_sections;
    index.schedule_names = schedule_names;
    let _ = shared::runtime_index::save(&index);
}

pub struct ChatSession {
    agent: ChatAgent,
    history: ChatHistory,
    context_manager: ContextManager,
    skills: Vec<Skill>,
    config: ModelConfig,
    using_small_model: bool,
    _mcp_manager: McpManager,
    mcp_tool_names: Vec<String>,
}

impl ChatSession {
    pub async fn new(config: ModelConfig) -> Result<Self> {
        let config_dir = shared::paths::config_dir()?;
        let data_dir = shared::paths::data_dir()?;

        let skills = load_skills(&config_dir)?;
        let effective_preamble = config
            .preamble
            .as_deref()
            .or(Some(DEFAULT_PREAMBLE));
        let preamble = build_preamble(effective_preamble, &skills);
        let skill_summaries = ContextManager::skill_summaries(&skills);

        let (mcp_manager, mcp_tools) = load_and_connect().await;
        let mcp_tool_names: Vec<String> = mcp_tools.iter().map(|t| t.name().to_string()).collect();

        let agent = create_agent(&config, Some(&preamble), skill_summaries, mcp_tools)?;
        let history = ChatHistory::load(&data_dir)?;
        let context_manager = ContextManager::new();

        refresh_runtime_index(&config, &data_dir);

        Ok(Self {
            agent,
            history,
            context_manager,
            skills,
            config,
            using_small_model: false,
            _mcp_manager: mcp_manager,
            mcp_tool_names,
        })
    }

    fn effective_preamble(&self) -> &str {
        self.config
            .preamble
            .as_deref()
            .unwrap_or(DEFAULT_PREAMBLE)
    }

    pub async fn send_message(&mut self, text: &str) -> Result<String> {
        self.run_compact_if_needed().await;

        let mut tool_descriptions: Vec<String> = vec![
            "bash: Execute shell commands".to_string(),
            "list_files: List directory contents".to_string(),
            "read_file: Read file contents (supports line offset and limit)".to_string(),
            "write_file: Write content to a file".to_string(),
            "read_skills: List available skills".to_string(),
            "memory: Read/write persistent hierarchical markdown memory".to_string(),
            "view_schedules: View configured cron schedules".to_string(),
        ];
        for name in &self.mcp_tool_names {
            tool_descriptions.push(format!("{name}: MCP tool"));
        }

        let preamble = self.effective_preamble().to_string();
        let context_msg = self.context_manager.build_context_message(
            Some(&preamble),
            &tool_descriptions,
            &self.skills,
        );

        if let Some(ctx) = context_msg {
            self.history.push(ChatMessage::system(ctx));
        }

        self.history.push(ChatMessage::user(text));

        let rig_messages = self.history.to_rig_messages();
        let prompt = Message::user(text);
        let response = match self.agent.chat(prompt, rig_messages).await {
            Ok(r) => r,
            Err(e) => {
                if !self.using_small_model && self.config.small_model.is_some() {
                    tracing::warn!("Primary model failed ({e}), falling back to small model");
                    self.switch_to_small_model()?;
                    let rig_messages = self.history.to_rig_messages();
                    let retry_prompt = Message::user(text);
                    self.agent.chat(retry_prompt, rig_messages).await?
                } else {
                    return Err(e.into());
                }
            }
        };

        self.history.push(ChatMessage::assistant(&response));
        self.history.save()?;

        Ok(response)
    }

    async fn run_compact_if_needed(&mut self) {
        let (trigger_ratio, compact_enabled, strategy) = match &self.config.compact {
            Some(c) if c.enabled => (c.trigger_ratio, true, c.strategy.clone()),
            _ => (0.8, false, CompactStrategy::Truncate),
        };

        if !self.history.needs_compact_with_ratio(self.config.context_window, trigger_ratio) {
            return;
        }

        let compacted = if compact_enabled && strategy == CompactStrategy::SmallModel {
            self.try_small_model_compact().await
        } else {
            false
        };

        if !compacted {
            self.history.compact(self.config.context_window);
        }

        self.context_manager.mark_dirty();
    }

    async fn try_small_model_compact(&mut self) -> bool {
        let compact_config = match &self.config.compact {
            Some(c) => c,
            None => return false,
        };

        let resolved = self.config.resolve_model_ref(&compact_config.model);
        let agent = match create_agent_from_small(&resolved, None) {
            Ok(a) => a,
            Err(_) => return false,
        };

        let language = self.history.detect_user_language();

        let conversation_text: String = self
            .history
            .messages()
            .iter()
            .map(|m| format!("[{}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let base_prompt = compact_config
            .prompt
            .as_deref()
            .unwrap_or(DEFAULT_COMPACT_PROMPT);

        let prompt = if let Some(ref lang) = language {
            format!(
                "{}\n\n[Current user language: {}]\n\n{}",
                base_prompt, lang, conversation_text
            )
        } else {
            format!("{}\n\n{}", base_prompt, conversation_text)
        };

        match agent.chat(Message::user(&prompt), vec![]).await {
            Ok(summary) => {
                self.history
                    .compact_with_summary(&summary, language.as_deref());
                true
            }
            Err(_) => false,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.history.reset()?;
        self.context_manager.mark_dirty();
        Ok(())
    }

    /// Manually trigger history compaction, regardless of context window ratio.
    /// Returns `true` if compaction was performed, `false` if history was already empty.
    pub async fn run_compact(&mut self) -> bool {
        if self.history.messages().is_empty() {
            return false;
        }

        let (compact_enabled, strategy) = match &self.config.compact {
            Some(c) if c.enabled => (true, c.strategy.clone()),
            _ => (false, CompactStrategy::Truncate),
        };

        let compacted = if compact_enabled && strategy == CompactStrategy::SmallModel {
            self.try_small_model_compact().await
        } else {
            false
        };

        if !compacted {
            self.history.compact(self.config.context_window);
        }

        self.context_manager.mark_dirty();
        true
    }

    pub fn history(&self) -> &ChatHistory {
        &self.history
    }

    pub async fn send_message_streaming(
        &mut self,
        text: &str,
        chunk_tx: mpsc::Sender<String>,
    ) -> Result<String> {
        self.run_compact_if_needed().await;

        let mut tool_descriptions: Vec<String> = vec![
            "bash: Execute shell commands".to_string(),
            "list_files: List directory contents".to_string(),
            "read_file: Read file contents (supports line offset and limit)".to_string(),
            "write_file: Write content to a file".to_string(),
            "read_skills: List available skills".to_string(),
            "memory: Read/write persistent hierarchical markdown memory".to_string(),
            "view_schedules: View configured cron schedules".to_string(),
        ];
        for name in &self.mcp_tool_names {
            tool_descriptions.push(format!("{name}: MCP tool"));
        }

        let preamble = self.effective_preamble().to_string();
        let context_msg = self.context_manager.build_context_message(
            Some(&preamble),
            &tool_descriptions,
            &self.skills,
        );

        if let Some(ctx) = context_msg {
            self.history.push(ChatMessage::system(ctx));
        }

        self.history.push(ChatMessage::user(text));

        let rig_messages = self.history.to_rig_messages();
        let prompt = Message::user(text);
        let fallback_tx = chunk_tx.clone();
        let response = match self.agent.stream_chat(prompt, rig_messages, chunk_tx).await {
            Ok(r) => r,
            Err(e) => {
                if !self.using_small_model && self.config.small_model.is_some() {
                    tracing::warn!("Primary model failed ({e}), falling back to small model");
                    self.switch_to_small_model()?;
                    let rig_messages = self.history.to_rig_messages();
                    let retry_prompt = Message::user(text);
                    self.agent.stream_chat(retry_prompt, rig_messages, fallback_tx).await?
                } else {
                    return Err(e);
                }
            }
        };

        self.history.push(ChatMessage::assistant(&response));
        self.history.save()?;

        Ok(response)
    }

    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    pub fn current_model_name(&self) -> &str {
        if self.using_small_model {
            self.config
                .small_model
                .as_ref()
                .map(|s| s.model.as_str())
                .unwrap_or(&self.config.model)
        } else {
            &self.config.model
        }
    }

    pub fn is_using_small_model(&self) -> bool {
        self.using_small_model
    }

    /// Fails if no `small_model` is configured.
    pub fn switch_to_small_model(&mut self) -> Result<String> {
        let small_config = self
            .config
            .small_model
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No small_model configured"))?;

        let preamble = build_preamble(
            self.config.preamble.as_deref().or(Some(DEFAULT_PREAMBLE)),
            &self.skills,
        );
        let skill_summaries = ContextManager::skill_summaries(&self.skills);
        let agent = create_agent_from_small_with_tools(small_config, Some(&preamble), skill_summaries)?;

        self.agent = agent;
        self.using_small_model = true;
        self.context_manager.mark_dirty();

        Ok(small_config.model.clone())
    }

    pub fn switch_to_primary_model(&mut self) -> Result<String> {
        let preamble = build_preamble(
            self.config.preamble.as_deref().or(Some(DEFAULT_PREAMBLE)),
            &self.skills,
        );
        let skill_summaries = ContextManager::skill_summaries(&self.skills);
        let agent = create_agent(&self.config, Some(&preamble), skill_summaries, vec![])?;

        self.agent = agent;
        self.using_small_model = false;
        self.context_manager.mark_dirty();

        Ok(self.config.model.clone())
    }
}
