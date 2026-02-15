## Why

The `hi-remote` crate currently depends on a custom fork of `frankenstein` (pinned to a specific git rev). `frankenstein` is a lower-level Telegram Bot API binding that requires manual polling loops, manual offset tracking, and verbose builder-based parameter construction. Replacing it with `teloxide` — the most widely adopted Rust Telegram framework — brings ergonomic abstractions (dispatcher, dependency injection, filter-based routing), built-in rate-limit retry, and long-term upstream maintenance stability without relying on a personal fork.

## What Changes

- **BREAKING**: Remove `frankenstein` git dependency from `hi-remote`; add `teloxide` with `reqwest` native-tls backend
- Replace manual `getUpdates` polling loop in `telegram.rs` with teloxide `Dispatcher`-based polling
- Replace `SendMessageParams` builder pattern with teloxide's method-chaining API (`bot.send_message(chat_id, text).await`)
- Replace `frankenstein::Error::Api` rate-limit matching with `teloxide::RequestError::RetryAfter`
- Adapt `chat_id` from bare `i64` to teloxide `ChatId` newtype wrapper
- Update `run_polling_loop` signature — the `Dispatcher` handles the event loop internally
- Keep `SessionManager`, `split_message`, and retry logic functionally identical

## Capabilities

### New Capabilities

_(none — this is a dependency swap, not a feature addition)_

### Modified Capabilities

_(none — the external behavior of the Telegram remote adapter is unchanged: long-polling, per-chat sessions, message splitting, retry on rate limit. This is a pure implementation-level change with no spec-level requirement differences.)_

## Impact

- **Code**: `package/hi-remote/src/telegram.rs` (primary), `package/hi-remote/Cargo.toml` (dependency swap)
- **Dependencies**: Remove `frankenstein` (git fork); add `teloxide` (crates.io). This also removes the transitive `frankenstein → reqwest` path and replaces it with `teloxide → reqwest`.
- **Build**: Eliminates git-based dependency fetch; all deps become crates.io-resolvable. May affect compile time (teloxide is larger).
- **Tests**: Existing `split_message` unit tests are unaffected. No integration tests exist for the polling loop.
- **Docs**: `README.md` does not mention frankenstein by name — no doc changes needed.
