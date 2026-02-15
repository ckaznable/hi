## 1. Create Schedule Store Module

- [x] 1.1 Create `shared/src/schedule_store.rs` with `ScheduleStore` struct
- [x] 1.2 Implement `load()` function to read schedules from `data_dir()/schedules.json`
- [x] 1.3 Implement fallback to config.json when schedules.json doesn't exist
- [x] 1.4 Add validation for required fields (name, cron, prompt)
- [x] 1.5 Export module from `shared/src/lib.rs`

## 2. Integrate with Scheduler

- [x] 2.1 Update `hi-core/src/scheduler.rs` to use `ScheduleStore` for loading tasks
- [x] 2.2 Handle missing/invalid schedules.json gracefully with warnings
- [x] 2.3 Test backward compatibility with config.json schedules

## 3. Documentation and Migration

- [ ] 3.1 Update README.md with new schedules.json location and format
- [ ] 3.2 Add example schedules.json file content to documentation
- [ ] 3.3 Test the complete flow: create schedules.json, start app, verify schedules run
