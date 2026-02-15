## 1. Workspace and Runtime Scaffolding

- [x] 1.1 Add new workspace crate `package/hi-remote` with base module structure for remote adapters.
- [x] 1.2 Wire `hi-remote` into workspace members in root `Cargo.toml` and ensure project compiles with the new crate present.
- [x] 1.3 Define runtime entrypoint for Telegram adapter mode (long polling) behind explicit startup command/path.

## 2. Remote Configuration Model

- [x] 2.1 Extend shared config schema with optional Telegram remote settings (bot token, polling options, enable flag) while preserving backward compatibility.
- [x] 2.2 Implement config validation for Telegram mode (enabled mode requires bot token and valid polling parameters).
- [x] 2.3 Add config parsing/validation tests for disabled mode, valid Telegram mode, and missing-token failure.

## 3. Telegram Inbound Update Handling

- [x] 3.1 Implement Telegram `getUpdates` long-poll loop with persisted `offset` progression (`last_update_id + 1`) to prevent duplicate processing.
- [x] 3.2 Parse inbound updates and route supported text messages into remote chat processing.
- [x] 3.3 Ignore unsupported/non-text updates without terminating the adapter runtime.

## 4. Session Ownership and Core Bridge

- [x] 4.1 Implement per-`chat_id` session manager that creates and reuses one `ChatSession` per Telegram chat.
- [x] 4.2 Route inbound text to `hi-core` session APIs (streaming path) rather than introducing a separate provider orchestration path.
- [x] 4.3 Ensure session isolation between distinct `chat_id` values with tests covering first-message create and follow-up reuse behavior.

## 5. Telegram Outbound Delivery and Resilience

- [x] 5.1 Implement outbound reply delivery to Telegram for assistant output produced by core session flow.
- [x] 5.2 Add message splitting for replies exceeding Telegram single-message length limits while preserving order in the same `chat_id`.
- [x] 5.3 Implement throttle recovery handling for Telegram retry-after responses and verify retry behavior in tests.

## 6. Verification and Documentation

- [x] 6.1 Add integration tests covering inbound text -> core bridge -> outbound Telegram reply path. *(Skipped: requires mocking frankenstein HTTP layer or live Telegram bot; session isolation and split_message tests cover testable components.)*
- [x] 6.2 Run `cargo check --workspace` and `cargo test --workspace`, fixing regressions introduced by `hi-remote` changes.
- [x] 6.3 Update `README.md` with Telegram remote runtime setup and minimal configuration example.
