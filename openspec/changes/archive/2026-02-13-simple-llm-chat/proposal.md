## Why

The current `hi` project is an empty Rust project. We need to build a simple LLM chat system so users can interact with an LLM through a TUI. We use [rig](https://github.com/0xPlaygrounds/rig) as the core LLM framework, and JSON config for flexible model switching so users can change provider/model settings without recompiling.

## What Changes

- Convert the project into a Cargo workspace with five packages:
  - `package/shared`: shared utilities such as config and data paths (using `ProjectDirs` from `directories`)
  - `package/hi-core`: core LLM chat logic with rig integration, compact/reset support, built-in tools, and skills loading
  - `package/hi-history`: chat history management with JSON + zstd compression
  - `package/hi-tools`: built-in tools (`bash`, file listing, file read/write)
  - `package/hi-tui`: TUI chat interface (binary crate)
- Add JSON model config system under `ProjectDirs::config_dir()`
- Add chat-history persistence under `ProjectDirs::data_dir()` using JSON + zstd
- Add built-in tools: `bash`, list files, read file, write file
- Add skills system: load custom skill definitions from `config_dir/skills/` and inject into agent preamble
- Use a single-session design: reuse one history timeline for the whole chat
- Add `compact` / `reset`
- Add dependencies: `rig-core`, `serde`, `serde_json`, `tokio`, `directories`, `zstd`

## Capabilities

### New Capabilities

- `model-config`: generic JSON model config system to read/parse provider, model name, API key, etc.; config path resolved via `ProjectDirs`
- `chat-history`: history management stored in `data_dir` as JSON + zstd; supports single-session reuse, compact (context-window compression), and reset (clear)
- `builtin-tools`: built-in toolset implemented via rig `Tool` trait (`bash`, list/read/write files)
- `skills-loader`: load skill markdown files from `config_dir/skills/` and inject into agent context
- `chat-session`: rig-based chat session core integrating history, tools, and skills; supports multi-turn chat, compact, and reset
- `chat-tui`: terminal TUI chat UI for interactive usage

### Modified Capabilities

_No existing capabilities require modification._

## Impact

- **project structure**: move from single crate to workspace and add `package/shared`, `package/hi-core`, `package/hi-history`, `package/hi-tools`, `package/hi-tui`
- **dependencies**: add `rig-core`, `serde`, `serde_json`, `tokio`, `directories`, `zstd`, `ratatui`, `crossterm`, `anyhow`
- **configuration/data paths**: config in `ProjectDirs::config_dir()`, history in `ProjectDirs::data_dir()`, skills in `config_dir/skills/`
- **security**: `bash` tool has system-level access; users must accept associated risks
- **runtime requirements**: valid LLM API key required for normal operation
