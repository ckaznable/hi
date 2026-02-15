## Context

Based on the existing `simple-llm-chat` architecture, the system currently uses one `ModelConfig` for one LLM model. We need to extend it with multi-model support, heartbeat, scheduling, and better memory efficiency.

## Goals / Non-Goals

**Goals:**

- Support `small_model` configuration for lightweight background tasks
- Heartbeat system: periodic background tasks with optional model selection
- Scheduled jobs: cron-like mechanism with per-task model selection
- Multi-model management: lazy initialization and shared provider clients
- Minimize runtime memory usage

**Non-Goals:**

- No automatic model selection (model choice stays user-configured)
- No distributed scheduling
- No persistent scheduler state (reload config on restart)

## Decisions

### 1. Config format extension

```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-...",
  "context_window": 128000,
  "preamble": "You are a helpful assistant.",

  "small_model": {
    "provider": "ollama",
    "model": "qwen2.5:3b",
    "api_key": null,
    "context_window": 4096
  },

  "heartbeat": {
    "enabled": true,
    "interval_secs": 300,
    "model": "small",
    "prompt": "Summarize the current session status in one line."
  },

  "schedules": [
    {
      "name": "daily-summary",
      "cron": "0 0 * * *",
      "model": "small",
      "prompt": "Generate a daily summary of recent conversations."
    }
  ]
}
```

**Model reference styles (`model` field value):**
- `"default"` or omitted -> use primary model
- `"small"` -> use `small_model`
- `{ "provider": "...", "model": "..." }` -> inline custom model

**Rationale:** flexible but simple. Most users only need `"small"` or `"default"`; advanced users can provide inline custom model config.

### 2. Multi-model manager (ModelPool)

```rust
pub struct ModelPool {
    clients: HashMap<ProviderKey, Arc<dyn ProviderClient>>,
    agents: HashMap<ModelKey, Weak<dyn Chat>>,
}

impl ModelPool {
    pub async fn get_agent(&mut self, model_ref: &ModelRef) -> Arc<dyn Chat>;
    fn get_or_create_client(&mut self, provider: &Provider, api_key: &str) -> Arc<dyn ProviderClient>;
}
```

**Memory optimization strategy:**
- **Shared clients**: one client per identical `provider + api_key`, shared via `Arc`
- **Lazy init**: do not pre-create all clients; build only when needed
- **Weak refs**: hold agents as `Weak<>` and auto-release when unused
- **On-demand upgrade**: upgrade `Weak` to `Arc` when needed, then let it drop back

**Rationale:** avoids keeping unused model objects alive. `Weak` leverages Rust reference counting for lifecycle management.

### 3. Heartbeat system

```rust
pub struct HeartbeatSystem {
    config: HeartbeatConfig,
    model_ref: ModelRef,
    handle: Option<JoinHandle<()>>,
}
```

- Use `tokio::time::interval` for periodic triggering
- Each trigger calls `prompt()` on the selected model
- Return results to the main system via mpsc channel (for TUI display or logging)
- No background task starts when `enabled: false`

**Rationale:** simple interval-based heartbeat without additional external dependency.

### 4. Scheduled jobs

```rust
pub struct Scheduler {
    tasks: Vec<ScheduledTask>,
    handles: Vec<JoinHandle<()>>,
}

pub struct ScheduledTask {
    name: String,
    cron: String,
    model_ref: ModelRef,
    prompt: String,
}
```

- Use `tokio-cron-scheduler` to parse cron expressions
- Run each job in its own Tokio task
- Share `ModelPool` via `Arc<Mutex<ModelPool>>`
- Report outputs through channel

**Rationale:** cron syntax is the standard for scheduling, and `tokio-cron-scheduler` fits the Tokio ecosystem.

### 5. Memory minimization strategy overview

| Strategy | Implementation |
|------|------|
| Shared provider clients | `HashMap<ProviderKey, Arc<Client>>` |
| Agent lazy init | Create only when needed |
| Agent weak refs | `Weak<dyn Chat>` auto-reclaim |
| History compression | zstd reduces disk I/O buffer usage |
| Lazy context injection | No duplicate injection (existing context manager) |
| String minimization | Use `Cow<str>` to avoid unnecessary clones |

### 6. New dependency

| Crate | Package | Purpose |
|-------|---------|------|
| `tokio-cron-scheduler` | hi-core | cron scheduling |

## Risks / Trade-offs

- **Weak-ref complexity**: `Weak` may already be dropped on access; handle upgrade failure by recreating the agent
- **Scheduling precision**: Tokio timers are not real-time (millisecond-level precision), which is sufficient for LLM tasks
- **ModelPool lock contention**: multiple jobs share `Arc<Mutex<ModelPool>>`; lock is held briefly during agent creation
- **Heartbeat cost**: even `small_model` still incurs API cost; users can increase interval or disable it
