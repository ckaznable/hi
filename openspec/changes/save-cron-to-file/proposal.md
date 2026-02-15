## Why

Currently, scheduled cron tasks are only configurable via the `config.json` file under the `schedules` array. This means users must edit the main config file to add, modify, or remove scheduled tasks. A separate dedicated file for schedules would allow users to manage cron jobs independently, making it easier to backup, share, and modify scheduled tasks without risking the main configuration.

## What Changes

- Add a new `schedules.json` file to store scheduled tasks separately from `config.json`
- Create a `ScheduleStore` module to load/save schedules from the dedicated file
- Support runtime reloading of schedules (file watcher or manual reload command)
- Fall back to config.json schedules if schedules.json doesn't exist (backward compatibility)
- Provide CLI command to list/add/remove scheduled tasks

## Capabilities

### New Capabilities
- `schedule-persistence`: Store scheduled tasks in a dedicated `schedules.json` file with CRUD operations

### Modified Capabilities
- (none - schedules currently only exist in config.json, this is a new storage location)

## Impact

- **New file**: `data_dir()/schedules.json` for storing scheduled tasks
- **Code changes**: New module in `shared` crate for schedule persistence
- **CLI**: Add `hi schedule` subcommand for managing schedules
- **Config**: Config.json `schedules` field becomes optional (falls back to schedules.json)
