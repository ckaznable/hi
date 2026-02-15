## Context

Currently, scheduled tasks (cron jobs) are defined only in `config.json` under the `schedules` array. The `Scheduler` in `hi-core` reads these configurations at startup from `ModelConfig`. This design has limitations:
- Users must edit the main config file to manage schedules
- No runtime CLI to add/remove schedules without editing config
- Schedules are tightly coupled with provider/model configuration

The application already uses file-based persistence for history (`history.json.lz4` in `data_dir()`) and configuration (`config.json` in `config_dir()`). We will follow a similar pattern for schedules.

## Goals / Non-Goals

**Goals:**
- Store scheduled tasks in a dedicated `schedules.json` file in `data_dir()`
- Load schedules from the new file, with fallback to config.json for backward compatibility
- Provide simple file-based CRUD operations for schedules
- Keep existing config.json schedules field working (no breaking changes)

**Non-Goals:**
- Runtime file watching for schedule changes (manual reload is sufficient)
- Schedule persistence to database (file is sufficient)
- Advanced scheduling features (cron expressions only, no intervals)
- Web API for schedule management

## Decisions

### 1. Storage location: `data_dir()/schedules.json`

**Decision:** Store schedules in `data_dir()/schedules.json` alongside history.

**Rationale:**
- Follows existing pattern: history in data_dir, config in config_dir
- Natural place for runtime-generated data
- Easy to locate for debugging

**Alternatives considered:**
- `config_dir()/schedules.json`: Would mix runtime data with static config
- Next to binary: Not portable across installations

### 2. File format: Plain JSON (no compression)

**Decision:** Use plain JSON for schedules file without compression.

**Rationale:**
- Schedules are small text data, compression unnecessary
- Easier to manually edit/view
- Simpler implementation

**Alternatives considered:**
- LZ4 compression (like history): Overkill for small text data
- YAML: Less common in Rust ecosystem, more dependencies

### 3. Backward compatibility: Fallback to config.json

**Decision:** If `schedules.json` doesn't exist, fall back to reading schedules from `config.json`.

**Rationale:**
- No migration required for existing users
- Gradual adoption path
- Prevents breaking changes

**Alternatives considered:**
- Require schedules.json: Breaking change, forces migration

### 4. No runtime file watching

**Decision:** Do not implement automatic file watching for schedule changes.

**Rationale:**
- Adds complexity (notify crate dependency)
- Not frequently changed data
- Manual restart is acceptable

**Alternatives considered:**
- File watcher: Could be added later if requested

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Corrupted schedules.json | Scheduler fails to start | Validate JSON on load, fallback to empty |
| Concurrent edits | Data loss | Single writer at a time, atomic write |
| Missing schedules.json | No schedules loaded | Fall back to config.json schedules |
| Invalid cron expression | Job fails to add | Validate on load, log errors |

## Migration Plan

1. Deploy new code that checks for `schedules.json` first
2. If file doesn't exist, load from config.json (existing behavior)
3. Users can optionally create `schedules.json` to use new feature
4. Document the new file location and format

## Open Questions

1. **Should we validate cron expressions on load?** - Yes, log warnings for invalid expressions
2. **Should we support importing schedules from config.json?** - Could be added as `hi schedule import` command
3. **Should schedules be editable via CLI?** - Nice to have, can be added later
