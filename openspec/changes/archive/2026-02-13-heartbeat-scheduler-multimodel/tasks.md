## 1. Config extension

- [ ] 1.1 Define `SmallModelConfig` struct (`provider`, `model`, `api_key`, `api_base`, `context_window`)
- [ ] 1.2 Define `HeartbeatConfig` struct (`enabled`, `interval_secs`, `model`, `prompt`)
- [ ] 1.3 Define `ScheduleTaskConfig` struct (`name`, `cron`, `model`, `prompt`)
- [ ] 1.4 Define `ModelRef` enum (`Default`, `Small`, `Custom(SmallModelConfig)`) and serde deserialization
- [ ] 1.5 Update `ModelConfig` with new `small_model`, `heartbeat`, and `schedules` fields
- [ ] 1.6 Add unit tests for config parsing with `small_model` / `heartbeat` / `schedules`

## 2. Multi-Model Pool

- [ ] 2.1 Define `ProviderKey` (`provider + api_key hash`) for client sharing
- [ ] 2.2 Implement `ModelPool` struct: `clients: HashMap<ProviderKey, Arc<Client>>`, `agents: HashMap<ModelKey, Weak<Agent>>`
- [ ] 2.3 Implement `ModelPool::get_or_create_client()` for lazy provider-client creation and Arc clone reuse
- [ ] 2.4 Implement `ModelPool::get_agent()` to upgrade from `Weak` and recreate agent if upgrade fails
- [ ] 2.5 Implement `ModelRef::resolve()` to resolve to concrete model config
- [ ] 2.6 Add unit tests for client sharing, agent recreation after GC, and `ModelRef` resolution

## 3. Heartbeat system

- [ ] 3.1 Implement `HeartbeatSystem` struct to hold config, model ref, and `JoinHandle`
- [ ] 3.2 Implement `HeartbeatSystem::start()` to start a Tokio interval background task
- [ ] 3.3 Heartbeat task flow: get agent from `ModelPool` -> prompt -> send result via channel
- [ ] 3.4 Implement `HeartbeatSystem::stop()` to cancel background task
- [ ] 3.5 Start heartbeat in `ChatSession::new()` based on config
- [ ] 3.6 Add unit tests for heartbeat enabled/disabled behavior and interval triggering

## 4. Scheduled jobs

- [ ] 4.1 Add `tokio-cron-scheduler` dependency to hi-core
- [ ] 4.2 Implement `Scheduler` struct with task list and `JoinHandle`
- [ ] 4.3 Implement `Scheduler::start()` to parse cron expressions and spawn one Tokio task per job
- [ ] 4.4 Scheduled job flow: get agent from `ModelPool` -> prompt -> send result via channel
- [ ] 4.5 Implement `Scheduler::stop()` to cancel all scheduled jobs
- [ ] 4.6 Start scheduler in `ChatSession::new()` based on config
- [ ] 4.7 Add unit tests for cron parsing, task triggering, and empty schedule handling

## 5. Memory-minimization integration

- [ ] 5.1 Refactor `ChatSession` to use `ModelPool` instead of directly holding an agent
- [ ] 5.2 Share one `Arc<Mutex<ModelPool>>` across heartbeat and scheduler tasks
- [ ] 5.3 Verify `Weak` agent refs are released correctly when unused (drop tests)
- [ ] 5.4 Use `Cow<str>` for preamble/prompt string optimization to reduce cloning

## 6. TUI integration

- [ ] 6.1 Route heartbeat and scheduler results into existing TUI event loop via mpsc channel
- [ ] 6.2 Show heartbeat status indicator in TUI (for example in bottom status bar)
- [ ] 6.3 Show scheduled-task result notifications in TUI
