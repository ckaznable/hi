use anyhow::Result;
use rig::completion::message::Message;
use tokio::sync::mpsc;

use hi_history::{ChatHistory, ChatMessage};
use shared::config::{CompactStrategy, ModelConfig};

use crate::context::ContextManager;
use crate::provider::{ChatAgent, create_agent, create_agent_from_small};
use crate::skills::{Skill, build_preamble, load_skills};

const DEFAULT_COMPACT_PROMPT: &str = "Summarize the following conversation concisely. \
Preserve key topics, decisions, tool results, and any context needed to continue naturally. \
Output only the summary, no preamble.";

pub struct ChatSession {
    agent: ChatAgent,
    history: ChatHistory,
    context_manager: ContextManager,
    skills: Vec<Skill>,
    config: ModelConfig,
}

impl ChatSession {
    pub fn new(config: ModelConfig) -> Result<Self> {
        let config_dir = shared::paths::config_dir()?;
        let data_dir = shared::paths::data_dir()?;

        let skills = load_skills(&config_dir)?;
        let preamble = build_preamble(config.preamble.as_deref(), &skills);
        let skill_summaries = ContextManager::skill_summaries(&skills);

        let agent = create_agent(&config, Some(&preamble), skill_summaries)?;
        let history = ChatHistory::load(&data_dir)?;
        let context_manager = ContextManager::new();

        Ok(Self {
            agent,
            history,
            context_manager,
            skills,
            config,
        })
    }

    pub async fn send_message(&mut self, text: &str) -> Result<String> {
        self.run_compact_if_needed().await;

        let tool_descriptions: Vec<String> = vec![
            "bash: Execute shell commands".to_string(),
            "list_files: List directory contents".to_string(),
            "read_file: Read file contents".to_string(),
            "write_file: Write content to a file".to_string(),
            "read_skills: List available skills".to_string(),
        ];

        let context_msg = self.context_manager.build_context_message(
            self.config.preamble.as_deref(),
            &tool_descriptions,
            &self.skills,
        );

        if let Some(ctx) = context_msg {
            self.history.push(ChatMessage::system(ctx));
        }

        self.history.push(ChatMessage::user(text));

        let rig_messages = self.history.to_rig_messages();
        let prompt = Message::user(text);
        let response = self.agent.chat(prompt, rig_messages).await?;

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

        let tool_descriptions: Vec<String> = vec![
            "bash: Execute shell commands".to_string(),
            "list_files: List directory contents".to_string(),
            "read_file: Read file contents".to_string(),
            "write_file: Write content to a file".to_string(),
            "read_skills: List available skills".to_string(),
        ];

        let context_msg = self.context_manager.build_context_message(
            self.config.preamble.as_deref(),
            &tool_descriptions,
            &self.skills,
        );

        if let Some(ctx) = context_msg {
            self.history.push(ChatMessage::system(ctx));
        }

        self.history.push(ChatMessage::user(text));

        let rig_messages = self.history.to_rig_messages();
        let prompt = Message::user(text);
        let response = self.agent.stream_chat(prompt, rig_messages, chunk_tx).await?;

        self.history.push(ChatMessage::assistant(&response));
        self.history.save()?;

        Ok(response)
    }

    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}
