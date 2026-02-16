# Tasks: add-llm-schedule-view-tool

## Implementation

- [x] Create `package/hi-tools/src/schedule_view.rs` with `ScheduleViewTool` implementing `rig::tool::Tool`
- [x] Tool reads `schedules.json` directly (no `shared` dependency — keeps crate boundaries clean)
- [x] Implement `load_schedules()` — reads JSON, filters invalid entries
- [x] Implement `format_schedule()` — human-readable formatting with name, cron, model, prompt
- [x] Implement `call()` — list all schedules or filter by name (case-insensitive)
- [x] Register `pub mod schedule_view` and `pub use schedule_view::ScheduleViewTool` in `hi-tools/src/lib.rs`
- [x] Add `ScheduleViewTool` to `build_tools()` in `hi-core/src/provider.rs` with `data_dir()/schedules.json` path
- [x] Add `"view_schedules: View configured cron schedules"` to tool_descriptions in both `send_message` and `send_message_streaming`

## Tests (10 in `schedule_view::tests`)

- [x] `test_load_schedules_nonexistent` — missing file returns empty vec
- [x] `test_load_schedules_valid` — parses valid JSON with model field
- [x] `test_load_schedules_filters_invalid` — skips entries with empty name/cron/prompt
- [x] `test_format_schedule_with_model` — formatting includes model name
- [x] `test_format_schedule_without_model` — formatting shows (default) when no model
- [x] `test_view_schedules_empty` — no file returns "No schedules configured."
- [x] `test_view_schedules_list_all` — lists all with count header
- [x] `test_view_schedules_filter_by_name` — filters to matching schedule only
- [x] `test_view_schedules_name_not_found` — shows available names when not found
- [x] `test_view_schedules_case_insensitive` — name matching is case-insensitive

## Verification

- [x] `cargo check --workspace` — clean
- [x] `cargo test --workspace` — 165 passed, 0 failed
