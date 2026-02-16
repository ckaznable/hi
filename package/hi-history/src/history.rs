use anyhow::{Context, Result};
use rig::completion::message::Message;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

use shared::config::MemoryConfig;
use shared::memory::evaluate_reclamation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn to_rig_message(&self) -> Message {
        match self.role.as_str() {
            "user" | "system" => Message::user(&self.content),
            "assistant" => Message::assistant(&self.content),
            _ => Message::user(&self.content),
        }
    }
}

pub struct ChatHistory {
    messages: Vec<ChatMessage>,
    history_path: PathBuf,
    memory_config: MemoryConfig,
}

impl ChatHistory {
    pub fn load(data_dir: &std::path::Path) -> Result<Self> {
        let history_path = data_dir.join("history.json.lz4");
        let messages = if history_path.exists() {
            let compressed =
                std::fs::read(&history_path).with_context(|| "Failed to read history file")?;
            let mut decoder = lz4_flex::frame::FrameDecoder::new(&compressed[..]);
            let mut json_bytes = Vec::new();
            decoder
                .read_to_end(&mut json_bytes)
                .with_context(|| "Failed to decompress history")?;
            serde_json::from_slice(&json_bytes).with_context(|| "Failed to parse history JSON")?
        } else {
            Vec::new()
        };

        Ok(Self {
            messages,
            history_path,
            memory_config: MemoryConfig::default(),
        })
    }

    pub fn set_memory_config(&mut self, config: MemoryConfig) {
        self.memory_config = config;
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.history_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json =
            serde_json::to_vec(&self.messages).with_context(|| "Failed to serialize history")?;
        let mut encoder = lz4_flex::frame::FrameEncoder::new(Vec::new());
        encoder
            .write_all(&json)
            .with_context(|| "Failed to compress history")?;
        let compressed = encoder
            .finish()
            .with_context(|| "Failed to finish compression")?;
        std::fs::write(&self.history_path, compressed)
            .with_context(|| "Failed to write history file")?;
        Ok(())
    }

    pub fn push(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn to_rig_messages(&self) -> Vec<Message> {
        self.messages.iter().map(|m| m.to_rig_message()).collect()
    }

    pub fn token_estimate(&self) -> usize {
        self.messages.iter().map(|m| m.content.len()).sum::<usize>() / 4
    }

    pub fn compact(&mut self, context_window: usize) {
        let estimate = self.token_estimate();
        let threshold = (context_window as f64 * 0.8) as usize;
        if estimate > threshold {
            let len = self.messages.len();
            let retain = len / 2;
            let released_bytes: usize = self.messages[..len - retain]
                .iter()
                .map(|m| m.content.len() + m.role.len())
                .sum();
            self.messages = self.messages.split_off(len - retain);
            evaluate_reclamation(&self.memory_config, released_bytes);
        }
    }

    pub fn compact_with_summary(&mut self, summary: &str, language_marker: Option<&str>) {
        let len = self.messages.len();
        let retain = len / 2;
        let released_bytes: usize = self.messages[..len - retain]
            .iter()
            .map(|m| m.content.len() + m.role.len())
            .sum();
        let recent = self.messages.split_off(len - retain);

        let mut summary_text = String::new();
        if let Some(lang) = language_marker {
            summary_text.push_str(&format!("[User Language: {}]\n", lang));
        }
        summary_text.push_str("[Conversation Summary]\n");
        summary_text.push_str(summary);

        self.messages = vec![ChatMessage::system(summary_text)];
        self.messages.extend(recent);
        evaluate_reclamation(&self.memory_config, released_bytes);
    }

    pub fn needs_compact(&self, context_window: usize) -> bool {
        self.needs_compact_with_ratio(context_window, 0.8)
    }

    pub fn needs_compact_with_ratio(&self, context_window: usize, trigger_ratio: f64) -> bool {
        let estimate = self.token_estimate();
        let threshold = (context_window as f64 * trigger_ratio) as usize;
        estimate > threshold
    }

    pub fn detect_user_language(&self) -> Option<String> {
        for msg in self.messages.iter().rev() {
            if msg.role == "user" && !msg.content.is_empty() {
                let content = &msg.content;
                let cjk_count = content
                    .chars()
                    .filter(|c| {
                        ('\u{4E00}'..='\u{9FFF}').contains(c)
                            || ('\u{3400}'..='\u{4DBF}').contains(c)
                            || ('\u{F900}'..='\u{FAFF}').contains(c)
                    })
                    .count();
                let jp_count = content
                    .chars()
                    .filter(|c| {
                        ('\u{3040}'..='\u{309F}').contains(c)
                            || ('\u{30A0}'..='\u{30FF}').contains(c)
                    })
                    .count();
                let kr_count = content
                    .chars()
                    .filter(|c| ('\u{AC00}'..='\u{D7AF}').contains(c))
                    .count();
                let total_chars = content.chars().count();
                if total_chars == 0 {
                    continue;
                }
                if jp_count > 0 {
                    return Some("Japanese".to_string());
                }
                if kr_count > 0 {
                    return Some("Korean".to_string());
                }
                if cjk_count as f64 / total_chars as f64 > 0.1 {
                    return Some("Chinese".to_string());
                }
                return Some("English".to_string());
            }
        }
        None
    }

    pub fn reset(&mut self) -> Result<()> {
        let released_bytes: usize = self
            .messages
            .iter()
            .map(|m| m.content.len() + m.role.len())
            .sum();
        self.messages.clear();
        if self.history_path.exists() {
            std::fs::remove_file(&self.history_path)
                .with_context(|| "Failed to delete history file")?;
        }
        if released_bytes > 0 {
            evaluate_reclamation(&self.memory_config, released_bytes);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_save_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("Hello"));
        history.push(ChatMessage::assistant("Hi there!"));
        history.save().unwrap();

        let loaded = ChatHistory::load(dir.path()).unwrap();
        assert_eq!(loaded.messages().len(), 2);
        assert_eq!(loaded.messages()[0].role, "user");
        assert_eq!(loaded.messages()[0].content, "Hello");
        assert_eq!(loaded.messages()[1].role, "assistant");
        assert_eq!(loaded.messages()[1].content, "Hi there!");
    }

    #[test]
    fn test_load_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let history = ChatHistory::load(dir.path()).unwrap();
        assert!(history.messages().is_empty());
    }

    #[test]
    fn test_token_estimate() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();

        history.push(ChatMessage::user("a".repeat(2000)));
        history.push(ChatMessage::assistant("b".repeat(2000)));
        assert_eq!(history.token_estimate(), 1000);
    }

    #[test]
    fn test_compact() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..20 {
            history.push(ChatMessage::user(format!("message {i}")));
        }

        history.compact(1);
        assert_eq!(history.messages().len(), 10);

        assert_eq!(history.messages()[0].content, "message 10");
    }

    #[test]
    fn test_reset() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("Hello"));
        history.save().unwrap();
        assert!(dir.path().join("history.json.lz4").exists());

        history.reset().unwrap();
        assert!(history.messages().is_empty());
        assert!(!dir.path().join("history.json.lz4").exists());
    }

    #[test]
    fn test_compact_with_summary() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..20 {
            history.push(ChatMessage::user(format!("message {i}")));
        }

        history.compact_with_summary("The user discussed topics 0-9.", None);

        assert_eq!(history.messages().len(), 11);
        assert_eq!(history.messages()[0].role, "system");
        assert!(
            history.messages()[0]
                .content
                .contains("[Conversation Summary]")
        );
        assert!(
            history.messages()[0]
                .content
                .contains("The user discussed topics 0-9.")
        );
        assert_eq!(history.messages()[1].content, "message 10");
        assert_eq!(history.messages()[10].content, "message 19");
    }

    #[test]
    fn test_compact_with_summary_language_marker() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..10 {
            history.push(ChatMessage::user(format!("message {i}")));
        }

        history.compact_with_summary("Summary text", Some("Chinese"));

        let summary_msg = &history.messages()[0];
        assert_eq!(summary_msg.role, "system");
        assert!(summary_msg.content.contains("[User Language: Chinese]"));
        assert!(summary_msg.content.contains("[Conversation Summary]"));
        assert!(summary_msg.content.contains("Summary text"));
    }

    #[test]
    fn test_detect_user_language_english() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("Hello, how are you?"));
        assert_eq!(history.detect_user_language(), Some("English".to_string()));
    }

    #[test]
    fn test_detect_user_language_chinese() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("你好，今天天氣如何？"));
        assert_eq!(history.detect_user_language(), Some("Chinese".to_string()));
    }

    #[test]
    fn test_detect_user_language_japanese() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("こんにちは、元気ですか？"));
        assert_eq!(history.detect_user_language(), Some("Japanese".to_string()));
    }

    #[test]
    fn test_needs_compact_with_ratio() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        history.push(ChatMessage::user("a".repeat(4000)));
        assert!(history.needs_compact_with_ratio(1000, 0.5));
        assert!(!history.needs_compact_with_ratio(10000, 0.5));
    }

    #[test]
    fn test_language_marker_persists_after_compact() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..10 {
            history.push(ChatMessage::user(format!("你好 {i}")));
            history.push(ChatMessage::assistant(format!("回覆 {i}")));
        }

        let lang = history.detect_user_language();
        assert_eq!(lang.as_deref(), Some("Chinese"));

        history.compact_with_summary("用戶討論了多個話題", lang.as_deref());

        let first = &history.messages()[0];
        assert!(first.content.contains("[User Language: Chinese]"));

        history.push(ChatMessage::user("繼續聊天"));
        history.push(ChatMessage::assistant("好的"));

        let marker_present = history
            .messages()
            .iter()
            .any(|m| m.content.contains("[User Language: Chinese]"));
        assert!(marker_present);
    }

    #[test]
    fn test_compact_without_language_marker() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..10 {
            history.push(ChatMessage::user(format!("hello {i}")));
        }

        history.compact_with_summary("User greeted multiple times", None);

        let first = &history.messages()[0];
        assert!(!first.content.contains("[User Language:"));
        assert!(first.content.contains("[Conversation Summary]"));
    }

    #[test]
    fn test_truncate_fallback_after_compact_no_summary() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = ChatHistory::load(dir.path()).unwrap();
        for i in 0..20 {
            history.push(ChatMessage::user(format!("msg {i}")));
        }

        history.compact(1);
        assert_eq!(history.messages().len(), 10);
        assert_eq!(history.messages()[0].content, "msg 10");
    }
}
