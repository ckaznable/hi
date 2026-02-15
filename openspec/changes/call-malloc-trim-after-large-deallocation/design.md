## Context

The current runtime uses long-lived in-memory structures for chat history, per-chat remote sessions, and streamed output buffers. While functional, these structures can hold onto large allocations longer than needed in long-running processes (especially Telegram remote mode). The proposal targets predictable memory reclamation without changing user-facing behavior.

Relevant current-state constraints:
- `ChatHistory` compacts by dropping older messages, but no explicit post-deallocation allocator trimming policy exists.
- `SessionManager` stores sessions in an unbounded `HashMap<chat_id, ChatSession>` with no lifecycle eviction.
- Multiple streaming paths use unbounded channels; producer/consumer imbalance can increase queue memory.
- Linux/glibc allocator behavior is platform-specific; reclamation controls must be gated and safe to no-op elsewhere.

## Goals / Non-Goals

**Goals:**
- Define a deterministic memory-reclamation policy for large deallocation points.
- Introduce lifecycle limits for long-lived remote sessions to prevent unbounded accumulation.
- Bound streaming buffers or enforce backpressure semantics where channels are currently unbounded.
- Add testable, observable acceptance criteria so memory-focused behavior does not regress silently.

**Non-Goals:**
- Rewriting the application into a multi-process architecture.
- Implementing exact RSS accounting in unit tests.
- Introducing a new persistence model for chat history.
- Changing end-user CLI/TUI command surface.

## Decisions

### 1) Add explicit memory-reclamation policy at high-impact release points
- Decision: Define policy-level thresholds and release triggers in `chat-history` and session flows; invoke allocator trim only when large memory release is likely.
- Rationale: avoids frequent low-value trimming while handling known large-release paths.
- Alternative considered: call trim after every clear/compact.
  - Rejected due to potential allocator overhead and throughput impact.

### 2) Gate allocator trim by platform and capability
- Decision: Require platform guardrails for `malloc_trim(0)` and permit no-op on unsupported environments.
- Rationale: behavior is libc/platform dependent; feature must remain portable and safe.
- Alternative considered: unconditional trim calls.
  - Rejected for portability and correctness risk.

### 3) Introduce session lifecycle controls in remote mode
- Decision: add session expiration and/or max-session constraints to `hi-remote` session ownership.
- Rationale: long-lived idle chats currently retain memory indefinitely.
- Alternative considered: keep infinite session retention and rely only on manual restarts.
  - Rejected because it fails long-running daemon use cases.

### 4) Replace unbounded queue growth with bounded flow control
- Decision: migrate unbounded streaming channels to bounded channels where practical, with explicit handling for full queues (await, drop policy, or coalescing strategy per path).
- Rationale: prevents memory runaway under sustained backpressure.
- Alternative considered: keep unbounded channels and monitor only.
  - Rejected because monitoring alone does not prevent OOM risk.

### 5) Keep behavior backward-compatible at API/UX level
- Decision: memory controls should change runtime characteristics without changing external commands or response semantics.
- Rationale: enables safe rollout and focused regression scope.

## Risks / Trade-offs

- [Risk] Overly aggressive trim/eviction can hurt latency or continuity -> Mitigation: thresholded policy, idle-time windows, and conservative defaults.
- [Risk] Bounded channels can block producers under bursty output -> Mitigation: per-path queue sizing and explicit overflow behavior in requirements.
- [Risk] Platform-specific allocator behavior may be inconsistent -> Mitigation: guard by target/platform and treat trim as best-effort optimization.
- [Risk] Session eviction might remove context users expected to persist -> Mitigation: define TTL/max-session behavior clearly and document defaults.

## Migration Plan

1. Define normative requirements for policy, lifecycle, and backpressure in delta/new specs.
2. Implement policy hooks in `hi-history`/session paths with platform guards.
3. Add remote session lifecycle controls and tests for reuse/expiry/eviction scenarios.
4. Convert selected unbounded channels to bounded channels and validate throughput behavior.
5. Add targeted tests and run workspace checks before completion.

Rollback strategy:
- Disable new policy/lifecycle behavior behind configuration defaults if regressions appear.
- Revert bounded-channel tuning conservatively while preserving correctness.

## Open Questions

- What default threshold should classify a deallocation as "large" for trim attempts?
- Should remote session lifecycle be TTL-only, max-count-only, or hybrid?
- Which streaming paths should coalesce/drop vs strictly block when queue is full?
