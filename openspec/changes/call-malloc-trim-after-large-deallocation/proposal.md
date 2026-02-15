## Why

In long-running `hi` processes, several flows can retain large heap allocations longer than needed (history buffers, remote sessions, and unbounded streaming queues). We need an explicit memory-release strategy so large allocations are reclaimed predictably and RSS growth is controlled.

## What Changes

- Define a memory-reclamation policy for large deallocation paths and when allocator trimming should be attempted.
- Add guardrails for long-lived session/state containers so unused per-chat state does not accumulate indefinitely.
- Define bounded buffering/backpressure rules for streaming channels that currently have unbounded growth potential.
- Add observability and tests to verify compaction/release behavior and avoid regressions.

## Capabilities

### New Capabilities
- `memory-reclamation-policy`: Standardize thresholds and conditions for large deallocation handling (including optional `malloc_trim(0)` on supported platforms).
- `remote-session-lifecycle`: Add lifecycle constraints (TTL and/or capacity limits) for Telegram chat sessions.
- `streaming-buffer-backpressure`: Replace or constrain unbounded queues where sustained producer/consumer imbalance can grow memory.

### Modified Capabilities
- None currently identified; update after reviewing canonical spec IDs under `openspec/specs/` during specs phase.

## Impact

- Affected crates/modules:
  - `package/hi-history/src/history.rs` (history compaction/reset release points)
  - `package/hi-core/src/session.rs` (large summary-string construction during compaction)
  - `package/hi-remote/src/session_manager.rs` (long-lived per-chat sessions)
  - `package/hi-tui/src/lib.rs`, `package/hi-remote/src/telegram.rs`, `package/hi-core/src/provider.rs` (unbounded streaming channels)
- API surface: no external API break expected; runtime behavior and memory profile will change.
- Platform considerations: allocator-trim behavior is libc/platform dependent and must be feature-gated or conditionally compiled.
