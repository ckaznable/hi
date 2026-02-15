## 1. Reclamation Policy Foundations

- [x] 1.1 Add memory policy configuration/types (large-release threshold and policy toggles) in `shared` config with backward-compatible defaults.
- [x] 1.2 Implement large-release evaluation points in `package/hi-history/src/history.rs` for compact and reset flows.
- [x] 1.3 Add platform-gated allocator trim hook (`malloc_trim(0)` best-effort) for supported targets and no-op behavior elsewhere.
- [x] 1.4 Emit structured runtime evidence for qualifying and non-qualifying reclamation decisions.

## 2. Remote Session Lifecycle Controls

- [x] 2.1 Extend remote/session config with TTL and max-session capacity controls.
- [x] 2.2 Implement idle-session eviction in `package/hi-remote/src/session_manager.rs` based on configured TTL.
- [x] 2.3 Implement deterministic capacity eviction policy when admitting a new session exceeds configured limit.
- [x] 2.4 Emit structured lifecycle events for session create/reuse/evict decisions.

## 3. Streaming Backpressure Hardening

- [x] 3.1 Inventory current unbounded streaming channels in `hi-core`, `hi-tui`, and `hi-remote` and define per-path bounded capacities.
- [x] 3.2 Replace selected `mpsc::unbounded_channel` usages with bounded channels plus explicit full-queue policy handling.
- [x] 3.3 Preserve chunk ordering guarantees while applying full-queue policy semantics.
- [x] 3.4 Emit saturation/unsaturation structured evidence for streaming pipelines.

## 4. Validation and Regression Coverage

- [x] 4.1 Add unit tests for policy threshold behavior (qualifying vs non-qualifying release paths).
- [x] 4.2 Add session manager tests for TTL eviction, capacity eviction, and session reuse behavior.
- [x] 4.3 Add streaming tests covering producer-faster-than-consumer backpressure behavior and ordering guarantees.
- [x] 4.4 Run `cargo check --workspace` and `cargo test --workspace`; address regressions introduced by this change.

## 5. Documentation and Rollout Notes

- [x] 5.1 Update `README.md` configuration examples with memory/session/backpressure controls.
- [x] 5.2 Document platform behavior notes for allocator trimming (supported vs no-op platforms).
- [x] 5.3 Add rollout guidance for conservative defaults and fallback/rollback toggles.
