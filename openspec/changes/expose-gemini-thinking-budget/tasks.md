## 1. Config Changes

- [x] 1.1 Define `ThinkingConfig` struct in `package/shared/src/config.rs`
- [x] 1.2 Add `thinking: Option<ThinkingConfig>` field to `ModelConfig`
- [x] 1.3 Add unit test for thinking config parsing

## 2. Small Model Support

- [x] 2.1 Add `thinking: Option<ThinkingConfig>` field to `SmallModelConfig`
- [x] 2.2 Update `as_small_model_config()` to include thinking config

## 3. Provider Integration

- [x] 3.1 Update `create_agent_from_parts()` in `package/hi-core/src/provider.rs` to accept thinking config
- [x] 3.2 Pass thinking config to rig's client request
- [x] 3.3 Update `create_agent()` and `create_agent_from_small()` to handle thinking config
- [x] 3.4 Add unit test for thinking config passed to agent

## 4. Verification

- [x] 4.1 Run `cargo check --workspace`
- [x] 4.2 Run `cargo test --workspace`
