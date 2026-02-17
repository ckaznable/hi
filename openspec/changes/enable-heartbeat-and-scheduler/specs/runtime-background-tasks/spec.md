## ADDED Requirements

### Requirement: Heartbeat integration in session
The system SHALL start the heartbeat background task when heartbeat is configured and enabled.

#### Scenario: Heartbeat starts when enabled
- **WHEN** config has `heartbeat.enabled: true`
- **THEN** the system SHALL start the heartbeat background task on session initialization

#### Scenario: Heartbeat does not start when disabled
- **WHEN** config has `heartbeat.enabled: false` or heartbeat config is missing
- **THEN** the system SHALL NOT start any heartbeat background task

### Requirement: Scheduler integration in session
The system SHALL start the scheduler background task when schedules are configured and enabled.

#### Scenario: Scheduler starts when enabled
- **WHEN** config has at least one schedule with `enabled: true`
- **THEN** the system SHALL start the scheduler background task on session initialization

#### Scenario: Scheduler does not start when no enabled schedules
- **WHEN** all schedules have `enabled: false` or schedules array is empty/missing
- **THEN** the system SHALL NOT start any scheduler background task

#### Scenario: Scheduler auto-enables on first schedule add
- **WHEN** a new schedule is added via CLI or tool while scheduler is not running
- **THEN** the system SHALL automatically set `enabled: true` for that schedule and start the scheduler

### Requirement: Background task result reporting
Both heartbeat and scheduler results SHALL be reported back to the main system via a channel for logging and display.

#### Scenario: Heartbeat result reported
- **WHEN** a heartbeat tick completes
- **THEN** the system SHALL send the result through the event channel

#### Scenario: Scheduler task result reported
- **WHEN** a scheduled task completes
- **THEN** the system SHALL send the task name and result through the event channel
