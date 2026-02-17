## 1. Config Changes

- [x] 1.1 Add `history_limit: Option<usize>` field to `ModelConfig` in `package/shared/src/config.rs`
- [x] 1.2 Add unit test for history_limit config parsing

## 2. Session Integration

- [x] 2.1 Import `history_limit` in `session.rs`
- [x] 2.2 Create helper function to filter history messages by limit
- [x] 2.3 Apply filter in `send_message()` before calling agent
- [x] 2.4 Apply filter in `send_message_streaming()` before calling agent
- [x] 2.5 Add unit test for history limit filtering

## 3. Verification

- [x] 3.1 Run `cargo check --workspace`
- [x] 3.2 Run `cargo test --workspace`
- [x] 3.3 Manual test: verify history is limited with config set
- [x] 3.4 Manual test: verify unlimited behavior when not set
