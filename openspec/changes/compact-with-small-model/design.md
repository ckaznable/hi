## Context

The current compact flow is heuristic-only: `ChatSession::send_message()` calls `history.compact(context_window)` when `needs_compact()` is true, and `ChatHistory::compact()` drops the oldest 50% of messages once estimated tokens exceed 80% threshold. This is simple and reliable but can remove essential context that the model still needs.

The change introduces an LLM-assisted compact strategy where a small model summarizes old context into a compact system message. The system already has `small_model` support and provider abstractions in place, so this design extends existing model resolution and session orchestration rather than adding a separate pipeline.

## Goals / Non-Goals

**Goals:**
- Add configurable compact strategy that can use a small model to summarize history.
- Keep existing truncation compact as deterministic fallback.
- Make compact behavior configurable in `config.json` (toggle, threshold, model ref, summary prompt).
- Integrate into existing pre-send path without breaking session/history persistence.

**Non-Goals:**
- Replacing token estimation heuristic with exact tokenizer accounting.
- Building a multi-stage memory/RAG subsystem.
- Adding streaming summarize flow.
- Changing TUI command surface for compact controls in this iteration.

## Decisions

### 1) Add explicit compact config block
- Decision: Add a dedicated `compact` config object under `ModelConfig` with fields:
  - `enabled` (bool)
  - `strategy` (`"truncate"` | `"small-model"`)
  - `trigger_ratio` (float, default 0.8)
  - `model` (`ModelRef`, default `"small"`)
  - `prompt` (optional summarize instruction)
- Rationale: keeps compact concerns isolated and avoids overloading unrelated settings.
- Alternative considered: infer strategy from `small_model` presence only.
  - Rejected because implicit behavior is hard to reason about and test.

### 2) Compact orchestration remains in ChatSession
- Decision: `ChatSession::send_message()` remains the decision point for compact execution; it chooses strategy and invokes history update before normal chat call.
- Rationale: session already owns config, history, context-manager dirtying, and message send lifecycle.
- Alternative considered: move orchestration into `ChatHistory`.
  - Rejected because `ChatHistory` should stay storage-oriented and not depend on model calls.

### 3) Add summary-based compaction API in ChatHistory
- Decision: extend `ChatHistory` with a summary-apply method that replaces older messages with a single synthesized system summary while preserving recent turns.
- Rationale: keeps final mutation logic in one place and allows deterministic persistence behavior.
- Alternative considered: rebuild message vector entirely in `ChatSession`.
  - Rejected to avoid duplicated mutation logic and tests.

### 4) Use existing small-model resolution and provider path
- Decision: resolve compact model using existing `ModelRef` resolution (`small` by default) and use existing provider/agent creation path.
- Rationale: minimal architectural change, consistent behavior across providers.
- Alternative considered: dedicated compact-only model client.
  - Rejected as unnecessary duplication.

### 5) Fail-safe fallback strategy
- Decision: if small-model compact fails (resolution/provider/chat error), immediately fallback to truncation compact and continue request flow.
- Rationale: compact must never block core chat UX.
- Alternative considered: fail request on compact error.
  - Rejected due to poor resilience.

### 6) Context reinjection after compact
- Decision: any compact mutation (summary or truncate) marks context as dirty for next injection.
- Rationale: system/tool/skill context consistency after history rewrite.

## Risks / Trade-offs

- [Risk] Small-model summary quality may omit critical details -> Mitigation: configurable prompt and preserving latest N messages unchanged.
- [Risk] Compact path adds latency spikes near threshold -> Mitigation: compact only when crossing trigger ratio and fallback quickly on error.
- [Risk] Additional model calls increase cost -> Mitigation: use `small` default and allow strategy `truncate` for cost-sensitive deployments.
- [Risk] Summary message growth over repeated compactions -> Mitigation: bounded summary prompt and retention policy for recent turns.

## Migration Plan

1. Add `compact` config schema/types/defaults in `shared` with backward-compatible optional semantics.
2. Add tests for config parsing and defaults.
3. Extend `ChatHistory` with summary-apply mutation and tests.
4. Implement compact strategy selection and fallback in `ChatSession::send_message()`.
5. Wire small-model resolution and chat invocation for summarize path.
6. Ensure context manager dirtying on compact mutations.
7. Validate via workspace check/tests and add config example updates.

Rollback: disable `small-model` strategy and keep truncate-only path while leaving config backward-compatible.

## Open Questions

- What exact retention window should remain verbatim after summary (fixed count vs ratio)?
- Should compact summary include tool outputs verbatim, or abstract them by default?
- Do we need a hard cap on summary token length separate from `trigger_ratio`?
