## 1. Add Command Handling to Telegram Adapter

- [x] 1.1 Update `package/hi-remote/src/telegram.rs` to detect commands (messages starting with `/`)
- [x] 1.2 Implement `/compact` command - call history compaction and send confirmation
- [x] 1.3 Implement `/new` command - reset chat history and send confirmation
- [x] 1.4 Implement `/help` command - show available commands list
- [x] 1.5 Handle unknown commands gracefully with helpful error message

## 2. Expose Session Methods

- [x] 2.1 Add `reset()` method to `SessionManager` if not already exposed
- [x] 2.2 Add compact method to `SessionManager` to trigger manual compaction

## 3. Test and Verify

- [x] 3.1 Run `cargo check -p hi-remote` to verify compilation
- [x] 3.2 Test `/compact` command works in Telegram
- [x] 3.3 Test `/new` command works in Telegram
- [x] 3.4 Test `/help` command shows correct information
- [x] 3.5 Test that unknown commands show helpful error
- [x] 3.6 Verify normal chat messages still work after adding commands
