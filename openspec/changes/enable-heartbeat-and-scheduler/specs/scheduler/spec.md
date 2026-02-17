## MODIFIED Requirements

### Requirement: Scheduler enabled flag
Each scheduled task SHALL support an optional `enabled` field (boolean) to control whether the scheduler background task runs.

#### Scenario: Scheduler starts with enabled schedule
- **WHEN** config has at least one schedule with `enabled: true`
- **THEN** the system SHALL start the scheduler background task

#### Scenario: Scheduler does not start when all disabled
- **WHEN** all schedules have `enabled: false` or the `enabled` field is omitted (defaults to false)
- **THEN** the system SHALL NOT start any scheduler background task

#### Scenario: First schedule auto-enables scheduler
- **WHEN** a user adds a new schedule via `/cron add` or `cron_add` tool and no scheduler is currently running
- **THEN** the system SHALL set `enabled: true` for the new schedule and start the scheduler

### Requirement: No schedules configured (UPDATED)
- **WHEN** the config has an empty or missing `schedules` array AND no schedules have been added at runtime
- **THEN** the system SHALL NOT start any scheduler background tasks

#### Scenario: Empty schedules
- **WHEN** `schedules` is `[]` and no runtime schedules exist
- **THEN** no scheduler tasks SHALL be created
