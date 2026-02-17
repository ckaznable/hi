## 1. Config Changes

- [x] 1.1 Add `enabled: bool` field to `ScheduleTaskConfig` in `package/shared/src/config.rs` with default `false`
- [x] 1.2 Add unit test for schedule config parsing with `enabled` field

## 2. Schedule Store Updates

- [x] 2.1 Update `schedule_store::load()` to parse `enabled` field from JSON
- [x] 2.2 Update `schedule_store::save()` to include `enabled` field in JSON output
- [x] 2.3 Add unit test for schedule store with enabled/disabled schedules

## 3. Scheduler Integration

- [x] 3.1 Modify `Scheduler::start()` to filter schedules where `enabled: true`
- [x] 3.2 Add `Scheduler::start_with_enable()` method that auto-sets enabled on first schedule
- [x] 3.3 Add unit test for scheduler filtering disabled schedules

## 4. Heartbeat Integration

- [x] 4.1 Import `HeartbeatSystem` in `session.rs`
- [x] 4.2 Add `heartbeat: Option<HeartbeatSystem>` field to `ChatSession` struct
- [x] 4.3 Call `HeartbeatSystem::start()` in `ChatSession::new()` when `config.heartbeat.enabled` is true
- [x] 4.4 Implement proper cleanup on session drop
- [x] 4.5 Add unit test for heartbeat startup behavior

## 5. Scheduler Session Integration

- [x] 5.1 Import `Scheduler` in `session.rs`
- [x] 5.2 Add `scheduler: Option<Scheduler>` field to `ChatSession` struct
- [x] 5.3 Call `Scheduler::start()` in `ChatSession::new()` when any schedule has `enabled: true`
- [x] 5.4 Implement proper cleanup on session drop
- [x] 5.5 Add unit test for scheduler startup with enabled/disabled schedules

## 6. CLI/Tool Auto-Enable

- [x] 6.1 Update `/cron add` handler in `package/hi-remote/src/telegram.rs` to auto-enable first schedule
- [x] 6.2 Update `cron_add` tool in `package/hi-tools/src/schedule_add.rs` to auto-enable first schedule
- [x] 6.3 Add integration test for auto-enable behavior

## 7. Verification

- [x] 7.1 Run `cargo check --workspace`
- [x] 7.2 Run `cargo test --workspace`
- [x] 7.3 Manual test: TUI with heartbeat enabled
- [x] 7.4 Manual test: TUI with scheduler enabled
- [x] 7.5 Manual test: Remote with cron add auto-enable
