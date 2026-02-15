## Why

Current compact behavior drops old messages by heuristic, which can lose important context and reduce answer quality in long conversations. We need an LLM-based compact path that lets a configured small model summarize history into a shorter, useful context while controlling cost.

## What Changes

- Add a compact strategy that calls a configured small model to summarize prior conversation instead of only truncating oldest messages.
- Add config fields to control compact behavior (enable/disable, trigger threshold, summary prompt, and model selection with `small` support).
- Keep fallback behavior: if small model compact fails, system falls back to existing truncation compact.
- Integrate compact result into session/history flow so subsequent turns use summarized context.

## Capabilities

### New Capabilities
- `llm-compact`: Provide model-driven history compaction using a small model and summary prompt, with deterministic fallback to truncation.

### Modified Capabilities
- `model-config`: Extend config schema with compact-related settings and model reference for compact execution.
- `chat-history`: Update compact requirement to support summary-based compaction output in addition to existing truncation fallback.
- `chat-session`: Update pre-send compact workflow to invoke small-model compact strategy when configured.

## Impact

- Affected code: `shared::config` (new compact config fields), `hi-core::session` (compact orchestration), `hi-core::provider/model_pool` (small model invocation path), and `hi-history` (summary replacement behavior).
- Runtime impact: extra model call at compact time, reduced token growth over long sessions, and improved retained context quality.
- Risks: compact quality variance from small model output and additional latency at compaction points.
