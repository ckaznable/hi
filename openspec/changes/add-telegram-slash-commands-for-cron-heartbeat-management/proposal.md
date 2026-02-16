## Why

Telegram users currently have to modify config files or rely on LLM-mediated flows to inspect and adjust cron, heartbeat, and MCP runtime switches. This is slow and error-prone for routine operations that should be handled deterministically by the program.

## What Changes

- Add Telegram slash commands to view and edit cron scheduling settings directly in remote mode.
- Add Telegram slash commands to view and edit heartbeat settings directly in remote mode.
- Add Telegram slash commands to view MCP enabled/disabled status and toggle MCP (`/mcp`, `/mcp on`, `/mcp off`).
- Route these commands to direct program handlers without invoking LLM inference.
- Enforce strict input format validation; when invalid, return an immediate usage example message.

## Capabilities

### New Capabilities
- `telegram-runtime-controls`: Telegram slash-command runtime controls for cron, heartbeat, and MCP state.

### Modified Capabilities
- (none)

## Impact

- **Code changes**: `package/hi-remote/src/telegram.rs` command parsing and command dispatch.
- **Runtime config integration**: Cron, heartbeat, and MCP control paths need safe read/write handlers.
- **User experience**: Telegram operators can manage runtime settings without editing files manually.
- **LLM cost/latency**: Operational commands bypass LLM calls.
