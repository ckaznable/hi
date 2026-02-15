## 1. Workspace Setup

- [ ] 1.1 Convert root `Cargo.toml` to workspace, add `package/shared`, `package/hi-history`, `package/hi-tools`, `package/hi-core`, `package/hi-tui` as members
- [ ] 1.2 Create `package/shared/Cargo.toml` with dependencies: `directories`, `serde`, `serde_json`, `anyhow`
- [ ] 1.3 Create `package/hi-history/Cargo.toml` with dependencies: `shared`, `serde`, `serde_json`, `zstd`, `anyhow`
- [ ] 1.4 Create `package/hi-tools/Cargo.toml` with dependencies: `rig-core`, `serde`, `serde_json`, `tokio`, `anyhow`
- [ ] 1.5 Create `package/hi-core/Cargo.toml` with dependencies: `shared`, `hi-history`, `hi-tools`, `rig-core`, `tokio`, `anyhow`
- [ ] 1.6 Create `package/hi-tui/Cargo.toml` (binary crate) with dependencies: `hi-core`, `shared`, `ratatui`, `crossterm`, `tokio`, `anyhow`
- [ ] 1.7 Create stub `lib.rs` for shared, hi-history, hi-tools, hi-core and `main.rs` for hi-tui
- [ ] 1.8 Verify workspace compiles with `cargo check`

## 2. Shared: Config & Paths

- [ ] 2.1 Implement `ProjectDirs` wrapper: `config_dir()`, `data_dir()` functions using `directories` crate
- [ ] 2.2 Define `Provider` enum (`OpenAI`, `Anthropic`, `Gemini`, `Ollama`) with serde lowercase deserialization
- [ ] 2.3 Define `ModelConfig` struct with fields: `provider`, `model`, `api_key` (optional), `api_base` (optional), `preamble` (optional), `context_window`
- [ ] 2.4 Implement `ModelConfig::load()` — read and parse `config_dir/config.json`, validate `api_key` required for non-Ollama
- [ ] 2.5 Add unit tests for config parsing: valid configs, missing api_key, unknown provider

## 3. History Management

- [ ] 3.1 Define serializable `ChatMessage` struct compatible with rig's `Message` type
- [ ] 3.2 Implement `ChatHistory` struct with `messages: Vec<ChatMessage>`
- [ ] 3.3 Implement `ChatHistory::load()` — decompress zstd + deserialize JSON from `data_dir/history.json.zst`, return empty if not found
- [ ] 3.4 Implement `ChatHistory::save()` — serialize JSON + compress with zstd, write to `data_dir/history.json.zst`
- [ ] 3.5 Implement `ChatHistory::push()`, `messages()`, `token_estimate()` (chars / 4)
- [ ] 3.6 Implement `ChatHistory::compact()` — remove oldest messages, retain recent 50% when exceeding 80% of context_window
- [ ] 3.7 Implement `ChatHistory::reset()` — clear in-memory messages and delete history file from disk
- [ ] 3.8 Add unit tests for load/save round-trip, compact behavior, reset, token estimation

## 4. Built-in Tools

- [ ] 4.1 Implement `BashTool` with rig `Tool` trait — args: `{ command }`, exec via `tokio::process::Command`, return stdout/stderr/exit_code
- [ ] 4.2 Implement `ListFilesTool` with rig `Tool` trait — args: `{ path }`, return directory entries via `tokio::fs::read_dir`
- [ ] 4.3 Implement `ReadFileTool` with rig `Tool` trait — args: `{ path }`, return file content via `tokio::fs::read_to_string`
- [ ] 4.4 Implement `WriteFileTool` with rig `Tool` trait — args: `{ path, content }`, write content via `tokio::fs::write`
- [ ] 4.5 Define `ToolDefinition` JSON schemas for each tool (name, description, parameters)
- [ ] 4.6 Export a `builtin_tools()` function returning `Vec<Box<dyn ToolDyn>>`
- [ ] 4.7 Add unit tests for each tool: success cases and error cases (non-existent paths, permission errors)

## 5. Skills Loader

- [ ] 5.1 Define `Skill` struct with `name: String` and `content: String`
- [ ] 5.2 Implement `load_skills(config_dir: &Path) -> Vec<Skill>` — scan `config_dir/skills/*.md`, return empty vec if dir doesn't exist
- [ ] 5.3 Implement `build_preamble(base: Option<&str>, skills: &[Skill]) -> String` — concatenate base preamble + skill contents
- [ ] 5.4 Add unit tests for skills loading: with skills dir, without skills dir, empty dir

## 6. Core: Chat Session

- [ ] 6.1 Implement provider client factory: match `Provider` enum → create rig client + build agent with preamble, tools, and model
- [ ] 6.2 Implement `ChatSession` struct holding agent, history, and config
- [ ] 6.3 Implement `ChatSession::send_message()` — auto-compact if needed, call `agent.chat()` with history, push user + assistant messages, save history
- [ ] 6.4 Implement `ChatSession::reset()` — delegate to history reset
- [ ] 6.5 Implement `ChatSession::new()` — load config, load history, load skills, build agent with tools + preamble
- [ ] 6.6 Add integration test: create session with mock/ollama provider, send message, verify history

## 7. TUI: Chat Interface

- [ ] 7.1 Set up ratatui + crossterm terminal initialization and cleanup (raw mode, alternate screen)
- [ ] 7.2 Implement chat UI layout: scrollable message area (top) + input field (bottom)
- [ ] 7.3 Implement message rendering with visual distinction between user and assistant messages
- [ ] 7.4 Implement text input handling: typing, backspace, Enter to send
- [ ] 7.5 Implement async LLM communication via `tokio::sync::mpsc` — send user message, show loading indicator, receive response
- [ ] 7.6 Implement scrolling for message history area
- [ ] 7.7 Implement reset command (key binding to trigger history reset)
- [ ] 7.8 Implement quit key binding with graceful exit (save history, restore terminal)
- [ ] 7.9 Wire up `main.rs` — init tokio runtime, create ChatSession, launch TUI event loop

## 8. Final Integration & Verification

- [ ] 8.1 Create sample `config.json` with documentation comments
- [ ] 8.2 Create sample skill file `skills/default.md`
- [ ] 8.3 End-to-end test: run full application with a provider, verify chat + tools + history persistence
- [ ] 8.4 Verify compact triggers correctly when approaching context_window limit
- [ ] 8.5 Verify reset clears history in TUI and on disk
