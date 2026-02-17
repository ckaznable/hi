## Context

The heartbeat and scheduler modules are already implemented in `hi-core`:
- `HeartbeatSystem` in `package/hi-core/src/heartbeat.rs` — periodic background tasks with configurable interval
- `Scheduler` in `package/hi-core/src/scheduler.rs` — cron-expression-based scheduled jobs using `tokio-cron-scheduler`

However, these are never instantiated in the TUI or Remote session startup flow. The `ChatSession::new()` in `session.rs` only creates the agent, history, and context manager—but does not start any background tasks.

Additionally, the scheduler lacks an `enabled` flag, making it impossible to add schedules without immediately triggering execution. This prevents users from pre-configuring schedules for later enablement.

## Goals / Non-Goals

**Goals:**
- Integrate heartbeat startup into TUI/Remote session initialization
- Integrate scheduler startup into TUI/Remote session initialization  
- Add `enabled` field to schedule config with sensible defaults
- Auto-enable scheduler when first schedule is added via CLI/tool

**Non-Goals:**
- Implement heartbeat/scheduler persistence across restarts (already handled)
- Add UI controls for enabling/disabling at runtime (future enhancement)
- Support dynamic interval changes without restart (future enhancement)

## Decisions

### Decision 1: Where to integrate startup

**Choice:** Integrate in `ChatSession::new()` after agent creation

**Rationale:** 
- `ChatSession` already holds config, agent, and has access to the event channel
- Both TUI and Remote modes go through `ChatSession::new()`, ensuring consistent behavior
- The session owns the lifecycle, so stopping session naturally stops background tasks

**Alternative considered:** Start in `main.rs` before session creation. Rejected because it would require passing heartbeat/scheduler handles separately, complicating the API.

### Decision 2: Scheduler enabled flag default

**Choice:** Default `enabled: false` for backwards compatibility

**Rationale:**
- Existing configs with schedules (added before this feature) should not suddenly start running
- Users must explicitly opt-in to scheduler execution
- New schedules added via CLI/tool will auto-enable

**Alternative considered:** Default `enabled: true`. Rejected because it breaks existing behavior and surprises users who add schedules for later use.

### Decision 3: Auto-enable on first schedule add

**Choice:** When `/cron add` or `cron_add` tool creates the first schedule and scheduler is not running, set `enabled: true` and start scheduler

**Rationale:**
- Provides immediate feedback that the schedule will run
- Avoids confusing user "why isn't my schedule running?" scenario
- Single action completes the intent: "add a schedule and run it"

**Alternative considered:** Always require manual enable after add. Rejected because it adds friction without clear benefit.

### Decision 4: Event channel for results

**Choice:** Reuse existing `mpsc::UnboundedSender<String>` pattern from existing code

**Rationale:**
- Heartbeat and scheduler already use this pattern in their implementations
- TUI already has channel infrastructure for streaming
- Remote mode can similarly receive and log results

## Risks / Trade-offs

- **[Risk] Background task lifecycle management** → Current implementation stores handles in `HeartbeatSystem` and `Scheduler` structs. Need to ensure proper drop behavior when session ends.
- **[Risk] Scheduler restart on schedule add** → When adding a new schedule at runtime, need to either restart scheduler or use dynamic job addition. `tokio-cron-scheduler` supports job addition after start.
- **[Risk] Concurrent schedule writes** → If user edits `schedules.json` while scheduler is running, need to handle potential race conditions. Mitigation: Use atomic write pattern already in `schedule_store`.

## Migration Plan

1. Add `enabled` field to `ScheduleTaskConfig` in `shared/src/config.rs`
2. Update `schedule_store` to load/save the `enabled` field
3. Modify `scheduler.rs` to check `enabled` flag before starting jobs
4. Integrate `HeartbeatSystem::start()` in `session.rs`
5. Integrate `Scheduler::start()` in `session.rs`
6. Update `/cron add` command to auto-enable first schedule
7. Update `cron_add` tool to auto-enable first schedule

## Open Questions

- Should heartbeat also have an auto-enable behavior when HEARTBEAT.md is first created? (Current design: heartbeat requires explicit `enabled: true` in config, simpler than scheduler)
- Should we add a `/scheduler enable` / `/scheduler disable` command for runtime control?
