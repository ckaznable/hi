## Context

The Telegram bot currently handles all incoming messages as chat input to the LLM. There is no command parsing or special handling. The bot uses long polling via `get_updates` and processes each text message through the session's `send_message_streaming` method.

The existing session management (`SessionManager` and `ChatSession`) already supports:
- `reset()` - clears chat history
- `run_compact_if_needed()` - compacts history based on context window

These methods are not currently exposed to the Telegram handler.

## Goals / Non-Goals

**Goals:**
- Add `/compact` command to manually trigger history compaction
- Add `/new` command to start a new conversation (reset history)
- Add `/help` command to show available commands
- Return confirmation messages after command execution

**Non-Goals:**
- Rich command responses (buttons, keyboard)
- Command aliases (/start, /restart)
- Persistent command state
- Admin-only commands

## Decisions

### 1. Command detection: Prefix matching

**Decision:** Check if message starts with `/` to identify commands.

**Rationale:**
- Simple to implement
- Telegram commands naturally start with `/`
- Non-command messages are passed through normally

**Alternatives considered:**
- Use Telegram's built-in commands API: More complex setup, requires botfather registration
- Regex matching: Overkill for simple prefix check

### 2. Command implementation: Inline in telegram.rs

**Decision:** Handle commands directly in the message handler without separate command module.

**Rationale:**
- Simple change, limited scope
- Commands are specific to Telegram adapter
- Easy to extend later if needed

**Alternatives considered:**
- Separate command module: Over-architecture for 3 commands
- Trait-based command handler: Too complex

### 3. Error handling: Graceful degradation

**Decision:** If a command fails, send an error message to the user instead of silently failing.

**Rationale:**
- Better UX - user knows something went wrong
- Easier debugging
- Consistent with bot behavior

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Command conflicts with normal chat | Users can't send messages starting with / | Document commands clearly, use uncommon command names |
| Compact fails mid-way | Partial state, user confusion | Wrap in try/catch, send error message on failure |
| Session not found | Command fails | Create session if doesn't exist |

## Migration Plan

1. Add command parsing logic to `handle_message` function
2. Add methods to SessionManager for reset and compact
3. Test commands work correctly
4. Deploy - no migration needed (new feature)

## Open Questions

1. **Should we support /reset as alias for /new?** - Nice to have, can add later
2. **Should we show commands in botfather menu?** - Optional, can be added later via botfather
