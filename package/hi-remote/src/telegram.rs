use std::sync::Arc;

use anyhow::Result;
use shared::config::{ModelConfig, TelegramConfig};
use teloxide::Bot;
use teloxide::RequestError;
use teloxide::payloads::{GetUpdatesSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{
    AllowedUpdate, ChatAction, ChatId, MediaKind, MessageKind, ParseMode, UpdateKind,
};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::session_manager::SessionManager;

// Telegram API: 4096 UTF-8 chars per message
const MAX_MESSAGE_LENGTH: usize = 4096;
const MAX_RETRY_ATTEMPTS: u32 = 3;
const TYPING_INTERVAL_SECS: u64 = 5;

pub async fn run_polling_loop(
    config: &ModelConfig,
    telegram_config: &TelegramConfig,
) -> Result<()> {
    let bot = Bot::new(&telegram_config.bot_token);
    let session_manager = Arc::new(SessionManager::new(config.clone()));
    let allowed_user_ids = telegram_config.allowed_user_ids.clone();

    let timeout = telegram_config.poll_timeout_secs.unwrap_or(30);
    let mut offset: i32 = 0;

    info!("Telegram adapter started. Polling for updates...");

    loop {
        match bot
            .get_updates()
            .offset(offset)
            .timeout(timeout)
            .allowed_updates([AllowedUpdate::Message])
            .await
        {
            Ok(updates) => {
                for update in updates {
                    offset = update.id.0 as i32 + 1;

                    if let UpdateKind::Message(message) = update.kind {
                        if let Some(ref allowed) = allowed_user_ids {
                            let sender_id = message.from.as_ref().map(|u| u.id.0 as i64);
                            match sender_id {
                                Some(id) if allowed.contains(&id) => {}
                                _ => {
                                    warn!(
                                        chat_id = message.chat.id.0,
                                        sender_id = ?sender_id,
                                        "Rejected message from unauthorized user"
                                    );
                                    continue;
                                }
                            }
                        }

                        let text = match &message.kind {
                            MessageKind::Common(common) => match &common.media_kind {
                                MediaKind::Text(text_media) => text_media.text.clone(),
                                _ => continue,
                            },
                            _ => continue,
                        };

                        let chat_id = message.chat.id.0;
                        let bot_clone = bot.clone();
                        let manager = Arc::clone(&session_manager);

                        tokio::spawn(async move {
                            if let Err(e) =
                                handle_message(chat_id, &text, &bot_clone, &manager).await
                            {
                                error!(chat_id, "Error handling message: {e}");
                            }
                        });
                    }
                }
            }
            Err(RequestError::Network(_)) => {
                continue;
            }
            Err(e) => {
                error!("Failed to get updates: {e:?}");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn handle_message(
    chat_id: i64,
    text: &str,
    bot: &Bot,
    session_manager: &SessionManager,
) -> Result<()> {
    if let Some(command) = text.strip_prefix('/') {
        return handle_command(chat_id, command.trim(), bot, session_manager).await;
    }

    // Send initial typing indicator and spawn periodic re-send
    let typing_handle = spawn_typing_indicator(bot.clone(), chat_id);

    let (stream_tx, mut stream_rx) =
        mpsc::channel::<String>(hi_core::provider::STREAM_CHANNEL_CAPACITY);

    let aggregator = tokio::spawn(async move {
        let mut buffer = String::new();
        while let Some(chunk) = stream_rx.recv().await {
            buffer.push_str(&chunk);
        }
        buffer
    });

    let session = session_manager.get_or_create(chat_id).await?;
    let result = {
        let mut session = session.lock().await;
        session.send_message_streaming(text, stream_tx).await
    };

    let aggregated = aggregator.await?;

    // Stop typing indicator before sending reply
    typing_handle.abort();

    let reply_text = match result {
        Ok(final_text) if !final_text.is_empty() => final_text,
        Ok(_) => aggregated,
        Err(e) => {
            let error_msg = format!("Error: {e}");
            send_message_with_retry(bot, chat_id, &error_msg).await?;
            return Err(e);
        }
    };

    if reply_text.is_empty() {
        return Ok(());
    }

    let chunks = split_message(&reply_text);
    for chunk in chunks {
        let escaped = escape_markdown_v2(&chunk);
        send_message_with_retry(bot, chat_id, &escaped).await?;
    }

    Ok(())
}

fn spawn_typing_indicator(bot: Bot, chat_id: i64) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Err(e) = bot
                .send_chat_action(ChatId(chat_id), ChatAction::Typing)
                .await
            {
                warn!(chat_id, "Failed to send typing indicator: {e}");
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(TYPING_INTERVAL_SECS)).await;
        }
    })
}

async fn handle_command(
    chat_id: i64,
    command: &str,
    bot: &Bot,
    session_manager: &SessionManager,
) -> Result<()> {
    let (cmd, args) = match command.split_once(char::is_whitespace) {
        Some((c, a)) => (c, a.trim()),
        None => (command, ""),
    };

    let reply = match cmd {
        "compact" => match session_manager.compact_session(chat_id).await {
            Ok(true) => "✓ History compacted.".to_string(),
            Ok(false) => "Nothing to compact.".to_string(),
            Err(e) => format!("Failed to compact: {e}"),
        },
        "new" => match session_manager.reset_session(chat_id).await {
            Ok(true) => "✓ Conversation reset.".to_string(),
            Ok(false) => "No active conversation to reset.".to_string(),
            Err(e) => format!("Failed to reset: {e}"),
        },
        "cron" => handle_cron_command(args, session_manager.config()),
        "heartbeat" => format_heartbeat(session_manager.config().heartbeat.as_ref()),
        "mcp" => format_mcp_servers(&shared::mcp_store::load()),
        "skills" => format_skills(),
        "help" => concat!(
            "Available commands:\n",
            "/compact - Compact chat history\n",
            "/new - Start a new conversation\n",
            "/cron - List scheduled tasks\n",
            "/cron add <name> <cron> <prompt> - Add a schedule\n",
            "/cron remove <name> - Remove a schedule\n",
            "/heartbeat - Show heartbeat status\n",
            "/mcp - List MCP servers\n",
            "/skills - List loaded skills\n",
            "/help - Show this help message",
        )
        .to_string(),
        _ => format!("Unknown command: /{cmd}\nUse /help to see available commands."),
    };

    send_message_with_retry(bot, chat_id, &reply).await?;
    Ok(())
}

fn handle_cron_command(args: &str, config: &ModelConfig) -> String {
    if args.is_empty() {
        return format_schedules(&shared::schedule_store::load(config.schedules.as_deref()));
    }

    let (sub, sub_args) = match args.split_once(char::is_whitespace) {
        Some((s, a)) => (s, a.trim()),
        None => (args, ""),
    };

    match sub {
        "add" => handle_cron_add(sub_args, config),
        "remove" => handle_cron_remove(sub_args, config),
        _ => {
            "Usage:\n/cron - List schedules\n/cron add <name> <cron> <prompt>\n/cron remove <name>"
                .to_string()
        }
    }
}

fn handle_cron_add(args: &str, config: &ModelConfig) -> String {
    let parts: Vec<&str> = args.splitn(7, char::is_whitespace).collect();
    if parts.len() < 7 {
        return "Usage: /cron add <name> <min> <hour> <dom> <mon> <dow> <prompt>\nExample: /cron add daily-summary 0 0 * * * Generate a daily summary.".to_string();
    }

    let name = parts[0];
    let cron_expr = format!(
        "{} {} {} {} {}",
        parts[1], parts[2], parts[3], parts[4], parts[5]
    );
    let prompt = parts[6].trim();

    if name.is_empty() || prompt.is_empty() {
        return "Name and prompt must not be empty.".to_string();
    }

    let mut schedules = shared::schedule_store::load(config.schedules.as_deref());

    if schedules.iter().any(|s| s.name == name) {
        return format!("Schedule '{name}' already exists. Remove it first to replace.");
    }

    let is_first_schedule = schedules.is_empty();
    let auto_enable = !schedules.iter().any(|s| s.enabled);

    schedules.push(shared::config::ScheduleTaskConfig {
        name: name.to_string(),
        cron: cron_expr.clone(),
        model: None,
        prompt: prompt.to_string(),
        enabled: auto_enable,
    });

    match shared::schedule_store::save(&schedules) {
        Ok(()) => {
            let msg = if auto_enable && is_first_schedule {
                format!(
                    "✓ Added schedule '{name}' ({cron_expr}).\nSchedule auto-enabled. Restart to activate."
                )
            } else {
                format!(
                    "✓ Added schedule '{name}' ({cron_expr}).\nNote: restart required for schedule to take effect."
                )
            };
            msg
        }
        Err(e) => format!("Failed to save schedule: {e}"),
    }
}

fn handle_cron_remove(args: &str, config: &ModelConfig) -> String {
    let name = args.split_whitespace().next().unwrap_or("");
    if name.is_empty() {
        return "Usage: /cron remove <name>".to_string();
    }

    let mut schedules = shared::schedule_store::load(config.schedules.as_deref());
    let before = schedules.len();
    schedules.retain(|s| s.name != name);

    if schedules.len() == before {
        return format!("Not found: {name}");
    }

    match shared::schedule_store::save(&schedules) {
        Ok(()) => format!(
            "✓ Removed schedule '{name}'.\nNote: restart required for change to take effect."
        ),
        Err(e) => format!("Failed to save: {e}"),
    }
}

fn format_schedules(schedules: &[shared::config::ScheduleTaskConfig]) -> String {
    if schedules.is_empty() {
        return "No schedules configured.".to_string();
    }

    let mut lines = vec!["Schedules:".to_string()];
    for s in schedules {
        let model = match &s.model {
            Some(shared::config::ModelRef::Named(n)) => n.clone(),
            Some(shared::config::ModelRef::Inline(c)) => format!("{}/{}", c.provider, c.model),
            None => "default".to_string(),
        };
        let prompt_preview = if s.prompt.len() > 50 {
            format!("{}…", &s.prompt[..50])
        } else {
            s.prompt.clone()
        };
        lines.push(format!(
            "• {} | {} | model={} | {}",
            s.name, s.cron, model, prompt_preview
        ));
    }
    lines.join("\n")
}

fn format_heartbeat(config: Option<&shared::config::HeartbeatConfig>) -> String {
    let Some(hb) = config else {
        return "Heartbeat: not configured.".to_string();
    };

    let model = match &hb.model {
        Some(shared::config::ModelRef::Named(n)) => n.clone(),
        Some(shared::config::ModelRef::Inline(c)) => format!("{}/{}", c.provider, c.model),
        None => "default".to_string(),
    };
    let prompt = hb.prompt.as_deref().unwrap_or("(none)");

    format!(
        "Heartbeat:\n• enabled: {}\n• interval: {}s\n• model: {}\n• prompt: {}",
        hb.enabled, hb.interval_secs, model, prompt
    )
}

fn format_mcp_servers(config: &shared::config::McpConfig) -> String {
    if config.mcp_servers.is_empty() {
        return "MCP: no servers configured.".to_string();
    }

    let mut lines = vec!["MCP servers:".to_string()];
    let mut names: Vec<&String> = config.mcp_servers.keys().collect();
    names.sort();
    for name in names {
        let server = &config.mcp_servers[name];
        let transport = if let Some(cmd) = &server.command {
            format!("stdio ({cmd})")
        } else if let Some(url) = &server.url {
            format!("http ({url})")
        } else {
            "unknown".to_string()
        };
        lines.push(format!("• {name} — {transport}"));
    }
    lines.join("\n")
}

fn format_skills() -> String {
    let config_dir = match shared::paths::config_dir() {
        Ok(d) => d,
        Err(_) => return "Failed to locate config directory.".to_string(),
    };
    let skills = match hi_core::skills::load_skills(&config_dir) {
        Ok(s) => s,
        Err(_) => return "Failed to load skills.".to_string(),
    };
    if skills.is_empty() {
        return "No skills loaded.".to_string();
    }
    let mut lines = vec!["Loaded skills:".to_string()];
    for s in &skills {
        lines.push(format!("• {} — {}", s.name, s.description));
    }
    lines.join("\n")
}

async fn send_message_with_retry(bot: &Bot, chat_id: i64, text: &str) -> Result<()> {
    for attempt in 0..MAX_RETRY_ATTEMPTS {
        match bot
            .send_message(ChatId(chat_id), text)
            .parse_mode(ParseMode::MarkdownV2)
            .await
        {
            Ok(_) => return Ok(()),
            Err(RequestError::RetryAfter(seconds)) => {
                let retry_after = seconds.seconds();

                warn!(
                    attempt = attempt + 1,
                    max_attempts = MAX_RETRY_ATTEMPTS,
                    retry_after,
                    "Rate limited. Retrying..."
                );

                tokio::time::sleep(seconds.duration()).await;
            }
            Err(e) => {
                warn!(
                    chat_id,
                    "MarkdownV2 send failed, falling back to plain text: {e}"
                );
                return send_message_plain_with_retry(bot, chat_id, text).await;
            }
        }
    }

    anyhow::bail!(
        "Failed to send message to chat {chat_id} after {MAX_RETRY_ATTEMPTS} retry attempts"
    )
}

async fn send_message_plain_with_retry(bot: &Bot, chat_id: i64, text: &str) -> Result<()> {
    for attempt in 0..MAX_RETRY_ATTEMPTS {
        match bot.send_message(ChatId(chat_id), text).await {
            Ok(_) => return Ok(()),
            Err(RequestError::RetryAfter(seconds)) => {
                let retry_after = seconds.seconds();

                warn!(
                    attempt = attempt + 1,
                    max_attempts = MAX_RETRY_ATTEMPTS,
                    retry_after,
                    "Rate limited (plain text). Retrying..."
                );

                tokio::time::sleep(seconds.duration()).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to send plain text message to chat {chat_id}: {e}"
                ));
            }
        }
    }

    anyhow::bail!(
        "Failed to send plain text message to chat {chat_id} after {MAX_RETRY_ATTEMPTS} retry attempts"
    )
}

fn split_message(text: &str) -> Vec<String> {
    if text.chars().count() <= MAX_MESSAGE_LENGTH {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.chars().count() <= MAX_MESSAGE_LENGTH {
            chunks.push(remaining.to_string());
            break;
        }

        let byte_limit = remaining
            .char_indices()
            .nth(MAX_MESSAGE_LENGTH)
            .map(|(i, _)| i)
            .unwrap_or(remaining.len());

        let split_at = remaining[..byte_limit]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(byte_limit);

        let (chunk, rest) = remaining.split_at(split_at);
        chunks.push(chunk.to_string());
        remaining = rest;
    }

    chunks
}

fn escape_markdown_v2(text: &str) -> String {
    const SPECIAL_CHARS: &[char] = &[
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
        '\\',
    ];

    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '`' && i + 2 < len && chars[i + 1] == '`' && chars[i + 2] == '`' {
            let fence_start = i;
            i += 3;
            while i < len
                && !(chars[i] == '`' && i + 2 < len && chars[i + 1] == '`' && chars[i + 2] == '`')
            {
                i += 1;
            }
            if i < len {
                i += 3;
            }
            for &c in &chars[fence_start..i] {
                result.push(c);
            }
        } else if chars[i] == '`' {
            let tick_start = i;
            i += 1;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            if i < len {
                i += 1;
            }
            for &c in &chars[tick_start..i] {
                result.push(c);
            }
        } else if SPECIAL_CHARS.contains(&chars[i]) {
            result.push('\\');
            result.push(chars[i]);
            i += 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message_short() {
        let text = "Hello, world!";
        let chunks = split_message(text);
        assert_eq!(chunks, vec!["Hello, world!"]);
    }

    #[test]
    fn test_split_message_exact_limit() {
        let text = "a".repeat(MAX_MESSAGE_LENGTH);
        let chunks = split_message(&text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), MAX_MESSAGE_LENGTH);
    }

    #[test]
    fn test_split_message_over_limit() {
        let text = "a".repeat(MAX_MESSAGE_LENGTH + 100);
        let chunks = split_message(&text);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), MAX_MESSAGE_LENGTH);
        assert_eq!(chunks[1].len(), 100);
    }

    #[test]
    fn test_split_message_at_newline() {
        let line1 = "a".repeat(MAX_MESSAGE_LENGTH - 10);
        let line2 = "b".repeat(20);
        let text = format!("{}\n{}", line1, line2);
        let chunks = split_message(&text);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].ends_with('\n'));
        assert!(chunks[1].starts_with('b'));
    }

    #[test]
    fn test_split_message_empty() {
        let chunks = split_message("");
        assert_eq!(chunks, vec![""]);
    }

    #[test]
    fn test_split_message_preserves_order() {
        let mut text = String::new();
        for i in 0..10 {
            text.push_str(&format!("Chunk {}\n", i));
            text.push_str(&"x".repeat(MAX_MESSAGE_LENGTH / 2));
            text.push('\n');
        }
        let chunks = split_message(&text);
        let reassembled: String = chunks.join("");
        assert_eq!(reassembled, text);
    }

    #[test]
    fn test_split_message_multibyte_utf8() {
        let unit = "你好世界🎉";
        let unit_chars = unit.chars().count();
        let repeats = (MAX_MESSAGE_LENGTH / unit_chars) + 2;
        let text: String = std::iter::repeat(unit).take(repeats).collect();
        assert!(text.chars().count() > MAX_MESSAGE_LENGTH);

        let chunks = split_message(&text);
        assert!(chunks.len() >= 2);

        for chunk in &chunks {
            assert!(chunk.chars().count() <= MAX_MESSAGE_LENGTH);
        }

        let reassembled: String = chunks.join("");
        assert_eq!(reassembled, text);
    }

    #[test]
    fn test_format_schedules_empty() {
        let result = format_schedules(&[]);
        assert_eq!(result, "No schedules configured.");
    }

    #[test]
    fn test_format_schedules_with_entries() {
        let schedules = vec![
            shared::config::ScheduleTaskConfig {
                name: "daily".to_string(),
                cron: "0 0 * * *".to_string(),
                model: None,
                prompt: "Summarize the day.".to_string(),
                enabled: true,
            },
            shared::config::ScheduleTaskConfig {
                name: "check".to_string(),
                cron: "*/5 * * * *".to_string(),
                model: Some(shared::config::ModelRef::Named("small".to_string())),
                prompt: "Check status.".to_string(),
                enabled: false,
            },
        ];
        let result = format_schedules(&schedules);
        assert!(result.contains("daily"));
        assert!(result.contains("0 0 * * *"));
        assert!(result.contains("model=default"));
        assert!(result.contains("check"));
        assert!(result.contains("model=small"));
    }

    #[test]
    fn test_format_schedules_long_prompt_truncated() {
        let schedules = vec![shared::config::ScheduleTaskConfig {
            name: "verbose".to_string(),
            cron: "0 0 * * *".to_string(),
            model: None,
            prompt: "A".repeat(100),
            enabled: true,
        }];
        let result = format_schedules(&schedules);
        assert!(result.contains("…"));
        assert!(!result.contains(&"A".repeat(100)));
    }

    #[test]
    fn test_format_heartbeat_none() {
        let result = format_heartbeat(None);
        assert_eq!(result, "Heartbeat: not configured.");
    }

    #[test]
    fn test_format_heartbeat_configured() {
        let hb = shared::config::HeartbeatConfig {
            enabled: true,
            interval_secs: 1200,
            model: Some(shared::config::ModelRef::Named("small".to_string())),
            prompt: Some("heartbeat check".to_string()),
        };
        let result = format_heartbeat(Some(&hb));
        assert!(result.contains("enabled: true"));
        assert!(result.contains("interval: 1200s"));
        assert!(result.contains("model: small"));
        assert!(result.contains("prompt: heartbeat check"));
    }

    #[test]
    fn test_format_heartbeat_disabled_no_prompt() {
        let hb = shared::config::HeartbeatConfig {
            enabled: false,
            interval_secs: 300,
            model: None,
            prompt: None,
        };
        let result = format_heartbeat(Some(&hb));
        assert!(result.contains("enabled: false"));
        assert!(result.contains("model: default"));
        assert!(result.contains("prompt: (none)"));
    }

    #[test]
    fn test_format_mcp_servers_empty() {
        let config = shared::config::McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        };
        let result = format_mcp_servers(&config);
        assert_eq!(result, "MCP: no servers configured.");
    }

    #[test]
    fn test_format_mcp_servers_with_entries() {
        let mut servers = std::collections::HashMap::new();
        servers.insert(
            "filesystem".to_string(),
            shared::config::McpServerConfig {
                command: Some("npx".to_string()),
                args: Some(vec!["-y".to_string(), "server".to_string()]),
                env: None,
                url: None,
            },
        );
        servers.insert(
            "remote".to_string(),
            shared::config::McpServerConfig {
                command: None,
                args: None,
                env: None,
                url: Some("http://localhost:8080/mcp".to_string()),
            },
        );
        let config = shared::config::McpConfig {
            mcp_servers: servers,
        };
        let result = format_mcp_servers(&config);
        assert!(result.contains("filesystem — stdio (npx)"));
        assert!(result.contains("remote — http (http://localhost:8080/mcp)"));
    }

    #[test]
    fn test_handle_cron_command_empty_args_lists() {
        let config = make_model_config(None);
        let result = handle_cron_command("", &config);
        assert_eq!(result, "No schedules configured.");
    }

    #[test]
    fn test_handle_cron_command_invalid_sub() {
        let config = make_model_config(None);
        let result = handle_cron_command("invalid", &config);
        assert!(result.contains("Usage:"));
    }

    #[test]
    fn test_handle_cron_add_missing_args() {
        let config = make_model_config(None);
        let result = handle_cron_add("daily 0 0 * *", &config);
        assert!(result.contains("Usage:"));
    }

    #[test]
    fn test_handle_cron_remove_missing_name() {
        let config = make_model_config(None);
        let result = handle_cron_remove("", &config);
        assert!(result.contains("Usage:"));
    }

    fn make_model_config(
        schedules: Option<Vec<shared::config::ScheduleTaskConfig>>,
    ) -> ModelConfig {
        let json_str = r#"{
            "provider": "ollama",
            "model": "qwen2.5:14b",
            "context_window": 32000
        }"#;
        let mut config: ModelConfig = serde_json::from_str(json_str).unwrap();
        config.schedules = schedules;
        config
    }

    #[test]
    fn test_escape_markdown_v2_plain_text() {
        assert_eq!(
            escape_markdown_v2("Hello! How are you."),
            r"Hello\! How are you\."
        );
    }

    #[test]
    fn test_escape_markdown_v2_special_chars() {
        assert_eq!(escape_markdown_v2("a_b*c[d]e"), r"a\_b\*c\[d\]e");
        assert_eq!(escape_markdown_v2("(x)~y>z"), r"\(x\)\~y\>z");
        assert_eq!(escape_markdown_v2("a+b-c=d"), r"a\+b\-c\=d");
        assert_eq!(escape_markdown_v2("{a|b}"), r"\{a\|b\}");
        assert_eq!(escape_markdown_v2("1.2!3"), r"1\.2\!3");
        assert_eq!(escape_markdown_v2(r"a\b"), r"a\\b");
    }

    #[test]
    fn test_escape_markdown_v2_code_block_preserved() {
        let input = "Hello!\n```rust\nfn main() { println!(\"!\"); }\n```\nDone.";
        let result = escape_markdown_v2(input);
        assert!(result.contains("```rust\nfn main() { println!(\"!\"); }\n```"));
        assert!(result.starts_with(r"Hello\!"));
        assert!(result.ends_with(r"Done\."));
    }

    #[test]
    fn test_escape_markdown_v2_inline_code_preserved() {
        let input = "Use `vec![1, 2]` here.";
        let result = escape_markdown_v2(input);
        assert!(result.contains("`vec![1, 2]`"));
        assert!(result.contains(r"here\."));
    }

    #[test]
    fn test_escape_markdown_v2_mixed_content() {
        let input = "Hello! `code.here` and more. ```\nblock!\n``` end!";
        let result = escape_markdown_v2(input);
        assert!(result.contains(r"Hello\!"));
        assert!(result.contains("`code.here`"));
        assert!(result.contains("```\nblock!\n```"));
        assert!(result.contains(r"end\!"));
    }

    #[test]
    fn test_escape_markdown_v2_no_special_chars() {
        assert_eq!(escape_markdown_v2("Hello world"), "Hello world");
    }

    #[test]
    fn test_escape_markdown_v2_empty() {
        assert_eq!(escape_markdown_v2(""), "");
    }

    #[test]
    fn test_escape_markdown_v2_unclosed_code_block() {
        let input = "Hello! ```code without closing";
        let result = escape_markdown_v2(input);
        assert!(result.contains("```code without closing"));
        assert!(result.starts_with(r"Hello\!"));
    }

    #[test]
    fn test_escape_markdown_v2_unclosed_inline_code() {
        let input = "Hello! `unclosed code";
        let result = escape_markdown_v2(input);
        assert!(result.contains("`unclosed code"));
        assert!(result.starts_with(r"Hello\!"));
    }

    #[test]
    fn test_escape_markdown_v2_hash_at_start() {
        assert_eq!(escape_markdown_v2("# Heading"), r"\# Heading");
    }
}
