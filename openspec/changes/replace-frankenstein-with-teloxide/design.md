## Context

The `hi-remote` crate provides a Telegram bot adapter that bridges LLM chat sessions to Telegram. It currently depends on a personal git fork of `frankenstein` (rev `f34d9ebd`), pinned to a specific commit. The crate has three source files:

- `lib.rs` — entrypoint, config validation, delegates to `telegram::run_polling_loop`
- `telegram.rs` — manual `getUpdates` polling loop, message handling, retry logic, message splitting
- `session_manager.rs` — per-chat `ChatSession` management via `Arc<Mutex<HashMap>>`

The frankenstein API surface used is small:
- `frankenstein::client_reqwest::Bot` (construction, clone, `get_updates`, `send_message`)
- `frankenstein::methods::{GetUpdatesParams, SendMessageParams}` (builder pattern)
- `frankenstein::types::AllowedUpdate` (enum variant `Message`)
- `frankenstein::updates::UpdateContent` (enum variant `Message(msg)`)
- `frankenstein::AsyncTelegramApi` (trait import for async methods)
- `frankenstein::Error::Api` (rate-limit detection via `error_code == 429`)

## Goals / Non-Goals

**Goals:**
- Replace `frankenstein` with `teloxide` as the Telegram Bot API client
- Maintain identical external behavior: long-polling, per-chat sessions, message splitting, rate-limit retry
- Remove git-based dependency in favor of crates.io
- Keep `SessionManager` and `split_message` logic unchanged

**Non-Goals:**
- Adopting teloxide's full `Dispatcher` / `dptree` framework (would be a larger refactor; keep manual polling for now to minimize change scope)
- Adding new Telegram features (inline keyboards, callback queries, webhooks)
- Changing the `SessionManager` architecture or `ChatSession` interface
- Modifying `lib.rs` beyond updating the function call

## Decisions

### Decision 1: Keep manual polling loop instead of teloxide Dispatcher

**Choice**: Use teloxide's `Bot` as a plain API client, keeping the existing manual `loop { bot.get_updates().await }` structure.

**Rationale**: The current code works well with a simple manual loop. Adopting teloxide's `Dispatcher` + `dptree` routing would require restructuring how `SessionManager` is injected, changing function signatures, and introducing framework-level abstractions. This is a dependency swap, not an architecture change.

**Alternative considered**: Full teloxide `Dispatcher` migration — rejected because it increases scope and risk with no functional benefit for the current simple message-only use case.

### Decision 2: Use teloxide's request builder methods directly

**Choice**: Replace `SendMessageParams::builder().chat_id(id).text(t).build()` + `bot.send_message(&params)` with teloxide's `bot.send_message(ChatId(id), text).await`.

**Rationale**: Teloxide uses method-chaining on request objects rather than separate param structs. This is more ergonomic and reduces boilerplate.

### Decision 3: Wrap chat_id with ChatId newtype

**Choice**: Keep `chat_id` as `i64` in `SessionManager` and the internal `handle_message` function, wrapping to `ChatId(chat_id)` only at the teloxide API boundary.

**Rationale**: `SessionManager` uses `i64` keys in its HashMap and is independent of the Telegram crate. Minimizing type changes keeps the diff small and avoids propagating teloxide types into `session_manager.rs`.

### Decision 4: Map rate-limit errors to teloxide's RequestError

**Choice**: Replace `frankenstein::Error::Api(err) if err.error_code == 429` with teloxide's `RequestError::RetryAfter(duration)` pattern match.

**Rationale**: Teloxide provides a dedicated variant for 429 responses, which is cleaner than manual error-code matching.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| teloxide's `get_updates` API may differ from frankenstein's builder pattern | Verify teloxide's `GetUpdates` request supports `timeout`, `offset`, and `allowed_updates` via method chaining |
| teloxide is a larger dependency → longer compile times | Acceptable trade-off for upstream maintenance and crates.io availability |
| teloxide's `Bot` may handle retries internally, conflicting with our retry logic | Verify teloxide does NOT auto-retry on 429; our manual retry in `send_message_with_retry` should remain |
| `MessageId` type in teloxide is `i32` vs frankenstein's `i64` | Not relevant — current code does not store or pass message IDs |

## Dependency Changes

| Before | After |
|--------|-------|
| `frankenstein = { git = "...", rev = "f34d9ebd...", features = ["client-reqwest-native-tls"] }` | `teloxide = { version = "0.13", default-features = false, features = ["native-tls"] }` |

> Note: Use the latest stable teloxide version available. The `native-tls` feature replaces frankenstein's `client-reqwest-native-tls`.
