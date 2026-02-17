## Why

Users may want to limit the number of conversation history messages sent to the LLM to reduce token usage, costs, or improve response latency. Currently, the system sends all history (subject to context window limits via compaction). A simple config option to set a fixed message count would give users more control.

## What Changes

- Add `history_limit` field to `ModelConfig` in `shared/src/config.rs`
- When set to a positive integer, only the most recent N messages are sent to the LLM
- When not set (default), existing behavior is preserved (send all history, apply compaction at threshold)

## Capabilities

### New Capabilities
- `history-limit`: Allow users to limit the number of conversation history messages sent to the LLM

### Modified Capabilities
- `model-config`: Add optional `history_limit` field to configuration

## Impact

- **Affected code**:
  - `package/shared/src/config.rs` — add `history_limit` field
  - `package/hi-core/src/session.rs` — filter history messages before sending to agent
  - `package/hi-history/src/lib.rs` — may need to expose message count
- **Config**: New `history_limit` field in `config.json` (optional, default: unlimited)
- **Dependencies**: No new dependencies
