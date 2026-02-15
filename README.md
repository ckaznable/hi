# hi

A terminal LLM chat tool implemented in Rust, organized as a workspace:

- `shared`: configuration and path management
- `hi-history`: chat history (JSON + LZ4 compression)
- `hi-tools`: built-in tools (`bash` / `list_files` / `read_file` / `write_file` / `read_skills`)
- `hi-core`: agent/session logic, skill loading, context injection, heartbeat, scheduling
- `hi-tui`: interactive TUI built with `ratatui` + `crossterm`
- `hi-remote`: bridge for external communication apps (currently Telegram)

## Feature Summary

- Single chat session model (no multi-session switching)
- Streaming responses (real-time display of model output)
- Automatic local history persistence with LZ4 compression
- Built-in tool calling support:
  - `bash`
  - `list_files`
  - `read_file`
  - `write_file`
  - `read_skills`
- Skill system: loads `skills/*.md` with optional `description` in frontmatter
- Optimized context injection:
  - Full context on first injection
  - No reinjection when unchanged
  - Delta updates when context changes
- Optional heartbeat task (fixed interval)
- Optional cron scheduling (`tokio-cron-scheduler`)
- Telegram bot remote mode (via `hi-remote`, one independent session per chat)

## Not Supported Yet

- RAG / embeddings / vector search
- Web API / HTTP server

## Requirements

- Rust stable (latest recommended)
- Cargo

## Install and Run

### Option 1: Run directly (common during development)

```bash
# Start TUI (requires tui feature)
cargo run --features tui -- tui

# Start Telegram remote mode
cargo run -- remote
```

### Option 2: Install a local executable

```bash
# Default install (without TUI)
cargo install --path .

# Install with TUI support
cargo install --path . --features tui

# Use after install
hi tui
hi remote
```

## Quick Start

1. Generate the default config file:

```bash
hi init
```

   This creates `config.json` at the standard config path (e.g. `~/.config/hi/config.json` on Linux).

2. Edit the generated config with your provider and API key.

3. Run TUI:

```bash
cargo run --features tui -- tui
```


### Option 2: Install a local executable

```bash
# Default install (without TUI)
cargo install --path bin/hi

# Install with TUI support
cargo install --path bin/hi --features tui

# Use after install
hi tui
hi remote
```

## Quick Start

1. Generate the default config file:

```bash
hi init
```

   This creates `config.json` at the standard config path (e.g. `~/.config/hi/config.json` on Linux).

2. Edit the generated config with your provider and API key.

3. Run TUI:

```bash
cargo run -p hi-cli --features tui -- tui
```

> You can also run `hi tui` if the executable was installed with `--features tui`.

## Configuration Examples

### Minimal working config

These four fields are enough to start:

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxxx",
  "context_window": 128000
}
```

If you use `ollama`, `api_key` can be omitted:

```json
{
  "provider": "ollama",
  "model": "qwen2.5:14b",
  "context_window": 32000
}
```

If you use an OpenAI-compatible endpoint (for example LiteLLM, vLLM, LocalAI):

```json
{
  "provider": "openai-compatible",
  "model": "gpt-4o-mini",
  "api_base": "http://localhost:8080/v1",
  "context_window": 32000
}
```

OpenAI-compatible endpoint with API key:

```json
{
  "provider": "openai-compatible",
  "model": "gpt-4o-mini",
  "api_key": "your-key",
  "api_base": "https://gateway.example.com/v1",
  "context_window": 32000
}
```

Main `config.json` fields:

- `provider`: `openai` / `openai-compatible` / `anthropic` / `gemini` / `ollama`
- `model`: primary model name
- `api_key`: required for providers except `openai-compatible` and `ollama`
- `api_base`: optional custom endpoint (`required` for `openai-compatible`)
- `preamble`: optional system prompt
- `context_window`: context size (integer)
- `small_model`: optional small-model config
- `compact`: optional context compaction settings
- `heartbeat`: optional heartbeat settings
- `schedules`: optional list of scheduled tasks
- `memory`: optional memory reclamation policy
- `remote`: optional remote adapter config (includes session lifecycle controls)

Example:

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxxx",
  "context_window": 128000,
  "preamble": "You are a helpful coding assistant.",
  "small_model": {
    "provider": "ollama",
    "model": "qwen2.5:3b",
    "context_window": 4096
  },
  "heartbeat": {
    "enabled": true,
    "interval_secs": 300,
    "model": "small",
    "prompt": "heartbeat check"
  },
  "schedules": [
    {
      "name": "daily-summary",
      "cron": "0 0 * * *",
      "model": "small",
      "prompt": "Generate a daily summary."
    }
  ]
}
```

## Compact Settings

When a conversation approaches the context window limit, history is compacted automatically. Two strategies are supported:

### Truncate mode (default)

Drops older messages directly:

```json
{
  "compact": {
    "enabled": true,
    "strategy": "truncate",
    "trigger_ratio": 0.8
  }
}
```

### Small-model summary mode

Uses a small model to summarize previous conversation and preserve more meaning:

```json
{
  "compact": {
    "enabled": true,
    "strategy": "small-model",
    "model": "small",
    "trigger_ratio": 0.8,
    "prompt": "Summarize the following conversation concisely."
  }
}
```

- `strategy`: `truncate` or `small-model`
- `model`: optional model for summarization (defaults to `small_model`; if missing, falls back to primary model)
- `trigger_ratio`: optional compaction threshold ratio (default `0.8`, i.e. 80% of context window)
- `prompt`: optional custom summary prompt
- If small-model summarization fails, it automatically falls back to truncate mode
- After compaction, user language is tagged in context (for example `[User Language: Chinese]`) to keep reply language consistent

## Memory Reclamation Policy

The runtime emits memory reclamation logs after large deallocations (compact, reset). Allocator-specific trim calls are currently disabled for target compatibility (including musl).

```json
{
  "memory": {
    "large_release_threshold_bytes": 1048576
  }
}
```

- `large_release_threshold_bytes`: minimum bytes released before logging a reclamation event (default `1048576`, 1 MB)

When a qualifying release occurs, a structured log line is emitted to stderr:

```
[memory] Reclamation: released 2097152 bytes, threshold 1048576, action: reclaim
```

Non-qualifying releases log `action: skip`. Qualifying releases log `action: reclaim`.

## Telegram Remote Mode

Extends LLM chat to Telegram through the Telegram Bot API. Each Telegram `chat_id` maintains an independent `ChatSession`.

### Start command

```bash
cargo run -- remote
```

> You can also run `hi remote` if the executable is installed.

### Configuration example

Add `remote` section in `config.json`:

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxxx",
  "context_window": 128000,
  "remote": {
    "telegram": {
      "enabled": true,
      "bot_token": "123456:ABC-DEF...",
      "poll_timeout_secs": 30
    },
    "session": {
      "ttl_secs": 3600,
      "max_sessions": 100
    }
  }
}
```

- `enabled`: enable Telegram remote mode
- `bot_token`: bot token from [@BotFather](https://t.me/BotFather)
- `poll_timeout_secs`: long polling timeout in seconds (default `30`)
- `session.ttl_secs`: idle session time-to-live in seconds (default `3600`). Sessions inactive longer than this are evicted on next access.
- `session.max_sessions`: maximum concurrent sessions (default `100`). When the limit is reached, the oldest idle session is evicted to make room.

### Behavior

- Receives messages through `getUpdates` long polling
- One independent session per `chat_id`
- Idle sessions are evicted after `ttl_secs` of inactivity
- When session count reaches `max_sessions`, the oldest idle session is evicted before creating a new one
- Splits and sends replies automatically if output exceeds 4096 characters
- Automatically waits and retries on Telegram rate limits (`429`)

## Skills Usage

Create a `skills/` directory under config directory, with one `.md` per skill:

```md
---
description: Expert Rust guidance
---
You are a senior Rust engineer focusing on correctness and performance.
```

- Filename is used as skill name
- `description` is used for skill summary
- Body content is merged into preamble/context

## TUI Controls

- Type message and press Enter to send
- `/reset`: clear current history
- `/quit` or `/exit`: quit
- `Esc` or `Ctrl+C`: quit

## CLI Subcommands

- `init`: create a starter config template at the default config path
- `tui`: start interactive terminal chat UI (requires `--features tui` at build time)
- `remote`: start Telegram bot long-polling mode
- `config validate`: validate config by sending a test message to the configured LLM provider

## Data Storage

- Config: `config_dir()/config.json`
- History: `data_dir()/history.json.lz4`

Actual paths are resolved by the `directories` crate per operating system.

## Development Commands

```bash
cargo check --workspace
cargo test --workspace
```
