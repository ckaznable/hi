## MODIFIED Requirements

### Requirement: Heartbeat interval
The system SHALL support a configurable heartbeat that periodically executes heartbeat tasks from `HEARTBEAT.md` using a specified model.

#### Scenario: Heartbeat enabled
- **WHEN** the config has `heartbeat.enabled: true` and `heartbeat.interval_secs: 300`
- **THEN** the system SHALL evaluate `HEARTBEAT.md` every 300 seconds and execute runnable heartbeat task items using the specified model

#### Scenario: Heartbeat disabled
- **WHEN** the config has `heartbeat.enabled: false`
- **THEN** the system SHALL NOT start any heartbeat background task

#### Scenario: Heartbeat uses small model
- **WHEN** the heartbeat config has `model: "small"`
- **THEN** the system SHALL use the `small_model` for heartbeat task execution

#### Scenario: Heartbeat uses default model
- **WHEN** the heartbeat config omits the `model` field
- **THEN** the system SHALL use the default heartbeat model selection behavior defined by config defaults

### Requirement: Heartbeat result reporting
The heartbeat result SHALL be reported back to the main system via a channel for logging or TUI display and SHALL be reflected in `HEARTBEAT.md` task status.

#### Scenario: Heartbeat result received
- **WHEN** a heartbeat task completes
- **THEN** the system SHALL send the result through a channel to the main event loop

#### Scenario: Ledger status update after run
- **WHEN** heartbeat task execution finishes
- **THEN** the system SHALL persist updated task status in `HEARTBEAT.md` before the next tick
