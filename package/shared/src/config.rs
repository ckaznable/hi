use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    #[serde(rename = "openai-compatible")]
    OpenAICompatible,
    Anthropic,
    Gemini,
    Ollama,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::OpenAI => write!(f, "openai"),
            Provider::OpenAICompatible => write!(f, "openai-compatible"),
            Provider::Anthropic => write!(f, "anthropic"),
            Provider::Gemini => write!(f, "gemini"),
            Provider::Ollama => write!(f, "ollama"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ModelRef {
    Named(String),
    Inline(Box<SmallModelConfig>),
}

impl Default for ModelRef {
    fn default() -> Self {
        Self::Named("default".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmallModelConfig {
    pub provider: Provider,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_base: Option<String>,
    pub context_window: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval_secs")]
    pub interval_secs: u64,
    #[serde(default)]
    pub model: Option<ModelRef>,
    #[serde(default)]
    pub prompt: Option<String>,
}

fn default_interval_secs() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTaskConfig {
    pub name: String,
    pub cron: String,
    #[serde(default)]
    pub model: Option<ModelRef>,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CompactStrategy {
    Truncate,
    SmallModel,
}

impl Default for CompactStrategy {
    fn default() -> Self {
        Self::Truncate
    }
}

fn default_trigger_ratio() -> f64 {
    0.8
}

fn default_poll_timeout_secs() -> Option<u32> {
    Some(30)
}

fn default_large_release_threshold_bytes() -> usize {
    1_048_576 // 1 MB
}

fn default_session_ttl_secs() -> u64 {
    3600
}

fn default_max_sessions() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default = "default_poll_timeout_secs")]
    pub poll_timeout_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_session_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_large_release_threshold_bytes")]
    pub large_release_threshold_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    #[serde(default)]
    pub telegram: Option<TelegramConfig>,
    #[serde(default)]
    pub session: Option<SessionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub strategy: CompactStrategy,
    #[serde(default = "default_trigger_ratio")]
    pub trigger_ratio: f64,
    #[serde(default)]
    pub model: Option<ModelRef>,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: Provider,
    pub model: String,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub preamble: Option<String>,
    pub context_window: usize,
    #[serde(default)]
    pub small_model: Option<SmallModelConfig>,
    #[serde(default)]
    pub heartbeat: Option<HeartbeatConfig>,
    #[serde(default)]
    pub schedules: Option<Vec<ScheduleTaskConfig>>,
    #[serde(default)]
    pub compact: Option<CompactConfig>,
    #[serde(default)]
    pub remote: Option<RemoteConfig>,
    #[serde(default)]
    pub memory: Option<MemoryConfig>,
}

const CONFIG_TEMPLATE: &str = r#"{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxxx",
  "context_window": 128000
}
"#;

/// Resolve the full path to the config file: `config_dir()/config.json`.
pub fn config_path() -> Result<std::path::PathBuf> {
    Ok(crate::paths::config_dir()?.join("config.json"))
}

/// Create a starter config template at the default config path.
///
/// Returns the resolved path on success.
/// Fails if the config file already exists or if the filesystem write fails.
pub fn init_config() -> Result<std::path::PathBuf> {
    let path = config_path()?;
    write_config_template(&path)?;
    Ok(path)
}

fn write_config_template(path: &std::path::Path) -> Result<()> {
    if path.exists() {
        bail!(
            "Config file already exists at: {}\nRemove it first if you want to regenerate.",
            path.display()
        );
    }

    std::fs::write(path, CONFIG_TEMPLATE)
        .with_context(|| format!("Failed to write config template to: {}", path.display()))?;

    Ok(())
}

impl ModelConfig {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Config file not found at: {}", path.display()))?;
        let config: ModelConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config at: {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if !matches!(self.provider, Provider::Ollama | Provider::OpenAICompatible)
            && self.api_key.is_none()
        {
            bail!("api_key is required for provider {:?}", self.provider);
        }
        if let Some(ref small) = self.small_model {
            if !matches!(
                small.provider,
                Provider::Ollama | Provider::OpenAICompatible
            ) && small.api_key.is_none()
            {
                bail!(
                    "api_key is required for small_model provider {:?}",
                    small.provider
                );
            }
        }
        if let Some(ref remote) = self.remote {
            if let Some(ref telegram) = remote.telegram {
                if telegram.enabled && telegram.bot_token.is_empty() {
                    bail!("bot_token is required when Telegram remote is enabled");
                }
            }
        }
        Ok(())
    }

    pub fn resolve_model_ref(&self, model_ref: &Option<ModelRef>) -> SmallModelConfig {
        match model_ref {
            None => self.as_small_model_config(),
            Some(ModelRef::Named(name)) => match name.as_str() {
                "small" => self
                    .small_model
                    .clone()
                    .unwrap_or_else(|| self.as_small_model_config()),
                _ => self.as_small_model_config(),
            },
            Some(ModelRef::Inline(config)) => *config.clone(),
        }
    }

    pub fn as_small_model_config(&self) -> SmallModelConfig {
        SmallModelConfig {
            provider: self.provider.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
            api_base: self.api_base.clone(),
            context_window: self.context_window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_config() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, Provider::OpenAI);
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.api_key.as_deref(), Some("sk-test"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_parse_ollama_config_no_api_key() {
        let json = r#"{
            "provider": "ollama",
            "model": "qwen2.5:14b",
            "context_window": 32000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, Provider::Ollama);
        assert!(config.api_key.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_non_ollama_without_api_key_fails() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_unknown_provider_fails() {
        let json = r#"{
            "provider": "unknown_provider",
            "model": "test",
            "context_window": 4096
        }"#;
        let result: Result<ModelConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_anthropic_config() {
        let json = r#"{
            "provider": "anthropic",
            "model": "claude-3-5-sonnet",
            "api_key": "sk-ant-test",
            "preamble": "You are a helpful assistant.",
            "context_window": 200000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, Provider::Anthropic);
        assert_eq!(
            config.preamble.as_deref(),
            Some("You are a helpful assistant.")
        );
    }

    #[test]
    fn test_config_with_small_model() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "small_model": {
                "provider": "ollama",
                "model": "qwen2.5:3b",
                "context_window": 4096
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.small_model.is_some());
        let small = config.small_model.as_ref().unwrap();
        assert_eq!(small.provider, Provider::Ollama);
        assert_eq!(small.model, "qwen2.5:3b");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_heartbeat() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "heartbeat": {
                "enabled": true,
                "interval_secs": 300,
                "model": "small",
                "prompt": "Status check"
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.heartbeat.is_some());
        let hb = config.heartbeat.as_ref().unwrap();
        assert!(hb.enabled);
        assert_eq!(hb.interval_secs, 300);
    }

    #[test]
    fn test_config_with_schedules() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "schedules": [
                {
                    "name": "daily-summary",
                    "cron": "0 0 * * *",
                    "model": "small",
                    "prompt": "Generate a daily summary."
                }
            ]
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.schedules.is_some());
        let schedules = config.schedules.as_ref().unwrap();
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].name, "daily-summary");
    }

    #[test]
    fn test_resolve_model_ref_default() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let resolved = config.resolve_model_ref(&None);
        assert_eq!(resolved.provider, Provider::OpenAI);
        assert_eq!(resolved.model, "gpt-4o");
    }

    #[test]
    fn test_resolve_model_ref_small() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "small_model": {
                "provider": "ollama",
                "model": "qwen2.5:3b",
                "context_window": 4096
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let model_ref = Some(ModelRef::Named("small".to_string()));
        let resolved = config.resolve_model_ref(&model_ref);
        assert_eq!(resolved.provider, Provider::Ollama);
        assert_eq!(resolved.model, "qwen2.5:3b");
    }

    #[test]
    fn test_resolve_model_ref_small_fallback() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let model_ref = Some(ModelRef::Named("small".to_string()));
        let resolved = config.resolve_model_ref(&model_ref);
        assert_eq!(resolved.provider, Provider::OpenAI);
        assert_eq!(resolved.model, "gpt-4o");
    }

    #[test]
    fn test_resolve_model_ref_inline() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let inline = SmallModelConfig {
            provider: Provider::Gemini,
            model: "gemini-pro".to_string(),
            api_key: Some("gkey".to_string()),
            api_base: None,
            context_window: 32000,
        };
        let model_ref = Some(ModelRef::Inline(Box::new(inline)));
        let resolved = config.resolve_model_ref(&model_ref);
        assert_eq!(resolved.provider, Provider::Gemini);
        assert_eq!(resolved.model, "gemini-pro");
    }

    #[test]
    fn test_parse_openai_compatible_config() {
        let json = r#"{
            "provider": "openai-compatible",
            "model": "gpt-4o-mini",
            "api_base": "http://localhost:11434/v1",
            "context_window": 32000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, Provider::OpenAICompatible);
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.api_key.is_none());
        assert_eq!(
            config.api_base.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_openai_compatible_with_api_key() {
        let json = r#"{
            "provider": "openai-compatible",
            "model": "gpt-4o-mini",
            "api_key": "test-key",
            "api_base": "https://gateway.example.com/v1",
            "context_window": 32000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.provider, Provider::OpenAICompatible);
        assert_eq!(config.api_key.as_deref(), Some("test-key"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_openai_compatible_small_model_no_api_key() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "small_model": {
                "provider": "openai-compatible",
                "model": "local-model",
                "api_base": "http://localhost:8080/v1",
                "context_window": 4096
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_ok());
        let small = config.small_model.as_ref().unwrap();
        assert_eq!(small.provider, Provider::OpenAICompatible);
        assert!(small.api_key.is_none());
    }

    #[test]
    fn test_resolve_model_ref_openai_compatible_small() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "small_model": {
                "provider": "openai-compatible",
                "model": "local-small",
                "api_base": "http://localhost:8080/v1",
                "context_window": 4096
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let model_ref = Some(ModelRef::Named("small".to_string()));
        let resolved = config.resolve_model_ref(&model_ref);
        assert_eq!(resolved.provider, Provider::OpenAICompatible);
        assert_eq!(resolved.model, "local-small");
        assert_eq!(
            resolved.api_base.as_deref(),
            Some("http://localhost:8080/v1")
        );
    }

    #[test]
    fn test_resolve_model_ref_inline_openai_compatible() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let inline = SmallModelConfig {
            provider: Provider::OpenAICompatible,
            model: "custom-model".to_string(),
            api_key: None,
            api_base: Some("http://custom:9000/v1".to_string()),
            context_window: 8192,
        };
        let model_ref = Some(ModelRef::Inline(Box::new(inline)));
        let resolved = config.resolve_model_ref(&model_ref);
        assert_eq!(resolved.provider, Provider::OpenAICompatible);
        assert_eq!(resolved.model, "custom-model");
        assert_eq!(resolved.api_base.as_deref(), Some("http://custom:9000/v1"));
    }

    #[test]
    fn test_config_without_compact() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.compact.is_none());
    }

    #[test]
    fn test_config_with_compact_small_model() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "compact": {
                "enabled": true,
                "strategy": "small-model",
                "trigger_ratio": 0.8,
                "model": "small",
                "prompt": "Summarize earlier context"
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let compact = config.compact.as_ref().unwrap();
        assert!(compact.enabled);
        assert_eq!(compact.strategy, CompactStrategy::SmallModel);
        assert!((compact.trigger_ratio - 0.8).abs() < f64::EPSILON);
        assert_eq!(compact.model, Some(ModelRef::Named("small".to_string())));
        assert_eq!(compact.prompt.as_deref(), Some("Summarize earlier context"));
    }

    #[test]
    fn test_config_with_compact_truncate() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "compact": {
                "enabled": true,
                "strategy": "truncate"
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let compact = config.compact.as_ref().unwrap();
        assert!(compact.enabled);
        assert_eq!(compact.strategy, CompactStrategy::Truncate);
        assert!((compact.trigger_ratio - 0.8).abs() < f64::EPSILON);
        assert!(compact.model.is_none());
        assert!(compact.prompt.is_none());
    }

    #[test]
    fn test_compact_defaults_when_minimal() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "compact": {
                "enabled": true
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let compact = config.compact.as_ref().unwrap();
        assert!(compact.enabled);
        assert_eq!(compact.strategy, CompactStrategy::Truncate);
        assert!((compact.trigger_ratio - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_without_remote() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.remote.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_telegram_enabled() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "remote": {
                "telegram": {
                    "enabled": true,
                    "bot_token": "123456:ABC-DEF"
                }
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let telegram = config.remote.as_ref().unwrap().telegram.as_ref().unwrap();
        assert!(telegram.enabled);
        assert_eq!(telegram.bot_token, "123456:ABC-DEF");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_with_telegram_disabled() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "remote": {
                "telegram": {
                    "enabled": false
                }
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let telegram = config.remote.as_ref().unwrap().telegram.as_ref().unwrap();
        assert!(!telegram.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_telegram_enabled_missing_token_fails() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "remote": {
                "telegram": {
                    "enabled": true
                }
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_telegram_with_poll_timeout() {
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "api_key": "sk-test",
            "context_window": 128000,
            "remote": {
                "telegram": {
                    "enabled": true,
                    "bot_token": "123456:ABC-DEF",
                    "poll_timeout_secs": 60
                }
            }
        }"#;
        let config: ModelConfig = serde_json::from_str(json).unwrap();
        let telegram = config.remote.as_ref().unwrap().telegram.as_ref().unwrap();
        assert_eq!(telegram.poll_timeout_secs, Some(60));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_write_config_template_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        write_config_template(&path).unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: ModelConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.provider, Provider::OpenAI);
        assert_eq!(parsed.model, "gpt-4o");
        assert_eq!(parsed.api_key.as_deref(), Some("sk-xxxx"));
        assert_eq!(parsed.context_window, 128000);
    }

    #[test]
    fn test_write_config_template_existing_file_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        std::fs::write(&path, "existing content").unwrap();

        let result = write_config_template(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already exists"));

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "existing content");
    }

    #[test]
    fn test_write_config_template_write_failure() {
        let path = std::path::PathBuf::from("/nonexistent_root_dir/config.json");

        let result = write_config_template(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to write config template"));
    }

    #[test]
    fn test_config_template_is_valid_json() {
        let parsed: serde_json::Value = serde_json::from_str(CONFIG_TEMPLATE).unwrap();
        assert!(parsed.get("provider").is_some());
        assert!(parsed.get("model").is_some());
        assert!(parsed.get("api_key").is_some());
        assert!(parsed.get("context_window").is_some());
    }

    #[test]
    fn test_load_from_path_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let json = r#"{
            "provider": "ollama",
            "model": "qwen2.5:14b",
            "context_window": 32000
        }"#;
        std::fs::write(&path, json).unwrap();

        let config = ModelConfig::load_from_path(&path).unwrap();
        assert_eq!(config.provider, Provider::Ollama);
        assert_eq!(config.model, "qwen2.5:14b");
    }

    #[test]
    fn test_load_from_path_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = ModelConfig::load_from_path(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("nonexistent.json"));
    }

    #[test]
    fn test_load_from_path_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.json");
        std::fs::write(&path, "not valid json").unwrap();

        let result = ModelConfig::load_from_path(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("bad.json"));
    }

    #[test]
    fn test_load_from_path_validation_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let json = r#"{
            "provider": "openai",
            "model": "gpt-4o",
            "context_window": 128000
        }"#;
        std::fs::write(&path, json).unwrap();

        let result = ModelConfig::load_from_path(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("api_key"));
    }
}
