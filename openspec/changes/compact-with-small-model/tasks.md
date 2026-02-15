## 1. Config Schema for Compact Strategy

- [x] 1.1 Add compact config types to `package/shared/src/config.rs` (`enabled`, `strategy`, `trigger_ratio`, `model`, `prompt`)
- [x] 1.2 Extend `ModelConfig` to include optional `compact` field with backward-compatible defaults
- [x] 1.3 Add config tests for `compact` parsing (`small-model` and `truncate`) and default behavior when omitted

## 2. History Compaction Data Operations

- [x] 2.1 Extend `package/hi-history/src/history.rs` with API to apply summary-based compaction while preserving recent turns
- [x] 2.2 Add support for embedding an explicit user-language marker in compacted summary context
- [x] 2.3 Add unit tests for summary compaction output, language marker presence, and truncate fallback behavior

## 3. Session Orchestration for Small-Model Compact

- [x] 3.1 Update `package/hi-core/src/session.rs` to choose compact strategy from config before normal send flow
- [x] 3.2 Implement small-model compact path using resolved `ModelRef` (default `small`) and summarization prompt
- [x] 3.3 Implement deterministic fallback to truncate when small-model compact fails and ensure request flow continues
- [x] 3.4 Ensure `context_manager.mark_dirty()` is called after any compact mutation (summary or truncate)

## 4. Compact Prompt and Language Continuity

- [x] 4.1 Define compact prompt template that instructs summary generation and includes the current user language marker
- [x] 4.2 Ensure next-turn response behavior preserves marked language unless user explicitly switches language
- [x] 4.3 Add tests covering compact-triggered language continuity across subsequent assistant responses

## 5. End-to-End Validation and Docs

- [x] 5.1 Run `cargo check --workspace` and fix compilation issues from compact strategy changes
- [x] 5.2 Run `cargo test --workspace` and ensure new and existing tests pass
- [x] 5.3 Update `README.md` with compact config examples including `small-model` strategy and language continuity note
