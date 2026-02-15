## 1. Config and Provider Model Updates

- [x] 1.1 Add `OpenAICompatible` to `Provider` enum in `package/shared/src/config.rs` with serde mapping `openai-compatible`
- [x] 1.2 Update `ModelConfig::validate()` so `api_key` is optional for `openai-compatible` and remains required for `openai`, `anthropic`, and `gemini`
- [x] 1.3 Add/adjust unit tests in `package/shared/src/config.rs` for deserializing `openai-compatible` and validating optional/required `api_key` behavior

## 2. Agent Construction for OpenAI-Compatible Endpoints

- [x] 2.1 Update provider matching in `package/hi-core/src/provider.rs` to handle `Provider::OpenAICompatible`
- [x] 2.2 Implement OpenAI-compatible branch using `rig::providers::openai::CompletionsClient` with `model`, optional `api_key`, and optional `api_base`
- [x] 2.3 Ensure both `create_agent()` and `create_agent_from_small()` paths work for OpenAI-compatible configs

## 3. Multi-Model and Feature Path Verification

- [x] 3.1 Verify `small_model` with `provider: "openai-compatible"` resolves correctly through `resolve_model_ref()` without behavior regressions
- [x] 3.2 Verify inline `ModelRef::Inline` using `openai-compatible` is preserved through heartbeat/scheduler model resolution
- [x] 3.3 Add/adjust tests that cover OpenAI-compatible usage in `small_model`, named `"small"` references, and inline model refs

## 4. End-to-End Validation and Documentation

- [x] 4.1 Run `cargo check --workspace` and fix any compile issues introduced by provider/config changes
- [x] 4.2 Run `cargo test --workspace` and ensure all existing plus new tests pass
- [x] 4.3 Update `README.md` config examples to include a minimal `openai-compatible` endpoint configuration
