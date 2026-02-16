## 1. Typing Indicator

- [x] 1.1 Import `ChatAction` from teloxide types in `telegram.rs`
- [x] 1.2 Send initial `ChatAction::Typing` in `handle_message` before `get_or_create`, ignoring errors
- [x] 1.3 Spawn periodic `tokio::spawn` task that re-sends `ChatAction::Typing` every 5 seconds, with `AbortHandle` to cancel
- [x] 1.4 Abort the periodic typing task after model response is received, before sending reply
- [x] 1.5 Add unit test: typing indicator is not triggered for `/` command messages

## 2. MarkdownV2 Escaping

- [x] 2.1 Implement `escape_markdown_v2(text: &str) -> String` that protects code blocks (triple backtick) and inline code (single backtick), escaping all MarkdownV2 special characters in non-code regions
- [x] 2.2 Add unit tests for `escape_markdown_v2`: plain text escaping, code block preservation, inline code preservation, mixed content

## 3. MarkdownV2 Send with Fallback

- [x] 3.1 Import `ParseMode` from teloxide types
- [x] 3.2 Modify `send_message_with_retry` to accept an optional `parse_mode` parameter and apply it to `send_message`
- [x] 3.3 Add plain-text fallback: on Telegram API error (non-rate-limit), retry the same text without `parse_mode`, log a warning
- [x] 3.4 Apply `escape_markdown_v2` to message text before calling `send_message_with_retry` with `ParseMode::MarkdownV2`

## 4. Verification

- [x] 4.1 Run `cargo check -p hi-remote` — no errors
- [x] 4.2 Run `cargo test -p hi-remote` — all tests pass
- [x] 4.3 Run `cargo check --workspace` — no workspace-wide errors
- [x] 4.4 Run `cargo test --workspace` — all workspace tests pass
