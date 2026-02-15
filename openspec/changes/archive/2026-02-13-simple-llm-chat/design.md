## Context

Project `hi` is a brand-new Rust project. The goal is to build a simple LLM chat system. Right now there is only an empty `main.rs`, so everything must be implemented from scratch.

The rig framework provides a unified LLM interface and supports multiple providers such as OpenAI, Anthropic, Gemini, and Ollama. Multi-turn chat is handled through `Chat` / `StreamingChat` traits with `Message::user()` / `Message::assistant()` history construction. rig's `Tool` trait enables custom tool calling by the LLM.

## Goals / Non-Goals

**Goals:**

- Create a Cargo workspace split into five crates: `shared`, `hi-core`, `hi-history`, `hi-tools`, `hi-tui`
- Implement a JSON config system with switchable LLM providers
- Implement multi-turn chat based on rig `Chat` trait
- Implement chat-history management with JSON + zstd storage, compact, and reset
- Provide built-in tools: `bash`, list files, read file, write file
- Implement skills loading from `config_dir/skills/`
- Keep single-session design and auto-load previous history on startup
- Provide a TUI chat interface

**Non-Goals:**

- No RAG, embeddings, or vector search
- No multi-session switching
- No streaming-response handling in initial version (initial release uses full responses)
- No Web API or HTTP server

## Decisions

### 1. Workspace structure

```
hi/
├── Cargo.toml             (workspace root)
├── package/
│   ├── shared/            (shared: paths and config structures)
│   ├── hi-history/        (history: storage, compact, reset)
│   ├── hi-tools/          (built-in tools: bash and file operations)
│   ├── hi-core/           (rig integration, chat logic, skills loading)
│   └── hi-tui/            (TUI interface, binary crate)
```

**Dependency flow:**
```
hi-tui -> hi-core -> hi-history -> shared
                  -> hi-tools
```

**Rationale:** `hi-tools` is separated so tools can be tested and extended independently. `hi-core` integrates tools + history + skills.

### 2. JSON config format

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-...",
  "api_base": null,
  "preamble": "You are a helpful assistant.",
  "context_window": 128000
}
```

- Path: `config.json` under `ProjectDirs::config_dir("hi")`
- Deserialize via `serde` + `serde_json` into `ModelConfig`
- `provider` is enum (`openai` / `anthropic` / `gemini` / `ollama`)
- `api_key` is optional for ollama, required for other providers
- `api_base` is optional for custom endpoints
- `context_window` is used as compact threshold baseline

**Rationale:** JSON is simple and intuitive. `ProjectDirs` stores config in standard OS locations (for example `~/.config/hi/`) and follows XDG conventions.

### 3. Provider abstraction

Use rig's unified interface and create a provider client dynamically from config:

```rust
match config.provider {
    Provider::OpenAI => openai::Client::new(&api_key),
    Provider::Anthropic => anthropic::Client::new(&api_key),
    Provider::Gemini => gemini::Client::new(&api_key),
    Provider::Ollama => ollama::Client::new(Nothing),
}
```

Return `Box<dyn Chat>` as a unified interface.

**Rationale:** rig already provides a common trait layer; matching provider to client creation is straightforward. Trait objects hide provider-specific details from upper layers.

### 4. History management architecture

**Storage format:** JSON serialized then zstd-compressed

```
data_dir/hi/history.json.zst
```

**Core structure:**

```rust
pub struct ChatHistory {
    messages: Vec<Message>,
}

impl ChatHistory {
    pub fn load() -> Self;
    pub fn save(&self);
    pub fn push(&mut self, msg: Message);
    pub fn messages(&self) -> &[Message];
    pub fn reset(&mut self);
    pub fn compact(&mut self);
    pub fn token_estimate(&self) -> usize;
}
```

**Compact strategy:**
- Estimate current history tokens (simple estimate: character count / 4)
- Auto-trigger when exceeding 80% of `context_window`
- Remove oldest messages and keep newest ~50%
- Check compact necessity before each send

**Reset:** clear `messages` and remove history file on disk

### 5. Built-in tool architecture

Implement four built-in tools with rig `Tool` trait in `hi-tools` crate:

```rust
// Each tool implements rig::tool::Tool trait
pub struct BashTool;      // execute bash commands
pub struct ListFilesTool; // list files in a target directory
pub struct ReadFileTool;  // read file content
pub struct WriteFileTool; // write content to a target file
```

**Tool args / output:**

| Tool | Args | Output |
|------|------|--------|
| `BashTool` | `{ command: String }` | `{ stdout: String, stderr: String, exit_code: i32 }` |
| `ListFilesTool` | `{ path: String }` | `{ entries: Vec<String> }` |
| `ReadFileTool` | `{ path: String }` | `{ content: String }` |
| `WriteFileTool` | `{ path: String, content: String }` | `{ success: bool }` |

Register via agent builder `.tool()` chain:

```rust
let agent = client
    .agent(model)
    .preamble(&preamble)
    .tool(BashTool)
    .tool(ListFilesTool)
    .tool(ReadFileTool)
    .tool(WriteFileTool)
    .build();
```

**Rationale:** rig natively supports tool calling, so implementing `Tool` directly is the most natural integration. Keeping tools in a dedicated crate improves testability and future extension.

### 6. Skills system

**Skills directory layout:**

```
config_dir/hi/skills/
├── coding-assistant.md
├── translator.md
└── ...
```

- Each `.md` file is one skill
- Filename (without extension) is skill name
- File content is the skill instruction/prompt
- Scan all `.md` files from `config_dir/skills/` at startup
- Append skill content after `preamble` as agent context document

**Loading logic in `hi-core`:**

```rust
pub fn load_skills(config_dir: &Path) -> Vec<Skill> {
    let skills_dir = config_dir.join("skills");
    // scan *.md and return Vec<Skill { name, content }>
}
```

**Rationale:** Markdown is easy to edit, and filesystem-based skill management is simple and intuitive. Skills are injected through rig context documents, requiring no special integration path.

### 7. TUI framework choice: ratatui

Use `ratatui` + `crossterm` as the TUI stack.

**Rationale:** `ratatui` is currently one of the most active Rust TUI frameworks, with strong community support and mature docs, suitable for chat UI development.

### 8. Async architecture

- Use `tokio` as async runtime (required by rig)
- Run TUI main loop on main thread; execute LLM requests in background via `tokio::spawn`
- Use `tokio::sync::mpsc` for communication between TUI and LLM tasks

**Rationale:** rig APIs are async-first, so Tokio is required. mpsc channel avoids blocking work on UI thread.

### 9. Dependency overview

| Crate | Package | Purpose |
|-------|---------|------|
| `directories` | shared | `ProjectDirs` for config/data paths |
| `serde`, `serde_json` | shared, hi-history, hi-tools | JSON serialization/deserialization |
| `rig-core` | hi-core, hi-tools | LLM framework + Tool trait |
| `tokio` | hi-core, hi-tools, hi-tui | async runtime + process |
| `zstd` | hi-history | history compression |
| `anyhow` | all | error handling |
| `ratatui` | hi-tui | TUI framework |
| `crossterm` | hi-tui | terminal backend |

## Risks / Trade-offs

- **Provider feature flags:** each rig provider may require feature flags -> verify against rig-core docs
- **Trait-object constraints:** `Box<dyn Chat>` may not fit rig trait design in all cases -> fallback to enum dispatch
- **TUI + async coordination:** ratatui itself is synchronous -> decouple via mpsc channel
- **Bash tool safety:** arbitrary command execution has system-level risk -> accepted by users in v1, no sandbox initially
- **Token estimate precision:** char/4 estimate is approximate -> acceptable for initial version
- **Skills reload timing:** skills are loaded once at startup, not reloaded at runtime -> app restart required for updates
