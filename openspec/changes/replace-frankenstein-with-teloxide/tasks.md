## 1. Dependency Swap

- [x] 1.1 In `package/hi-remote/Cargo.toml`, remove the `frankenstein` git dependency and add `teloxide` with `native-tls` feature (e.g. `teloxide = { version = "0.13", default-features = false, features = ["native-tls"] }`)
- [x] 1.2 Run `cargo check -p hi-remote` to verify the dependency resolves and identify all compilation errors from removed frankenstein imports

## 2. Update telegram.rs Imports and Bot Initialization

- [x] 2.1 Replace all `frankenstein::*` imports with teloxide equivalents: `teloxide::prelude::*`, `teloxide::types::{AllowedUpdate, ChatId, Message as TgMessage}`, `teloxide::requests::Requester`
- [x] 2.2 Replace `Bot::new(&telegram_config.bot_token)` with `teloxide::Bot::new(&telegram_config.bot_token)`

## 3. Update Polling Loop

- [x] 3.1 Replace `GetUpdatesParams::builder().timeout(timeout).allowed_updates(...).build()` with teloxide's `bot.get_updates().timeout(timeout).allowed_updates(vec![AllowedUpdate::Message]).offset(offset)` request pattern
- [x] 3.2 Replace `bot.get_updates(&update_params).await` with teloxide's request `.await` pattern; update offset tracking from `response.result` iteration to teloxide's response shape
- [x] 3.3 Replace `UpdateContent::Message(message)` pattern match with `update.message` (Option) extraction
- [x] 3.4 Update `message.chat.id` access and `message.text` access to match teloxide's `Message` struct field names

## 4. Update send_message_with_retry

- [x] 4.1 Replace `SendMessageParams::builder().chat_id(chat_id).text(text).build()` + `bot.send_message(&params)` with `bot.send_message(ChatId(chat_id), text).await`
- [x] 4.2 Replace `frankenstein::Error::Api(ref err) if err.error_code == 429` with `teloxide::RequestError::RetryAfter(duration)` pattern match; extract retry duration directly from the variant
- [x] 4.3 Update the generic error arm to use teloxide's error type

## 5. Update Function Signatures

- [x] 5.1 Update `handle_message` and `send_message_with_retry` function signatures to accept `teloxide::Bot` instead of `frankenstein::client_reqwest::Bot`
- [x] 5.2 Ensure `bot.clone()` still works (teloxide `Bot` is `Clone`)

## 6. Verify and Test

- [x] 6.1 Run `cargo check -p hi-remote` — zero errors
- [x] 6.2 Run `cargo test -p hi-remote` — all existing `split_message` and `session_manager` tests pass
- [x] 6.3 Run `cargo check --workspace` — no workspace-wide breakage
- [x] 6.4 Run `cargo test --workspace` — all tests pass
