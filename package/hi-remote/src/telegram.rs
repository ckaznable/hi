use std::sync::Arc;

use anyhow::Result;
use shared::config::{ModelConfig, TelegramConfig};
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::payloads::GetUpdatesSetters;
use teloxide::types::{AllowedUpdate, ChatId, MediaKind, MessageKind, UpdateKind};
use teloxide::RequestError;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::session_manager::SessionManager;

// Telegram API: 4096 UTF-8 chars per message
const MAX_MESSAGE_LENGTH: usize = 4096;
const MAX_RETRY_ATTEMPTS: u32 = 3;

pub async fn run_polling_loop(config: &ModelConfig, telegram_config: &TelegramConfig) -> Result<()> {
    let bot = Bot::new(&telegram_config.bot_token);
    let session_manager = Arc::new(SessionManager::new(config.clone()));

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
                                error!(
                                    chat_id,
                                    "Error handling message: {e}"
                                );
                            }
                        });
                    }
                }
            }
            Err(RequestError::Network(_)) => {
                // Network/timeout errors are expected during long polling — silently retry
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

    let (stream_tx, mut stream_rx) = mpsc::channel::<String>(hi_core::provider::STREAM_CHANNEL_CAPACITY);

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
        send_message_with_retry(bot, chat_id, &chunk).await?;
    }

    Ok(())
}

async fn handle_command(
    chat_id: i64,
    command: &str,
    bot: &Bot,
    session_manager: &SessionManager,
) -> Result<()> {
    let cmd = command.split_whitespace().next().unwrap_or("");

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
        "help" => concat!(
            "Available commands:\n",
            "/compact - Compact chat history\n",
            "/new - Start a new conversation\n",
            "/help - Show this help message",
        ).to_string(),
        _ => format!("Unknown command: /{cmd}\nUse /help to see available commands."),
    };

    send_message_with_retry(bot, chat_id, &reply).await?;
    Ok(())
}

async fn send_message_with_retry(bot: &Bot, chat_id: i64, text: &str) -> Result<()> {
    for attempt in 0..MAX_RETRY_ATTEMPTS {
        match bot.send_message(ChatId(chat_id), text).await {
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
                return Err(anyhow::anyhow!(
                    "Failed to send message to chat {chat_id}: {e}"
                ));
            }
        }
    }

    anyhow::bail!(
        "Failed to send message to chat {chat_id} after {MAX_RETRY_ATTEMPTS} retry attempts"
    )
}

fn split_message(text: &str) -> Vec<String> {
    if text.len() <= MAX_MESSAGE_LENGTH {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if remaining.len() <= MAX_MESSAGE_LENGTH {
            chunks.push(remaining.to_string());
            break;
        }

        let split_at = remaining[..MAX_MESSAGE_LENGTH]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(MAX_MESSAGE_LENGTH);

        let (chunk, rest) = remaining.split_at(split_at);
        chunks.push(chunk.to_string());
        remaining = rest;
    }

    chunks
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
}
