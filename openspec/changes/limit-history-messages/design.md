## Context

Users want to limit the number of conversation history messages sent to the LLM to control costs, latency, or token usage. This is a simple config-based approach that filters messages before sending to the agent.

## Goals / Non-Goals

**Goals:**
- Add `history_limit` config option to limit history messages
- Preserve existing behavior when not set (unlimited)
- Simple implementation with clear semantics

**Non-Goals:**
- Per-message token counting/filtering (future enhancement)
- Different limits for different contexts (future enhancement)

## Decisions

### Decision 1: Implementation location

**Choice:** Filter in `session.rs` before calling `agent.chat()` / `agent.stream_chat()`

**Rationale:** 
- Session already has access to history and config
- Keeps history storage unchanged
- Easy to understand and debug

### Decision 2: Zero/negative handling

**Choice:** 
- `0` = send no history (current message only)
- Negative = treat as unlimited (same as not set)

**Rationale:** Simple, intuitive semantics. User can explicitly disable history with 0.

### Decision 3: Interaction with compaction

**Choice:** History limit applies AFTER compaction

**Rationale:** Compaction already reduces history to fit context window. History limit is an additional user-controlled filter. They serve different purposes.

## Risks / Trade-offs

- **[Risk] User confusion** → Clear documentation needed: this limits message count, not tokens
- **[Risk] Compaction interaction** → Need to ensure limit is applied after compaction to avoid conflicts
