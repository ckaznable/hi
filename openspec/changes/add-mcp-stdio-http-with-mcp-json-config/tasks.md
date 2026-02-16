# Tasks: add-mcp-stdio-http-with-mcp-json-config

- [x] Add `rmcp` workspace dependency with stdio + HTTP transport features
- [x] Enable `rmcp` feature on `rig-core` dependency
- [x] Add `McpServerConfig` and `McpConfig` structs to `shared/src/config.rs`
- [x] Create `shared/src/mcp_store.rs` — load/parse `mcp.json` from config dir (7 tests)
- [x] Create `hi-core/src/mcp.rs` — `McpManager` with stdio + HTTP connect, tool discovery (5 tests)
- [x] Add `extra_tools` parameter to `create_agent()` in `provider.rs`
- [x] Make `ChatSession::new()` async, add `McpManager` field, call `load_and_connect()`
- [x] Pass MCP tools to `create_agent()` as `extra_tools` on session init
- [x] Update `switch_to_primary_model` to pass `vec![]` for `extra_tools` (MCP tools only on init)
- [x] Dynamically include MCP tool names in `tool_descriptions` for `send_message` and `send_message_streaming`
- [x] Update TUI call site for async `ChatSession::new()`
- [x] Update remote `SessionManager` call site for async `ChatSession::new()`
- [x] `cargo check --workspace` — 0 warnings
- [x] `cargo test --workspace` — 177 tests, 0 failures
