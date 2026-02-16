# Tasks — persist-heartbeat-cron-memory-index-and-minimal-runtime-context

## 1. Create RuntimeIndex struct and persistence — DONE
- Created `shared/src/runtime_index.rs` with `RuntimeIndex { memory_sections, schedule_names, last_heartbeat_epoch }`
- `load()` / `save()` using `data_dir()/runtime_index.json`
- `refresh_memory_sections(path)` parses markdown headers
- `refresh_schedule_names(schedules)` extracts schedule names
- `build_context_preamble()` generates summary string for agents
- 11 unit tests

## 2. Wire heartbeat to runtime index — DONE
- `heartbeat.rs`: loads RuntimeIndex, passes `build_context_preamble()` as preamble to heartbeat agent
- After successful heartbeat tick, updates `last_heartbeat_epoch` and saves index

## 3. Wire scheduler to runtime index — DONE
- `scheduler.rs`: loads RuntimeIndex, passes context preamble to cron job agents

## 4. Refresh runtime index on session start — DONE
- Added `refresh_runtime_index(config, data_dir)` in `session.rs`
- Called during `ChatSession::new()` to keep index current with memory.md sections and configured schedules

## 5. Verification — DONE
- `cargo check --workspace` — clean
- `cargo test --workspace` — 188 tests, 0 failures
