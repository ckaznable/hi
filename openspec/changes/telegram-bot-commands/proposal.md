## Why

Currently, Telegram bot users can only interact with the LLM by sending messages. There is no way to manually control the conversation state from Telegram. Users who want to start a fresh conversation or compact the history to save context window must use the TUI or edit config files. Adding command support will improve the Telegram user experience.

## What Changes

- Register `/compact` command in Telegram bot to manually trigger chat history compaction
- Register `/new` command in Telegram bot to reset/clear the current conversation (same as `/reset` in TUI)
- Show help text listing available commands when user sends `/help`
- Send confirmation message after successful command execution

## Capabilities

### New Capabilities
- `telegram-commands`: Register and handle Telegram bot commands (/compact, /new, /help)

### Modified Capabilities
- (none)

## Impact

- **Code changes**: Update `package/hi-remote/src/telegram.rs` to handle commands
- **Session API**: Expose `reset()` and compact functionality to Telegram handler
- **User experience**: Users can now control conversation state from Telegram
