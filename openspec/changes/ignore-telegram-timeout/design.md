## Context

The Telegram adapter in `package/hi-remote/src/telegram.rs` uses long polling via `getUpdates` API. The current implementation logs every error from `get_updates()` to stderr:

```rust
Err(e) => {
    eprintln!("[hi-remote] Failed to get updates: {e:?}");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

This creates noisy output during normal operation - particularly when network timeouts occur during long polling.

## Goals / Non-Goals

**Goals:**
- Ignore timeout-related errors silently during long polling
- Continue normal polling behavior after timeout
- Only log meaningful errors (API errors, authentication failures, unexpected network issues)

**Non-Goals:**
- Change the polling behavior or interval
- Add new dependencies
- Modify other error handling paths

## Decisions

### 1. Timeout Error Detection
**Decision**: Check for `RequestError::Network` error variant and silently ignore it.

**Rationale**: 
- Long polling timeout is a normal operation - Telegram returns empty updates, not an error
- Network timeouts (connection dropped, server didn't respond) manifest as `Network` variant
- Other error variants (API errors, rate limits) should still be logged

**Alternative considered**: Parse error message strings - rejected due to fragility

### 2. Error Logging Strategy
**Decision**: Keep existing `eprintln!` for non-timeout errors, remove for timeout.

**Rationale**: 
- Minimal code change
- Existing error logging is useful for debugging actual issues
- Only timeout errors create noise during normal operation

## Risks / Trade-offs

- **Risk**: Network error detection might miss some edge cases → **Mitigation**: Can refine error matching later if needed
- **Risk**: Loss of visibility into polling issues → **Mitigation**: Non-timeout errors still logged; can add debug logging if needed

## Migration Plan

1. Single code change in `telegram.rs`
2. No migration needed - change is backward compatible
3. Rollback: simply revert the code change
