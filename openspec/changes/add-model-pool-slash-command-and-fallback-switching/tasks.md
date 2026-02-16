# Tasks: add-model-pool-slash-command-and-fallback-switching

## Implementation

- [x] 1.1 Add `using_small_model: bool` field to `ChatSession`
- [x] 1.2 Add `current_model_name()` method to `ChatSession`
- [x] 1.3 Add `is_using_small_model()` method to `ChatSession`
- [x] 1.4 Add `switch_to_small_model()` method — creates agent from `small_model` with full tools
- [x] 1.5 Add `switch_to_primary_model()` method — recreates agent from primary config
- [x] 1.6 Add `create_agent_from_small_with_tools()` to `provider.rs` — small model agent with tools

## Fallback Logic

- [x] 2.1 Add fallback in `send_message()` — on failure, retry with small model if available
- [x] 2.2 Add fallback in `send_message_streaming()` — clone `chunk_tx` before first attempt for retry

## TUI Integration

- [x] 3.1 Add `SessionCmd::SwitchModel(String)` variant
- [x] 3.2 Add `SessionReply::ModelSwitched(String)` variant
- [x] 3.3 Handle `SwitchModel` command in session task (dispatch to switch methods)
- [x] 3.4 Handle `ModelSwitched` reply in UI (display system message)
- [x] 3.5 Add `/model` and `/model small` and `/model primary` input parsing

## Documentation

- [x] 4.1 Update README TUI Controls section with `/model` commands

## Verification

- [x] 5.1 `cargo check --workspace` passes
- [x] 5.2 `cargo test --workspace` passes (165 tests)
