## Why

The heartbeat and scheduler modules are already implemented in `hi-core` but are not integrated into the TUI or Remote startup flow. Users cannot use these background task features without this integration. Additionally, the scheduler lacks an `enabled` flag, which means it starts automatically whenever schedules exist—making it impossible to add schedules without triggering execution immediately. Adding an explicit enable flag improves user control and backwards compatibility.

## What Changes

- Integrate `HeartbeatSystem::start()` into the TUI/Remote session initialization flow
- Integrate `Scheduler::start()` into the TUI/Remote session initialization flow
- Add `enabled: bool` field to `ScheduleTaskConfig` (default: `false`) to match heartbeat behavior
- Add validation: scheduler only starts when `enabled: true` OR when a new schedule is added via CLI/tool
- When a new schedule is added via `/cron add` or `cron_add` tool, automatically set `enabled: true` and restart scheduler

## Capabilities

### New Capabilities
- `runtime-background-tasks`: Core capability to start and manage heartbeat and scheduler background tasks from config or runtime events

### Modified Capabilities
- `scheduler`: Add `enabled` flag requirement — scheduler only starts when explicitly enabled or when first schedule is added

## Impact

- **Affected code**:
  - `package/hi-core/src/session.rs` — integrate heartbeat and scheduler startup in `ChatSession::new()`
  - `package/hi-core/src/scheduler.rs` — add `enabled` field handling
  - `package/shared/src/config.rs` — add `enabled` field to `ScheduleTaskConfig`
  - `package/hi-remote/src/telegram.rs` — handle cron add to trigger scheduler enable
  - `package/hi-tools/src/schedule_add.rs` — set `enabled: true` when adding first schedule
- **Config**: `schedules[].enabled` (optional, defaults to `false`)
- **Dependencies**: No new dependencies
